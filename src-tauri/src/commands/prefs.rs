use tauri::State;

use crate::models::errors::TaskError;
use crate::models::prefs::Preferences;
use crate::persistence::Store;

fn db_err(e: rusqlite::Error) -> TaskError {
    TaskError::new(crate::models::errors::ErrorCode::Ec40, Some(e.to_string()))
}

#[tauri::command]
pub fn get_preferences(store: State<'_, Store>) -> Result<Preferences, TaskError> {
    store.get_preferences().map_err(db_err)
}

#[tauri::command]
pub fn save_preferences(
    prefs: Preferences,
    store: State<'_, Store>,
) -> Result<Preferences, TaskError> {
    store.save_preferences(&prefs).map_err(db_err)?;
    Ok(prefs)
}
