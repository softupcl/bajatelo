mod commands;
mod models;
mod services;

/// Punto de entrada de la app (AGENTS.md §6: la lógica vive fuera de la UI).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![commands::diagnostics::get_diagnostics])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
