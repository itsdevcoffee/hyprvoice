// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod daemon_client;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_daemon_status,
            commands::start_recording,
            commands::stop_recording,
            commands::get_transcription_history,
            commands::download_model,
            commands::get_system_info,
            commands::get_config,
            commands::save_config,
            commands::restart_daemon,
            commands::validate_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
