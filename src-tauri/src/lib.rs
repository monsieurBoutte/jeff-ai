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
use deepgram::{
    Deepgram,
    common::options::{Encoding, Options},
    common::stream_response::StreamResponse,
};
use tokio::sync::mpsc;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: String,
    email: String,
    family_name: String,
    given_name: String,
}

struct AppState {
    user: Mutex<Option<User>>,
    is_recording: Arc<AtomicBool>,
    audio_sender: Mutex<Option<mpsc::Sender<Vec<f32>>>>,
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
    log::info!("Starting recording");
    let host = cpal::default_host();

    // Get the default input device
    let device = host.default_input_device()
        .ok_or_else(|| "No input device available".to_string())?;

    log::info!("Using default input device: {}", device.name().map_err(|e| e.to_string())?);

    // Get the default config for the input device
    let config = device.default_input_config()
        .map_err(|e| e.to_string())?;

    log::info!("Default input config: {:?}", config);

    // Set up the recording flag
    state.is_recording.store(true, Ordering::SeqCst);
    let recording_flag = Arc::clone(&state.is_recording);

    // Create channel for audio data
    let (tx, mut rx) = mpsc::channel::<Vec<f32>>(1024);
    *state.audio_sender.lock().map_err(|e| e.to_string())? = Some(tx.clone());

    // Set up Deepgram client
    // let api_key = env::var("DEEPGRAM_API_KEY").map_err(|e| {
    //     log::error!("Failed to get DEEPGRAM_API_KEY: {}", e);
    //     e.to_string()
    // })?;

    let dg = Deepgram::new("57851d87f568a11263a1a48e46f0b4a358ac44b2").map_err(|e| e.to_string())?;

    // Clone for tokio task
    let recording_flag_task = Arc::clone(&recording_flag);

    // Spawn transcription task
    tokio::spawn(async move {
        let options = Options::builder()
            .smart_format(true)
            .build();

        let mut stream = match dg.transcription()
            .stream_request_with_options(options)
            .encoding(Encoding::Linear16)
            .sample_rate(44100)
            .channels(1)
            .handle()
            .await {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Failed to build stream: {}", e);
                    return;
                }
            };

        while let Some(audio_chunk) = rx.recv().await {
            if !recording_flag_task.load(Ordering::SeqCst) {
                break;
            }

            let audio_data: Vec<i16> = audio_chunk
                .iter()
                .map(|&x| (x * 32767.0) as i16)
                .collect();

            // Convert i16 samples to bytes
            let audio_bytes: Vec<u8> = audio_data.iter()
                .flat_map(|&sample| sample.to_le_bytes())
                .collect();

            if let Err(e) = stream.send_data(audio_bytes).await {
                log::error!("Error sending audio: {}", e);
                break;
            }

            while let Some(result) = stream.receive().await {
                match result {
                    Ok(response) => {
                        match response {
                            StreamResponse::SpeechStartedResponse { type_field, channel, timestamp } => {
                                log::info!("Speech started: {:?}", timestamp);
                            }
                            StreamResponse::TranscriptResponse { channel, .. } => {
                                if let Some(alt) = channel.alternatives.first() {
                                    log::info!("Transcript: {}", alt.transcript);
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(e) => log::error!("Transcription error: {}", e),
                }
            }
        }
    });

    // Clone for input stream
    let recording_flag_stream = Arc::clone(&recording_flag);

    // Create an input stream with cloned values
    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _| {
            if recording_flag_stream.load(Ordering::SeqCst) {
                let audio_data = data.to_vec();
                if let Err(e) = tx.try_send(audio_data) {
                    log::error!("Error sending audio data: {}", e);
                }
            }
        },
        move |err| {
            log::error!("Error in audio stream: {}", err);
        },
        None
    ).map_err(|e| e.to_string())?;

    log::info!("Stream created, attempting to play...");
    stream.play().map_err(|e| e.to_string())?;
    log::info!("Stream started successfully");

    Ok(())
}

#[tauri::command]
async fn stop_recording(state: tauri::State<'_, AppState>) -> Result<(), String> {
    log::info!("Stopping recording");
    state.is_recording.store(false, Ordering::SeqCst);
    log::info!("Recording flag: {:?}", state.is_recording);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    dotenv().ok();

    let app_state = AppState {
        user: Mutex::new(None),
        is_recording: Arc::new(AtomicBool::new(false)),
        audio_sender: Mutex::new(None),
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
