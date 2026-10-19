/// Gestor de tareas de descarga (D-12) — docs/spec/architecture.md §4.
/// Dueño de la cola y de la máquina de estados (T1–T6); cada tarea lanza
/// su propio proceso del motor con argumentos cerrados (RNF-01) y señales
/// para pausar/cancelar; los eventos viajan a la UI por `task://*`.
///
/// Aviso (documentado): la pausa se aplica entre líneas del progreso del
/// motor; en descargas activas las líneas llegan con alta frecuencia.
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::app::naming;
use crate::models::errors::{ErrorCode, TaskError};
use crate::models::history::{redact_url, HistoryEntry};
use crate::models::media::{MediaInfo, QualityOption};
use crate::models::prefs::ConflictPolicy;
use crate::models::progress::{parse_motor_line, MotorLine, Phase, ProgressInfo};
use crate::models::task::{DownloadTask, OutputFormat, TaskKind, TaskState};
use crate::persistence::Store;
use crate::services::{ffmpeg, tool_locator, ytdlp};

/// Señal de control compartida entre la API y el runner de la tarea.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MotorSignal {
    Ninguna,
    Pausar,
    Cancelar,
}

/// Resultado del análisis: el id de la tarea creada y el resultado del motor
/// (`media` Some = `ready`; `error` Some = la tarea queda `failed` en la cola).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisOutcome {
    pub task_id: String,
    pub media: Option<MediaInfo>,
    pub error: Option<TaskError>,
}

/// Evento de progreso hacia la UI.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub task_id: String,
    pub progress: ProgressInfo,
}

pub struct TaskManager {
    tasks: Mutex<BTreeMap<String, DownloadTask>>,
    signals: Mutex<HashMap<String, Arc<Mutex<MotorSignal>>>>,
    app: AppHandle,
    next_seq: AtomicU64,
}

impl TaskManager {
    pub fn new(app: AppHandle) -> Arc<Self> {
        Arc::new(Self {
            tasks: Mutex::new(BTreeMap::new()),
            signals: Mutex::new(HashMap::new()),
            app,
            next_seq: AtomicU64::new(1),
        })
    }

    fn new_id(&self) -> String {
        let millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        format!(
            "{millis:x}-{:x}",
            self.next_seq.fetch_add(1, Ordering::Relaxed)
        )
    }

    fn now_iso() -> String {
        chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    }

    fn snapshot(&self, id: &str) -> Option<DownloadTask> {
        self.tasks.lock().expect("tasks lock").get(id).cloned()
    }

    /// Transición validada (T1) + emisión del nuevo estado a la UI (T5).
    fn set_state(&self, id: &str, next: TaskState) -> Result<(), TaskError> {
        let mut tasks = self.tasks.lock().expect("tasks lock");
        let task = tasks
            .get_mut(id)
            .ok_or_else(|| TaskError::new(ErrorCode::Ec23, Some(format!("tarea {id} inexistente"))))?;
        if !task.state.can_transition_to(next) {
            let from = task.state;
            return Err(TaskError::new(
                ErrorCode::Ec23,
                Some(format!("transición ilegal {from:?} → {next:?}")),
            ));
        }
        task.state = next;
        task.updated_at = Self::now_iso();
        let snapshot = task.clone();
        drop(tasks);
        let _ = self.app.emit("task://state", snapshot);
        Ok(())
    }

    /// Análisis (RF-01/RF-02): crea la tarea (`queued → inspecting`) y termina
    /// en `ready` con metadatos, o `failed` con el error del catálogo.
    pub async fn run_analysis(&self, normalized_url: &str) -> AnalysisOutcome {
        let id = self.new_id();
        let task = DownloadTask {
            id: id.clone(),
            kind: TaskKind::Download,
            seq: self.next_seq.fetch_add(1, Ordering::Relaxed),
            source_url: Some(normalized_url.to_string()),
            portal: None,
            title: None,
            duration_sec: None,
            input_path: None,
            qualities: Vec::new(),
            selected_quality_id: None,
            selected_format: None,
            destination: None,
            state: TaskState::Queued,
            progress: None,
            error: None,
            retry_count: 0,
            created_at: Self::now_iso(),
            updated_at: Self::now_iso(),
        };
        self.tasks
            .lock()
            .expect("tasks lock")
            .insert(id.clone(), task);
        self.set_state(&id, TaskState::Inspecting)
            .expect("queued → inspecting es legal");

        match ytdlp::analyze(normalized_url).await {
            Ok(info) => {
                {
                    let mut tasks = self.tasks.lock().expect("tasks lock");
                    if let Some(task) = tasks.get_mut(&id) {
                        task.portal = info.portal.clone();
                        task.title = info.title.clone();
                        task.duration_sec = info.duration_sec;
                        task.qualities = info.qualities.clone();
                    }
                }
                self.set_state(&id, TaskState::Ready)
                    .expect("inspecting → ready es legal");
                AnalysisOutcome { task_id: id, media: Some(info), error: None }
            }
            Err(e) => {
                {
                    let mut tasks = self.tasks.lock().expect("tasks lock");
                    if let Some(task) = tasks.get_mut(&id) {
                        task.error = Some(e.clone());
                    }
                }
                self.set_state(&id, TaskState::Failed)
                    .expect("inspecting → failed es legal");
                let failed = self.snapshot(&id).expect("acaba de crearse");
                self.record_history(&failed);
                AnalysisOutcome { task_id: id, media: None, error: Some(e) }
            }
        }
    }

