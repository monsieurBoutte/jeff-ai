#[cfg(debug_assertions)]
use dotenv::dotenv;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

mod audio;
mod handlers;
mod models;
mod state;

use handlers::*;
use state::{AppState, RecordingState};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    // Load environment variables from .env file
    dotenv().ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let app_state = AppState {
                user: Mutex::new(None),
                existing_user: Mutex::new(None),
                temp_file: Arc::new(Mutex::new(None)),
                recording_state: Mutex::new(RecordingState::Stopped),
                is_recording: Arc::new(AtomicBool::new(false)),
                audio_writer: Mutex::new(None),
                recording_sender: Arc::new(Mutex::new(None)),
                app_handle: app.handle().clone(),
            };
            app.manage(app_state);
            Ok(())
        })
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("app.log".to_string()),
                    },
                ))
                .build(),
        )
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            refine_text,
            convert_to_markdown,
            fetch_tasks,
            capture_user,
            set_user,
            start_recording,
            stop_recording
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
