use tauri::State;

use crate::models::errors::TaskError;
use crate::models::history::HistoryEntry;
use crate::persistence::Store;

fn db_err(e: rusqlite::Error) -> TaskError {
    TaskError::new(crate::models::errors::ErrorCode::Ec40, Some(e.to_string()))
}

#[tauri::command]
pub fn history_list(store: State<'_, Store>) -> Result<Vec<HistoryEntry>, TaskError> {
    store.history_list().map_err(db_err)
}

#[tauri::command]
pub fn history_remove(id: String, store: State<'_, Store>) -> Result<bool, TaskError> {
    store.history_remove(&id).map_err(db_err)
}

#[tauri::command]
pub fn history_clear(store: State<'_, Store>) -> Result<usize, TaskError> {
    store.history_clear().map_err(db_err)
}