    /// Inicia la descarga de una tarea `ready` (RF-03/RF-05/D-05). Devuelve el
    /// destino final resuelto (con la política de conflicto aplicada).
    pub async fn start_download(
        self: &Arc<Self>,
        task_id: &str,
        quality_id: &str,
        format: OutputFormat,
        destination_base: &str,
        policy: ConflictPolicy,
    ) -> Result<String, TaskError> {
        let task = self
            .snapshot(task_id)
            .ok_or_else(|| TaskError::new(ErrorCode::Ec23, Some("tarea inexistente".into())))?;
        if task.state != TaskState::Ready {
            return Err(TaskError::new(
                ErrorCode::Ec23,
                Some(format!("la tarea está en estado {:?}", task.state)),
            ));
        }
        let quality: QualityOption = task
            .qualities
            .iter()
            .find(|q| q.id == quality_id)
            .cloned()
            .ok_or_else(|| TaskError::new(ErrorCode::Ec23, Some("calidad no disponible".into())))?;

        let extension = match format {
            OutputFormat::Original => {
                if quality.container.is_empty() {
                    "mp4".to_string()
                } else {
                    quality.container.clone()
                }
            }
            OutputFormat::Mp4 => "mp4".to_string(),
            OutputFormat::Mp3 => "mp3".to_string(),
        };

        // D-04: la política de conflicto se aplica en el backend (naming) sobre
        // el destino propuesto por el diálogo nativo de guardado.
        let base = naming::strip_extension(destination_base);
        let filename = if base.is_empty() {
            naming::sanitize_filename(task.title.as_deref().unwrap_or("descarga"))
        } else {
            base.to_string()
        };
        let final_destination = naming::resolve_destination(&filename, &extension, policy)?;

        {
            let mut tasks = self.tasks.lock().expect("tasks lock");
            let task = tasks
                .get_mut(task_id)
                .ok_or(TaskError::new(ErrorCode::Ec23, None))?;
            task.selected_quality_id = Some(quality_id.to_string());
            task.selected_format = Some(format);
            task.destination = Some(final_destination.clone());
        }
        self.set_state(task_id, TaskState::Downloading)?;
        Self::spawn_runner(Arc::clone(self), task_id.to_string());
        Ok(final_destination)
    }

    /// Pausa (D-05/T6): señal al runner; el estado pasa por `pausing` y el
    /// runner aplica `paused` cuando confirma la muerte del proceso.
    pub fn pause_task(&self, task_id: &str) -> Result<(), TaskError> {
        let state = self
            .snapshot(task_id)
            .ok_or_else(|| TaskError::new(ErrorCode::Ec23, Some("tarea inexistente".into())))?
            .state;
        if state != TaskState::Downloading {
            return Err(TaskError::new(
                ErrorCode::Ec23,
                Some("solo una descarga activa se puede pausar".into()),
            ));
        }
        self.send_signal(task_id, MotorSignal::Pausar);
        self.set_state(task_id, TaskState::Pausing)
    }

    /// Reanuda (D-05/T6): re-lanza el motor; los `.part` continúan.
    pub async fn resume_task(self: &Arc<Self>, task_id: &str) -> Result<(), TaskError> {
        let state = self
            .snapshot(task_id)
            .ok_or_else(|| TaskError::new(ErrorCode::Ec23, Some("tarea inexistente".into())))?
            .state;
        if state != TaskState::Paused {
            return Err(TaskError::new(
                ErrorCode::Ec23,
                Some("solo una tarea pausada se puede reanudar".into()),
            ));
        }
        self.set_state(task_id, TaskState::Downloading)?;
        Self::spawn_runner(Arc::clone(self), task_id.to_string());
        Ok(())
    }

