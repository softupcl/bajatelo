use std::sync::Arc;

use tauri::{AppHandle, Manager, State};

use crate::app::task_manager::TaskManager;
use crate::models::errors::TaskError;
use crate::services::engine_updater;

/// Actualiza el motor de descarga (yt-dlp) desde el release oficial, con
/// verificación SHA-256 contra el manifest oficial, e instala en
/// `<datos>/engines` (prioridad sobre el PATH — D-09/C). Fallo → EC-20/23/25.
#[tauri::command]
pub async fn update_ytdlp(
    app: AppHandle,
    _manager: State<'_, Arc<TaskManager>>,
) -> Result<engine_updater::EngineUpdateOutcome, TaskError> {
    let data_dir = app.path().app_data_dir().expect("app data dir");
    engine_updater::update_into(&data_dir.join("engines")).await
}
