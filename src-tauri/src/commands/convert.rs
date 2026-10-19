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
    manager: State<'_, Arc<TaskManager>>,
) -> Result<String, TaskError> {
    manager.start_conversion(&input, format, &destination, policy).await
}