    /// Cancela (T4): estados vivos → señal al runner (que limpia); estados
    /// tempranos → transición directa + historial.
    pub fn cancel_task(&self, task_id: &str) -> Result<(), TaskError> {
        let state = self
            .snapshot(task_id)
            .ok_or_else(|| TaskError::new(ErrorCode::Ec23, Some("tarea inexistente".into())))?
            .state;
        match state {
            TaskState::Downloading | TaskState::Pausing | TaskState::Converting => {
                self.send_signal(task_id, MotorSignal::Cancelar);
                Ok(())
            }
            TaskState::Queued | TaskState::Inspecting | TaskState::Ready | TaskState::Paused => {
                self.set_state(task_id, TaskState::Cancelled)?;
                let task = self.snapshot(task_id).expect("acaba de leerse");
                self.record_history(&task);
                Ok(())
            }
            other => Err(TaskError::new(
                ErrorCode::Ec23,
                Some(format!("estado {other:?} no admite cancelación")),
            )),
        }
    }

    /// Reintenta (T3): solo `failed`/`cancelled` con configuración previa; nunca
    /// crea procesos duplicados porque el runner es el único lanzador.
    pub async fn retry_task(self: &Arc<Self>, task_id: &str) -> Result<(), TaskError> {
        let task = self
            .snapshot(task_id)
            .ok_or_else(|| TaskError::new(ErrorCode::Ec23, Some("tarea inexistente".into())))?;
        if task.state != TaskState::Failed && task.state != TaskState::Cancelled {
            return Err(TaskError::new(
                ErrorCode::Ec23,
                Some(format!("{:?} no admite reintento", task.state)),
            ));
        }
        if task.qualities.is_empty() {
            return Err(TaskError::new(
                ErrorCode::Ec23,
                Some("el análisis falló; repita el análisis de la URL".into()),
            ));
        }
        {
            let mut tasks = self.tasks.lock().expect("tasks lock");
            if let Some(task) = tasks.get_mut(task_id) {
                task.retry_count += 1;
                task.error = None;
            }
        }
        self.set_state(task_id, TaskState::Downloading)?;
        Self::spawn_runner(Arc::clone(self), task_id.to_string());
        Ok(())
    }

    /// Inicia la conversión de un archivo local (C-20/RF-06). Crea la tarea,
    /// resuelve el destino con la política (D-04) y lanza el runner.
    /// Devuelve el id de la tarea; el progreso llega por `task://state`.
    pub async fn start_conversion(
        self: &Arc<Self>,
        input_path: &str,
        format: OutputFormat,
        destination_base: &str,
        policy: ConflictPolicy,
    ) -> Result<String, TaskError> {
        if !std::path::Path::new(input_path).is_file() {
            return Err(TaskError::new(ErrorCode::Ec30, Some("el archivo de entrada no existe".into())));
        }
        // Para conversión local: MP4 y MP3 (el "Original" es de descargas).
        let extension = match format {
            OutputFormat::Mp4 => "mp4",
            OutputFormat::Mp3 => "mp3",
            OutputFormat::Original => {
                return Err(TaskError::new(ErrorCode::Ec23, Some("Original no aplica a conversión local".into())))
            }
        }
        .to_string();

        let id = self.new_id();
        let base = naming::strip_extension(destination_base);
        let filename = if base.is_empty() {
            naming::sanitize_filename(
                std::path::Path::new(input_path)
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .as_deref()
                    .unwrap_or("reconvertir"),
            )
        } else {
            base.to_string()
        };
        let final_destination = naming::resolve_destination(&filename, &extension, policy)?;

        let task = DownloadTask {
            id: id.clone(),
            kind: TaskKind::Convert,
            seq: self.next_seq.fetch_add(1, Ordering::Relaxed),
            source_url: None,
            portal: None,
            title: std::path::Path::new(input_path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string()),
            duration_sec: None,
            qualities: Vec::new(),
            input_path: Some(input_path.to_string()),
            selected_quality_id: None,
            selected_format: Some(format),
            destination: Some(final_destination),
            state: TaskState::Queued,
            progress: None,
            error: None,
            retry_count: 0,
            created_at: Self::now_iso(),
            updated_at: Self::now_iso(),
        };
        self.tasks.lock().expect("tasks lock").insert(id.clone(), task);
        self.set_state(&id, TaskState::Inspecting)?;
        Self::spawn_convert_runner(Arc::clone(self), id.clone());
        Ok(id)
    }

