use std::sync::Arc;

use tauri::State;

use crate::app::task_manager::AnalysisOutcome;
use crate::app::task_manager::TaskManager;
use crate::app::validation;
use crate::models::errors::TaskError;
use crate::models::prefs::ConflictPolicy;
use crate::models::task::{DownloadTask, OutputFormat};

/// Análisis real de una URL (RF-01/RF-02): valida y normaliza (V-01)
/// y crea la tarea en la cola (`inspecting → ready|failed`, D-12).
#[tauri::command]
pub async fn analyze_url(
    url: String,
    manager: State<'_, Arc<TaskManager>>,
) -> Result<AnalysisOutcome, TaskError> {
    let normalized = validation::validate_and_normalize_url(&url)?;
    Ok(manager.run_analysis(&normalized).await)
}

/// Inicia la descarga (RF-03/RF-04/D-05): `ready → downloading` + runner.
#[tauri::command]
pub async fn start_download(
    task_id: String,
    quality_id: String,
    format: OutputFormat,
    destination: String,
    policy: ConflictPolicy,
    audio_bitrate: Option<String>,
    manager: State<'_, Arc<TaskManager>>,
) -> Result<String, TaskError> {
    manager
        .start_download(&task_id, &quality_id, format, &destination, policy, audio_bitrate.as_deref())
        .await
}

/// Pausa una descarga activa (D-05/T6).
#[tauri::command]
pub fn pause_task(
    task_id: String,
    manager: State<'_, Arc<TaskManager>>,
) -> Result<(), TaskError> {
    manager.pause_task(&task_id)
}

/// Reanuda una tarea pausada (D-05/T6).
#[tauri::command]
pub async fn resume_task(
    task_id: String,
    manager: State<'_, Arc<TaskManager>>,
) -> Result<(), TaskError> {
    manager.resume_task(&task_id).await
}

/// Cancela una tarea (T4).
#[tauri::command]
pub fn cancel_task(
    task_id: String,
    manager: State<'_, Arc<TaskManager>>,
) -> Result<(), TaskError> {
    manager.cancel_task(&task_id)
}

/// Reintenta una tarea `failed`/`cancelled` (T3).
#[tauri::command]
pub async fn retry_task(
    task_id: String,
    manager: State<'_, Arc<TaskManager>>,
) -> Result<(), TaskError> {
    manager.retry_task(&task_id).await
}

/// Snapshot de la cola (la UI lo usa para refrescarse al montar).
#[tauri::command]
pub fn get_tasks(manager: State<'_, Arc<TaskManager>>) -> Vec<DownloadTask> {
    manager.list()
}
