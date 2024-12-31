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
use tauri::Listener;
use tauri_plugin_clipboard_manager::ClipboardExt;

use rdev::{simulate, EventType, Key, SimulateError};
use std::{thread, time::Duration};

fn simulate_paste() {
    // Helper function to send a key event
    fn send(event_type: EventType) {
        if let Err(SimulateError) = simulate(&event_type) {
            eprintln!("Failed to send {:?}", event_type);
        }
        // Introduce a small delay between events to ensure they are registered
        thread::sleep(Duration::from_millis(20));
    }

    #[cfg(target_os = "macos")]
    {
        // macOS uses the Command key for shortcuts
        send(EventType::KeyPress(Key::MetaLeft));
        send(EventType::KeyPress(Key::KeyV));
        send(EventType::KeyRelease(Key::KeyV));
        send(EventType::KeyRelease(Key::MetaLeft));
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Windows and Linux typically use the Control key for shortcuts
        send(EventType::KeyPress(Key::ControlLeft));
        send(EventType::KeyPress(Key::KeyV));
        send(EventType::KeyRelease(Key::KeyV));
        send(EventType::KeyRelease(Key::ControlLeft));
    }
}

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

            let app_handle = app.handle().clone();
            app.listen("refined-transcription-complete", move |event| {
                log::info!("Refined transcription completed: {:?}", event.payload());

                // Trim quotes from the payload if present
                let payload = event.payload();
                let trimmed_payload = payload.trim_matches('"');

                // Get the trimmed payload and write to clipboard
                if let Err(e) = app_handle.clipboard().write_text(trimmed_payload) {
                    log::error!("Error writing to clipboard: {:?}", e);
                    return;
                }

                // paste the text
                simulate_paste();
            });

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
