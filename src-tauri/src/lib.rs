#[cfg(debug_assertions)]
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::env;
use tauri_plugin_clipboard_manager::ClipboardExt;
use std::sync::Mutex;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender};
use hound::{WavSpec, WavWriter};
use std::fs::File;
use std::io::BufWriter;
use chrono::Local;
use dirs;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: String,
    email: String,
    family_name: String,
    given_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RecordingState {
    Stopped,
    Recording,
    Paused,
}

struct AppState {
    user: Mutex<Option<User>>,
    recording_state: Mutex<RecordingState>,
    is_recording: Arc<AtomicBool>,
    audio_writer: Arc<Mutex<Option<WavWriter<BufWriter<File>>>>>,
    recording_sender: Arc<Mutex<Option<Sender<RecordingMessage>>>>,
}

// Add message enum for recording control
#[derive(Debug)]
enum RecordingMessage {
    Stop,
    Pause,
    Resume,
}

// Helper function to get WAV spec from CPAL config
fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> WavSpec {
    WavSpec {
        channels: config.channels() as u16,
        sample_rate: config.sample_rate().0,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    }
}

// Helper function to write input data
fn write_input_data<T>(input: &[T], writer: &Arc<Mutex<Option<WavWriter<BufWriter<File>>>>>)
where
    T: cpal::Sample + hound::Sample,
{
    if let Ok(mut guard) = writer.lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                writer.write_sample(sample).unwrap();
            }
        }
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn set_user(state: tauri::State<'_, AppState>, user_data: User) -> Result<(), String> {
    let mut user = state.user.lock().map_err(|e| e.to_string())?;
    *user = Some(user_data);
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct RefinedMessage {
    suggested_message_rewrite: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    id: i32,
    name: String,
    done: bool,
    #[serde(rename = "userId")]
    user_id: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
}

#[tauri::command]
async fn refine_message(app: tauri::AppHandle, msg: String) -> Result<RefinedMessage, String> {
    let api_key = env::var("GROQ_API_KEY").map_err(|e| {
        log::error!("Failed to get GROQ_API_KEY: {}", e);
        "GROQ_API_KEY not found in environment".to_string()
    })?;

    // Construct the request body
    let body = json!({
        "messages": [
            {
                "role": "system",
                "content": r###"You are a message refinement API. Your goal is to suggest a more articulate, polished version of the original message, maintaining a professional tone without sounding overly formal. If the message has a casual flair, keep some of its original character so it still feels like the author.

                Your response should be in JSON format:
                {
                "suggested_message_rewrite": "string (a refined version of the original message, with improved clarity and a tone that balances professionalism with the original casual feel)"
                }

                Guidelines:
                1. Retain the message's intent and main points, improving clarity and flow.
                2. Preserve any casual flair to maintain the author's voice, but ensure the tone is articulate and approachable.
                3. Respond in valid JSON format."###
            },
            {
                "role": "user",
                "content": format!("help me analyze the following:\n\n\"\"\"\n{}\n\"\"\"", msg)
            }
        ],
        "model": "llama-3.2-90b-text-preview",
        "temperature": 1,
        "max_tokens": 1024,
        "top_p": 1,
        "stream": false,
        "response_format": {
            "type": "json_object"
        },
        "stop": null
    });

    // Create the client
    let client = reqwest::Client::new();

    // Make the request
    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to send request to Groq API: {}", e);
            e.to_string()
        })?;

    // Parse the response as JSON
    let json_response = response.json::<Value>().await.map_err(|e| {
        log::error!("Failed to parse Groq API response as JSON: {}", e);
        e.to_string()
    })?;

    // Extract the content string from the first choice
    let content = json_response["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| {
            let err = "Failed to get message content from response";
            log::error!("{}", err);
            err.to_string()
        })?;

    // Parse the content string as JSON to get the suggested_message_rewrite
    let content_json: RefinedMessage = serde_json::from_str(content).map_err(|e| {
        log::error!("Failed to parse message content as RefinedMessage: {}", e);
        e.to_string()
    })?;

    // write the refined message to the clipboard
    if let Err(e) = app.clipboard().write_text(content_json.suggested_message_rewrite.clone()) {
        log::error!("Failed to write to clipboard: {}", e);
    }

    // Return the entire RefinedMessage struct
    Ok(content_json)
}

#[tauri::command]
async fn fetch_tasks(token: String) -> Result<Value, String> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://jeff-ai-cf-be.mrboutte21.workers.dev/api/tasks")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to fetch tasks: {}", e);
            e.to_string()
        })?;

    // Get the raw JSON value
    let json_value = response.json::<Value>().await.map_err(|e| {
        log::error!("Failed to parse response as JSON: {}", e);
        e.to_string()
    })?;

    log::info!("Raw API response: {:?}", json_value);
    Ok(json_value)
}

