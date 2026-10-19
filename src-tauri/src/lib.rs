mod app;
mod commands;
mod models;
mod persistence;
mod services;

use tauri::Manager;

/// Punto de entrada de la app (AGENTS.md §6: la lógica vive fuera de la UI).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Persistencia SQLite (D-07): se abre al arrancar y se comparte vía State.
            let data_dir = app.path().app_data_dir().expect("app data dir");
            let (store, outcome) =
                persistence::Store::open(&data_dir.join("rea.db")).expect("open store");
            if outcome != persistence::OpenOutcome::Clean {
                // EC-41: la base se restauró o recreó. El aviso visible en la UI
                // llegará con el canal de eventos; por ahora queda en el log local.
                eprintln!("store: apertura con recuperación ({outcome:?})");
            }
            app.manage(store);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::diagnostics::get_diagnostics,
            commands::tasks::analyze_url,
            commands::prefs::get_preferences,
            commands::prefs::save_preferences,
            commands::history::history_list,
            commands::history::history_remove,
            commands::history::history_clear,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