    /// Runner de conversión (C-20/RF-06): FFprobe valida el archivo (EC-30),
    /// FFmpeg ejecuta y el resultado se verifica (T2): exit 0 + existe + > 0 bytes.
    async fn run_convert_inner(manager: Arc<Self>, task_id: String) {
        let Some(task) = manager.snapshot(&task_id) else { return };
        let (Some(input), Some(format), Some(destination)) = (
            task.input_path.clone(),
            task.selected_format,
            task.destination.clone(),
        ) else {
            let _ = manager.apply_failure(&task_id, &TaskError::new(ErrorCode::Ec23, Some("conversión sin configuración".into())));
            return;
        };

        // Cancelación durante la inspección inicial (T4).
        if manager.current_signal(&task_id) == MotorSignal::Cancelar {
            let _ = manager.set_state(&task_id, TaskState::Cancelled);
            let snap = manager.snapshot(&task_id).expect("tarea existente");
            manager.record_history(&snap);
            return;
        }

        // FFprobe valida que el archivo sea multimedia real (CA-06.2).
        if let Err(e) = ffmpeg::probe_file(Path::new(&input)).await {
            let _ = manager.apply_failure(&task_id, &e);
            return;
        }

        if manager.set_state(&task_id, TaskState::Converting).is_err() {
            return; // cancelado a tiempo
        }
        manager.store_progress(
            &task_id,
            &ProgressInfo {
                percent: None, // sin progreso granular confiable: indeterminado (RF-04/REA F6.3)
                bytes_downloaded: None,
                bytes_total: None,
                speed_bps: None,
                eta_seconds: None,
                phase: Phase::Converting,
            },
        );

        // Validaciones previas: espacio en disco estimado (EC-24) — control básico.
        let input_size = std::fs::metadata(&input).map(|m| m.len()).unwrap_or(0);
        if let Some(parent) = Path::new(&destination).parent() {
            if parent.exists() {
                if let Some(bait) = Self::available_space(parent) {
                    if bait < input_size * 2 {
                        let _ = manager.apply_failure(&task_id, &TaskError::new(ErrorCode::Ec24, None));
                        return;
                    }
                }
            }
        }

        // Ejecución real + verificación final (T2/CA-06.1).
        match ffmpeg::convert_download(Path::new(&input), Path::new(&destination), format).await {
            Ok(()) => {
                let size = std::fs::metadata(&destination).map(|m| m.len()).unwrap_or(0);
                {
                    let mut tasks = manager.tasks.lock().expect("tasks lock");
                    if let Some(task) = tasks.get_mut(&task_id) {
                        task.progress = Some(ProgressInfo {
                            percent: Some(100.0),
                            bytes_downloaded: Some(size as i64),
                            bytes_total: Some(size as i64),
                            speed_bps: None,
                            eta_seconds: None,
                            phase: Phase::Converting,
                        });
                    }
                }
                let _ = manager.set_state(&task_id, TaskState::Completed);
                let snap = manager.snapshot(&task_id).expect("tarea existente");
                manager.record_history(&snap);
            }
            Err(e) => {
                let _ = manager.apply_failure(&task_id, &e);
            }
        }
    }