#[tauri::command]
async fn capture_user(token: String, state: tauri::State<'_, AppState>) -> Result<Value, String> {
    let client = reqwest::Client::new();
    // Get user_id and drop the lock immediately
    let user_data = {
        let current_user = state.user.lock().map_err(|e| e.to_string())?;
        current_user.as_ref()
            .map(|u| (u.id.clone(), u.email.clone(), format!("{} {}", u.given_name, u.family_name)))
            .ok_or_else(|| "No user found in state".to_string())?
    };

    let response = client
        .post("https://jeff-ai-cf-be.mrboutte21.workers.dev/api/capture")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&json!({
            "authUserId": user_data.0,
            "email": user_data.1,
            "displayName": user_data.2
        }))
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to create capture: {}", e);
            e.to_string()
        })?;

    // Check status code
    let status = response.status();
    let json_value = response.json::<Value>().await.map_err(|e| {
        log::error!("Failed to parse response as JSON: {}", e);
        e.to_string()
    })?;

    log::info!("user in state: {:?}", state.user);
    log::info!("status: {:?}", status);

    log::info!("Raw API response: {:?}", json_value);
    Ok(json_value)

}

#[tauri::command]
async fn start_recording(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut recording_state = state.recording_state.lock().map_err(|e| e.to_string())?;

    match *recording_state {
        RecordingState::Stopped => {
            *recording_state = RecordingState::Recording;

            // Rest of your existing recording logic...
            log::info!("Starting recording");

            // Get desktop directory and create output path
            let desktop_dir = dirs::desktop_dir()
                .ok_or_else(|| "Could not find desktop directory".to_string())?;

            // Create a timestamp for the filename
            let timestamp = Local::now().format("%Y%m%d_%H%M%S");
            let output_path = desktop_dir.join(format!("recording_{}.wav", timestamp));

            log::info!("Recording to: {:?}", output_path);

            let host = cpal::default_host();
            let device = host.default_input_device()
                .ok_or_else(|| "No input device available".to_string())?;

            let config = device.default_input_config()
                .map_err(|e| e.to_string())?;

            // Create WAV writer
            let spec = wav_spec_from_config(&config);
            let writer = WavWriter::create(&output_path, spec)
                .map_err(|e| e.to_string())?;

            let writer = Arc::new(Mutex::new(Some(writer)));
            let writer_clone = Arc::clone(&writer);

            // Set up communication channel
            let (sender, receiver) = channel();
            *state.recording_sender.lock().map_err(|e| e.to_string())? = Some(sender);

            // Store writer in state
            *state.audio_writer.lock().map_err(|e| e.to_string())? = Some(writer.lock().unwrap().take().unwrap());

            // Set recording flag
            state.is_recording.store(true, Ordering::SeqCst);
            let recording_flag = Arc::clone(&state.is_recording);

            // Spawn recording thread
            std::thread::spawn(move || {
                let stream = device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _| {
                        if recording_flag.load(Ordering::SeqCst) {
                            write_input_data(data, &writer_clone);
                        }
                    },
                    move |err| {
                        log::error!("Error in audio stream: {}", err);
                    },
                    None
                ).unwrap();

                stream.play().unwrap();

                // Handle control messages
                while let Ok(msg) = receiver.recv() {
                    match msg {
                        RecordingMessage::Stop => break,
                        RecordingMessage::Pause => { let _ = stream.pause(); },
                        RecordingMessage::Resume => { let _ = stream.play(); },
                    }
                }
            });

            Ok(())
        }
        _ => Err("Recording already in progress".to_string())
    }
}

#[tauri::command]
async fn stop_recording(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut recording_state = state.recording_state.lock().map_err(|e| e.to_string())?;

    match *recording_state {
        RecordingState::Recording | RecordingState::Paused => {
            *recording_state = RecordingState::Stopped;

            // Rest of your existing stop logic...
            log::info!("Stopping recording");
            state.is_recording.store(false, Ordering::SeqCst);

            // Send stop message
            if let Some(sender) = state.recording_sender.lock().map_err(|e| e.to_string())?.as_ref() {
                sender.send(RecordingMessage::Stop).map_err(|e| e.to_string())?;
            }

            // Finalize WAV file
            if let Some(writer) = state.audio_writer.lock().map_err(|e| e.to_string())?.take() {
                writer.finalize().map_err(|e| e.to_string())?;
            }

            Ok(())
        }
        RecordingState::Stopped => Err("Recording not started".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    dotenv().ok();

    let app_state = AppState {
        user: Mutex::new(None),
        recording_state: Mutex::new(RecordingState::Stopped),
        is_recording: Arc::new(AtomicBool::new(false)),
        audio_writer: Arc::new(Mutex::new(None)),
        recording_sender: Arc::new(Mutex::new(None)),
    };

    tauri::Builder::default()
        .manage(app_state)
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("app.log".to_string()),
                    },
                ))
                .build()
        )
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            refine_message,
            fetch_tasks,
            capture_user,
            set_user,
            start_recording,
            stop_recording
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
