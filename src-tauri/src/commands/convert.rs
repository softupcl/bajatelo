use std::sync::Arc;

use tauri::State;

use crate::app::task_manager::TaskManager;
use crate::models::errors::TaskError;
use crate::models::media::LocalMediaInfo;
use crate::models::prefs::ConflictPolicy;
use crate::models::task::OutputFormat;

/// Inspección de un archivo local con FFprobe (RF-06) — ficha de la vista Convertir.
#[tauri::command]
pub async fn probe_local_file(
    path: String,
    _manager: State<'_, Arc<TaskManager>>,
) -> Result<LocalMediaInfo, TaskError> {
    crate::services::ffmpeg::probe_file(std::path::Path::new(&path)).await
}

/// Inicia la conversión de un archivo local (C-20/RF-06): crea la tarea en
/// la cola y devuelve su id; los estados llegan por `task://state`.
#[tauri::command]
pub async fn start_conversion(
    input: String,
    format: OutputFormat,
    destination: String,
    policy: ConflictPolicy,
    audio_bitrate: Option<String>,
    manager: State<'_, Arc<TaskManager>>,
) -> Result<String, TaskError> {
    manager
        .start_conversion(&input, format, &destination, policy, audio_bitrate.as_deref())
        .await
}

/// Abre un archivo con el visor del SO o lo revela en el Finder/explorador
/// (req. 10 del enunciado de conversión; fallo = EC-43 del catálogo).
/// Solo actúa sobre rutas existentes y sin argumentos al shell (RNF-01).
#[tauri::command]
pub async fn open_item(path: String, reveal: bool) -> Result<(), TaskError> {
    use crate::models::errors::ErrorCode;
    let p = std::path::PathBuf::from(&path);
    if !p.is_file() && !p.is_dir() {
        return Err(TaskError::new(
            ErrorCode::Ec43,
            Some("la ruta no existe".into()),
        ));
    }
    let outcome = if reveal {
        tauri_plugin_opener::reveal_item_in_dir(&p)
    } else {
        tauri_plugin_opener::open_path(&p, None::<&str>)
    };
    outcome.map_err(|e| TaskError::new(ErrorCode::Ec43, Some(e.to_string())))
}