    /// Espacio libre del sistema de archivos (ec-24). Función con test en Unix.
    #[cfg(target_family = "unix")]
    fn available_space(dir: &Path) -> Option<u64> {
        let stat = std::process::Command::new("df")
            .args(["-k", &dir.display().to_string()])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&stat.stdout);
        // 3º campo del data row del `df -k` = espacio disponible (KiB).
        let row = text.lines().nth(1)?;
        let free_kib: u64 = row.split_whitespace().nth(3)?.parse().ok()?;
        Some(free_kib.saturating_mul(1024))
    }

    #[cfg(target_family = "windows")]
    fn available_space(_dir: &Path) -> Option<u64> {
        None
    }

    /// Vista de la cola: la más reciente arriba (REA C4).
    pub fn list(&self) -> Vec<DownloadTask> {
        let mut list: Vec<DownloadTask> = self
            .tasks
            .lock()
            .expect("tasks lock")
            .values()
            .cloned()
            .collect();
        list.sort_by(|a, b| b.seq.cmp(&a.seq));
        list
    }

    fn send_signal(&self, task_id: &str, signal: MotorSignal) {
        let signals = self.signals.lock().expect("signals lock");
        if let Some(slot) = signals.get(task_id) {
            *slot.lock().expect("signal slot") = signal;
        }
    }

    /// Lanza el runner asíncrono de una tarea (único creador de procesos).
    fn spawn_runner(manager: Arc<TaskManager>, task_id: String) {
        tauri::async_runtime::spawn(async move {
            TaskManager::run_download_inner(manager, &task_id).await;
        });
    }

    /// Lanza el runner de conversión local (único creador de procesos).
    fn spawn_convert_runner(manager: Arc<TaskManager>, task_id: String) {
        tauri::async_runtime::spawn(async move {
            TaskManager::run_convert_inner(manager, task_id).await;
        });
    }

    /// Runner de descarga (RF-04): proceso del motor con plantilla temporal
    /// estable por tarea, progreso verificado con bytes reales en disco (REA §4.4),
    /// señales de pausa/cancelación y conversión final con verificación real (T2).
    async fn run_download_inner(manager: Arc<Self>, task_id: &str) {
        let Some(task) = manager.snapshot(task_id) else { return };
        let (Some(quality_id), Some(format), Some(destination)) = (
            task.selected_quality_id.clone(),
            task.selected_format,
            task.destination.clone(),
        ) else {
            let e = TaskError::new(ErrorCode::Ec23, Some("tarea sin configuración de descarga".into()));
            let _ = manager.apply_failure(task_id, &e);
            return;
        };
        let Some(quality) = task.qualities.iter().find(|q| q.id == quality_id).cloned() else {
            let e = TaskError::new(ErrorCode::Ec23, Some("la calidad configurada ya no existe".into()));
            let _ = manager.apply_failure(task_id, &e);
            return;
        };

        let Some(yt_dlp_path) = tool_locator::which("yt-dlp").await else {
            let _ = manager.apply_failure(task_id, &TaskError::new(ErrorCode::Ec13, None));
            return;
        };

        // Plantilla temporal estable por tarea (REA §4.4): pausas/reintentos
        // continúan los mismos `.part`.
        let temp_dir = std::env::temp_dir().join("rea");
        let _ = std::fs::create_dir_all(&temp_dir);
        let template = temp_dir
            .join(format!("cg-yt-{}-%(id)s-%(format_id)s.%(ext)s", task.id))
            .display()
            .to_string();
        let temp_prefix = format!("cg-yt-{}", task.id);

        let args = build_download_arguments(
            &template,
            &quality.id,
            quality.audio_format_id.as_deref(),
            &task.source_url.clone().unwrap_or_default(),
        );

        let spawn_result = Command::new(&yt_dlp_path)
            .args(&args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn();

        let mut child = match spawn_result {
            Ok(child) => child,
            Err(e) => {
                let err = TaskError::new(ErrorCode::Ec23, Some(format!("no se pudo iniciar yt-dlp: {e}")));
                let _ = manager.apply_failure(task_id, &err);
                return;
            }
        };

        let stdout = child.stdout.take().expect("stdout piped");
        let stderr = child.stderr.take().expect("stderr piped");

        // Acumular stderr en una tarea independiente (límite 8 KiB — CAT-3).
        let stderr_buf = Arc::new(Mutex::new(String::new()));
        let stderr_worker_buf = Arc::clone(&stderr_buf);
        tauri::async_runtime::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut chunk = String::new();
            loop {
                chunk.clear();
                match reader.read_line(&mut chunk).await {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        let mut buf = stderr_worker_buf.lock().expect("stderr buf");
                        if buf.len() < 8192 {
                            buf.push_str(&chunk);
                        }
                    }
                }
            }
        });

        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        let mut final_temp: Option<String> = None;
        let mut estimates: Vec<i64> = Vec::new(); // una por archivo; la última es la activa
        let mut reported_error: Option<TaskError> = None;

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }

            // Señal de control entre líneas (D-05/T6).
            if manager.current_signal(task_id) == MotorSignal::Cancelar {
                let _ = child.kill().await;
                Self::cleanup_temps(&temp_dir, &temp_prefix);
                let _ = manager.set_state(task_id, TaskState::Cancelled);
                let task = manager.snapshot(task_id).expect("tarea existente");
                manager.record_history(&task);
                return;
            }

            match parse_motor_line(&line) {
                MotorLine::Destination(path) => {
                    final_temp = Some(path.clone());
                    estimates.push(0);
                }
                MotorLine::Merged(path) | MotorLine::AlreadyDownloaded(path) => {
                    final_temp = Some(path);
                }
                MotorLine::Downloaded { percent, size_estimate, speed_bps, eta_seconds } => {
                    // Calibración anti-salto (REA §4.4): solo estimaciones crecientes.
                    if let Some(slot) = estimates.last_mut() {
                        if size_estimate > *slot {
                            *slot = size_estimate;
                        }
                    }
                    let total_estimate: i64 = estimates.iter().sum();
                    let real_bytes = Self::sum_temp_bytes(&temp_dir, &temp_prefix);
                    let progress = if total_estimate > 0 && real_bytes > 0 {
                        ProgressInfo {
                            // Tope 99,9% hasta confirmar fin (REA C6/CA-04.2).
                            percent: Some(((real_bytes as f64 * 100.0) / total_estimate as f64).min(99.9)),
                            bytes_downloaded: Some(real_bytes),
                            bytes_total: Some(total_estimate),
                            speed_bps,
                            eta_seconds,
                            phase: Phase::Downloading,
                        }
                    } else {
                        ProgressInfo {
                            percent: Some(percent.min(99.9)),
                            bytes_downloaded: None,
                            bytes_total: Some(total_estimate.max(1)),
                            speed_bps,
                            eta_seconds,
                            phase: Phase::Downloading,
                        }
                    };
                    manager.store_progress(task_id, &progress);
                }
                MotorLine::Error(message) => {
                    let code = ytdlp::classify_ytdlp_error(&message);
                    reported_error = Some(TaskError::new(code, Some(message)));
                    let _ = child.kill().await;
                    break;
                }
                MotorLine::Otra => {}
            }
        }

        let status = child.wait().await;
        let stderr_text = stderr_buf.lock().expect("stderr buf").clone();
        let signal = manager.current_signal(task_id);

        if signal == MotorSignal::Cancelar {
            Self::cleanup_temps(&temp_dir, &temp_prefix);
            let _ = manager.set_state(task_id, TaskState::Cancelled);
            let task = manager.snapshot(task_id).expect("tarea existente");
            manager.record_history(&task);
            return;
        }
        if signal == MotorSignal::Pausar {
            // Pausa confirmada: el proceso murió por la señal; NO se limpian los
            // temporales (los `.part` continúan en el reanudar) — T6.
            let _ = manager.set_state(task_id, TaskState::Paused);
            return;
        }

        if let Some(err) = reported_error {
            Self::cleanup_temps(&temp_dir, &temp_prefix);
            let _ = manager.apply_failure(task_id, &err);
            return;
        }

        match status {
            Ok(exit) if exit.success() => {
                // Verificación de archivo final real (T2/REA F7.5): existe y no vacío.
                let final_path = final_temp
                    .clone()
                    .filter(|p| std::path::Path::new(p).metadata().is_ok_and(|m| m.len() > 0))
                    .or_else(|| Self::newest_temp_file(&temp_dir, &temp_prefix))
                    .filter(|p| {
                        std::path::Path::new(p).metadata().map(|m| m.len() > 0).unwrap_or(false)
                    });

                let Some(final_path) = final_path else {
                    let _ = manager.apply_failure(task_id, &TaskError::new(ErrorCode::Ec34, None));
                    Self::cleanup_temps(&temp_dir, &temp_prefix);
                    return;
                };

                // Fase de conversión (converting → completed, REA F6.3).
                if manager.set_state(task_id, TaskState::Converting).is_err() {
                    return; // ya terminó de otra forma (cancelación tardía)
                }
                manager.store_progress(
                    task_id,
                    &ProgressInfo {
                        // Conversión sin progreso granular: indeterminado honesto (RF-04).
                        percent: None,
                        bytes_downloaded: None,
                        bytes_total: None,
                        speed_bps: None,
                        eta_seconds: None,
                        phase: Phase::Converting,
                    },
                );

                if manager.current_signal(task_id) == MotorSignal::Cancelar {
                    Self::cleanup_temps(&temp_dir, &temp_prefix);
                    let _ = manager.set_state(task_id, TaskState::Cancelled);
                    let task = manager.snapshot(task_id).expect("tarea existente");
                    manager.record_history(&task);
                    return;
                }

                match ffmpeg::convert_download(Path::new(&final_path), Path::new(&destination), format)
                    .await
                {
                    Ok(()) => {
                        Self::cleanup_temps(&temp_dir, &temp_prefix);
                        let size = std::fs::metadata(&destination).map(|m| m.len()).unwrap_or(0);
                        {
                            let mut tasks = manager.tasks.lock().expect("tasks lock");
                            if let Some(task) = tasks.get_mut(task_id) {
                                task.progress = Some(ProgressInfo {
                                    percent: Some(100.0),
                                    bytes_downloaded: Some(size as i64),
                                    bytes_total: Some(size as i64),
                                    speed_bps: None,
                                    eta_seconds: None,
                                    phase: Phase::Converting,
                                });
                            }
                        }
                        let _ = manager.set_state(task_id, TaskState::Completed);
                        let task = manager.snapshot(task_id).expect("tarea existente");
                        manager.record_history(&task);
                    }
                    Err(e) => {
                        Self::cleanup_temps(&temp_dir, &temp_prefix);
                        let _ = manager.apply_failure(task_id, &e);
                    }
                }
            }
            _ => {
                // Terminó con error sin mensaje clasificable en línea.
                let code = ytdlp::classify_ytdlp_error(&stderr_text);
                let err = TaskError::new(code, Some(stderr_text.lines().last().unwrap_or("").to_string()));
                Self::cleanup_temps(&temp_dir, &temp_prefix);
                let _ = manager.apply_failure(task_id, &err);
            }
        }
    }

    fn current_signal(&self, task_id: &str) -> MotorSignal {
        let signals = self.signals.lock().expect("signals lock");
        match signals.get(task_id) {
            Some(slot) => *slot.lock().expect("signal slot"),
            None => MotorSignal::Ninguna,
        }
    }

    /// Marca la tarea como `failed` con el error dado.
    fn apply_failure(&self, task_id: &str, err: &TaskError) -> Result<(), TaskError> {
        {
            let mut tasks = self.tasks.lock().expect("tasks lock");
            if let Some(task) = tasks.get_mut(task_id) {
                task.error = Some(err.clone());
            }
        }
        let from = self.snapshot(task_id).map(|t| t.state).unwrap_or(TaskState::Failed);
        // Transición al fallido; si ya está terminal (p. ej. cancelada a tiempo),
        // se acepta sin error (la cancelación ganó, comportamiento válido).
        if from.can_transition_to(TaskState::Failed) {
            self.set_state(task_id, TaskState::Failed)?;
        }
        let task = self.snapshot(task_id).expect("tarea existente");
        self.record_history(&task);
        Ok(())
    }

    /// Guarda y emite el progreso (T5).
    fn store_progress(&self, task_id: &str, progress: &ProgressInfo) {
        let event = {
            let mut tasks = self.tasks.lock().expect("tasks lock");
            let task = match tasks.get_mut(task_id) {
                Some(task) => task,
                None => return,
            };
            task.progress = Some(*progress);
            ProgressEvent { task_id: task_id.to_string(), progress: *progress }
        };
        let _ = self.app.emit("task://progress", event);
    }

    /// Bytes reales en disco de los temporales de la tarea (progreso verificado, REA §4.4).
    fn sum_temp_bytes(temp_dir: &Path, prefix: &str) -> i64 {
        let mut total = 0i64;
        if let Ok(entries) = std::fs::read_dir(temp_dir) {
            for entry in entries.flatten() {
                if entry.file_name().to_string_lossy().starts_with(prefix) {
                    total += entry.metadata().map(|m| m.len() as i64).unwrap_or(0);
                }
            }
        }
        total
    }

    /// Temporal `.part` activo (fallback del archivo final tras reanudaciones).
    fn newest_temp_file(temp_dir: &Path, prefix: &str) -> Option<String> {
        let mut best: Option<(std::time::SystemTime, String)> = None;
        if let Ok(entries) = std::fs::read_dir(temp_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(prefix) && name.ends_with(".part") {
                    let modified = entry
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    match &best {
                        Some((the_time, _)) if *the_time >= modified => {}
                        _ => best = Some((modified, entry.path().display().to_string())),
                    }
                }
            }
        }
        best.map(|(_, path)| path)
    }

    /// Elimina los temporales de la tarea (patrón REA: limpiar al terminar/cancelar;
    /// nunca tras pausa — los `.part` continúan).
    fn cleanup_temps(temp_dir: &Path, prefix: &str) {
        let Ok(entries) = std::fs::read_dir(temp_dir) else { return };
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().starts_with(prefix) {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    /// Registra el resultado en el historial (D-13/RF-07). Un fallo de escritura
    /// no altera el estado de la tarea (EC-40 queda en log, CAT-3).
    fn record_history(&self, task: &DownloadTask) {
        let status = match task.state {
            TaskState::Completed => "completed",
            TaskState::Failed => "failed",
            TaskState::Cancelled => "cancelled",
            _ => return,
        };
        let entry = HistoryEntry {
            id: task.id.clone(),
            kind: match task.kind {
                TaskKind::Download => "download",
                TaskKind::Convert => "convert",
            }
            .to_string(),
            title: task.title.clone().unwrap_or_else(|| "[sin título]".to_string()),
            portal: task.portal.clone(),
            source_url: task.source_url.as_deref().map(redact_url),
            format: task.selected_format.map(|f| match f {
                OutputFormat::Mp4 => "mp4".to_string(),
                OutputFormat::Mp3 => "mp3".to_string(),
                OutputFormat::Original => "original".to_string(),
            }),
            quality: task
                .selected_quality_id
                .as_ref()
                .and_then(|qid| task.qualities.iter().find(|q| &q.id == qid))
                .map(|q| q.label.clone()),
            destination_path: task.destination.clone().unwrap_or_default(),
            final_size_bytes: task.progress.and_then(|p| p.bytes_downloaded),
            status: status.to_string(),
            error_code: task.error.as_ref().map(error_code_string),
            started_at: task.created_at.clone(),
            finished_at: Self::now_iso(),
        };
        let store = self.app.state::<Store>();
        if let Err(e) = store.history_insert(&entry) {
            eprintln!("historial (EC-40): {e}");
        }
    }
}

/// Construye los argumentos exactos de descarga (función pura — verificados por test).
pub fn build_download_arguments(
    template: &str,
    video_id: &str,
    audio_id: Option<&str>,
    url: &str,
) -> Vec<String> {
    let fmt = match audio_id {
        Some(audio) => format!("{video_id}+{audio}"),
        None => video_id.to_string(),
    };
    vec![
        "--newline".to_string(),
        "--no-playlist".to_string(),
        "--no-mtime".to_string(),
        "-o".to_string(),
        template.to_string(),
        "-f".to_string(),
        fmt,
        url.to_string(),
    ]
}

fn error_code_string(e: &TaskError) -> String {
    match e.code {
        ErrorCode::Ec01 => "EC-01",
        ErrorCode::Ec02 => "EC-02",
        ErrorCode::Ec03 => "EC-03",
        ErrorCode::Ec10 => "EC-10",
        ErrorCode::Ec11 => "EC-11",
        ErrorCode::Ec12 => "EC-12",
        ErrorCode::Ec13 => "EC-13",
        ErrorCode::Ec20 => "EC-20",
        ErrorCode::Ec21 => "EC-21",
        ErrorCode::Ec22 => "EC-22",
        ErrorCode::Ec23 => "EC-23",
        ErrorCode::Ec24 => "EC-24",
        ErrorCode::Ec25 => "EC-25",
        ErrorCode::Ec26 => "EC-26",
        ErrorCode::Ec30 => "EC-30",
        ErrorCode::Ec31 => "EC-31",
        ErrorCode::Ec32 => "EC-32",
        ErrorCode::Ec33 => "EC-33",
        ErrorCode::Ec34 => "EC-34",
        ErrorCode::Ec35 => "EC-35",
        ErrorCode::Ec40 => "EC-40",
        ErrorCode::Ec41 => "EC-41",
        ErrorCode::Ec42 => "EC-42",
        ErrorCode::Ec43 => "EC-43",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// REA §4/RF-04: los argumentos de descarga son exactos y de lista (RNF-01).
    #[test]
    fn download_arguments_are_exact() {
        let args = build_download_arguments(
            "/tmp/cg-yt-x-%(id)s-%(format_id)s.%(ext)s",
            "137",
            Some("140"),
            "https://example.com/v",
        );
        assert_eq!(
            args,
            vec![
                "--newline",
                "--no-playlist",
                "--no-mtime",
                "-o",
                "/tmp/cg-yt-x-%(id)s-%(format_id)s.%(ext)s",
                "-f",
                "137+140",
                "https://example.com/v",
            ]
        );
        let args = build_download_arguments(
            "/tmp/cg-yt-x-%(id)s-%(format_id)s.%(ext)s",
            "18",
            None,
            "https://example.com/v",
        );
        assert_eq!(args[6], "18");
    }

    #[test]
    fn temp_bytes_and_cleanup_respect_task_boundaries() {
        let dummy = std::env::temp_dir().join("rea-taskcalc-test");
        std::fs::create_dir_all(&dummy).unwrap();
        std::fs::write(dummy.join("cg-yt-T1-a.part"), vec![0; 8]).unwrap();
        std::fs::write(dummy.join("cg-yt-T1-b.mp4"), vec![0; 4]).unwrap();
        std::fs::write(dummy.join("otro-archivo"), vec![0; 2]).unwrap();
        assert_eq!(TaskManager::sum_temp_bytes(&dummy, "cg-yt-T1-"), 12);
        TaskManager::cleanup_temps(&dummy, "cg-yt-T1-");
        assert_eq!(TaskManager::sum_temp_bytes(&dummy, "cg-yt-T1-"), 0);
        assert!(dummy.join("otro-archivo").exists()); // no borra lo ajeno
        let _ = std::fs::remove_file(dummy.join("otro-archivo"));
        let _ = std::fs::remove_dir(&dummy);
    }
}
