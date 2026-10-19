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
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Persistencia SQLite (D-07): se abre al arrancar y se comparte vía State.
            let data_dir = app.path().app_data_dir().expect("app data dir");
            let (store, outcome) =
                persistence::Store::open(&data_dir.join("bajatelo.db")).expect("open store");
            if outcome != persistence::OpenOutcome::Clean {
                // EC-41: la base se restauró o recreó. El aviso visible en la UI
                // llegará con el canal de eventos; por ahora queda en el log local.
                eprintln!("store: apertura con recuperación ({outcome:?})");
            }
            app.manage(store);

            // Gestor de tareas (D-12): dueño de la cola, procesos y eventos.
            let task_manager = app::task_manager::TaskManager::new(app.handle().clone());
            app.manage(task_manager);

            // Motores (D-09/C): el yt-dlp actualizado desde la app vive en
            // <datos>/engines y tiene prioridad sobre el PATH.
            services::tool_locator::register_engine_dir(data_dir.join("engines"));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::diagnostics::get_diagnostics,
            commands::tasks::analyze_url,
            commands::tasks::start_download,
            commands::tasks::pause_task,
            commands::tasks::resume_task,
            commands::tasks::cancel_task,
            commands::tasks::retry_task,
            commands::tasks::get_tasks,
            commands::convert::probe_local_file,
            commands::convert::start_conversion,
            commands::convert::open_item,
            commands::engine::update_ytdlp,
            commands::prefs::get_preferences,
            commands::prefs::save_preferences,
            commands::history::history_list,
            commands::history::history_remove,
            commands::history::history_clear,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
