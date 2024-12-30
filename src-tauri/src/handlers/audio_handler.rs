use crate::audio::{wav_spec_from_config, write_input_data};
use crate::state::AppState;
use crate::state::RecordingState;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::WavWriter;
use reqwest::multipart::{Form, Part};
use serde_json::Value;
use std::sync::mpsc::channel;
use std::sync::{atomic::Ordering, Arc, Mutex};
use std::time::Instant;
use tauri::{Emitter, EventTarget};
use tempfile::Builder;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[tauri::command]
pub async fn start_recording(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut recording_state = state.recording_state.lock().map_err(|e| e.to_string())?;

    match *recording_state {
        RecordingState::Stopped => {
            *recording_state = RecordingState::Recording;

            log::info!("Starting recording");

            // Create temp file with .wav extension
            let temp_file = Builder::new()
                .prefix("recording_")
                .suffix(".wav")
                .tempfile()
                .map_err(|e| format!("Failed to create temp file: {}", e))?;

            let output_path = temp_file.path().to_path_buf();
            log::info!("Recording to temporary file: {:?}", output_path);

            let host = cpal::default_host();
            let device = host
                .default_input_device()
                .ok_or_else(|| "No input device available".to_string())?;

            let config = device.default_input_config().map_err(|e| e.to_string())?;

            let spec = wav_spec_from_config(&config);
            let writer = WavWriter::create(&output_path, spec).map_err(|e| e.to_string())?;
            let path_str = output_path.to_string_lossy().to_string();

            let writer = Arc::new(Mutex::new(Some((writer, path_str))));
            let writer_clone = Arc::clone(&writer);

            let (sender, receiver) = channel();
            *state.recording_sender.lock().map_err(|e| e.to_string())? = Some(sender);

            *state.audio_writer.lock().map_err(|e| e.to_string())? = Some(Arc::clone(&writer));

            state.is_recording.store(true, Ordering::SeqCst);
            let recording_flag = Arc::clone(&state.is_recording);

            // Store temp_file handle in state to prevent premature deletion
            *state.temp_file.lock().map_err(|e| e.to_string())? = Some(temp_file);

            std::thread::spawn(move || {
                let recording_flag_stream = Arc::clone(&recording_flag);
                let stream = device
                    .build_input_stream(
                        &config.into(),
                        move |data: &[f32], _| {
                            if recording_flag_stream.load(Ordering::SeqCst) {
                                write_input_data(data, &writer_clone);
                            }
                        },
                        move |err| {
                            log::error!("Error in audio stream: {}", err);
                        },
                        None,
                    )
                    .unwrap();

                stream.play().unwrap();

                while recording_flag.load(Ordering::SeqCst) {
                    if receiver.recv().is_err() {
                        break;
                    }
                }

                if let Ok(mut writer_guard) = writer.lock() {
                    if let Some((writer, _)) = writer_guard.take() {
                        writer.finalize().unwrap();
                    }
                }
            });

            Ok(())
        }
        _ => Err("Recording already in progress".to_string()),
    }
}

#[tauri::command]
pub async fn stop_recording(
    state: tauri::State<'_, AppState>,
    _app_handle: tauri::AppHandle,
    token: String,
) -> Result<(), String> {
    {
        let mut recording_state = state.recording_state.lock().map_err(|e| e.to_string())?;
        match *recording_state {
            RecordingState::Recording | RecordingState::Paused => {
                *recording_state = RecordingState::Stopped;
            }
            RecordingState::Stopped => return Err("Recording not started".to_string()),
        }
    }

    log::info!("Stopping recording");
    state.is_recording.store(false, Ordering::SeqCst);

    if let Some(sender) = state
        .recording_sender
        .lock()
        .map_err(|e| e.to_string())?
        .as_ref()
    {
        sender.send(()).map_err(|e| e.to_string())?;
    }

    let audio_file_path = {
        let audio_writer_guard = state.audio_writer.lock().map_err(|e| e.to_string())?;
        if let Some(writer_arc) = audio_writer_guard.as_ref() {
            let writer_guard = writer_arc.lock().map_err(|e| e.to_string())?;
            writer_guard.as_ref().map(|(_, path)| path.clone())
        } else {
            None
        }
    };

    if let Some(file_path) = audio_file_path {
        log::info!("Transcribing audio file: {}", file_path);

        let transcription_result = transcribe_audio(token, file_path.clone()).await;

        // Clean up and verify temp file deletion
        if let Ok(mut temp_file_guard) = state.temp_file.lock() {
            let path = temp_file_guard.as_ref().map(|f| f.path().to_owned());
            temp_file_guard.take(); // This will remove and delete the temp file

            // Verify deletion
            if let Some(file_path) = path {
                if file_path.exists() {
                    log::warn!("Temporary file still exists at: {:?}", file_path);
                } else {
                    log::info!("Successfully deleted temporary file at: {:?}", file_path);
                }
            }
        }

        match transcription_result {
            Ok(transcript) => {
                state
                    .app_handle
                    .emit_to(
                        EventTarget::any(),
                        "transcription-complete",
                        Some(transcript),
                    )
                    .map_err(|e| e.to_string())?;
            }
            Err(e) => {
                state
                    .app_handle
                    .emit_to(EventTarget::any(), "transcription-error", Some(e))
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}

async fn transcribe_audio(token: String, file_path: String) -> Result<String, String> {
    let start_time = Instant::now();
    log::info!("Starting transcription for file: {}", file_path);

    // Read the file into a buffer
    let mut file = File::open(&file_path)
        .await
        .map_err(|e| format!("Failed to open file: {}", e))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Create the multipart form
    let part = Part::bytes(buffer)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| format!("Failed to create form part: {}", e))?;

    let form = Form::new()
        .part("file", part)
        // todo: let the user adjust this in settings
        .text("refine", "false");

    log::info!("Sending transcription request");

    // Send the request
    let client = reqwest::Client::new();
    let response = client
        // .post("http://localhost:8787/api/transcribe")
        .post("https://jeff-ai-cf-be.mrboutte21.workers.dev/api/transcribe")
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to send transcription request: {}", e);
            e.to_string()
        })?;

    let json_value = response.json::<Value>().await.map_err(|e| {
        log::error!("Failed to parse response as JSON: {}", e);
        e.to_string()
    })?;

    log::info!("Transcription response: {}", json_value);

    // Extract the transcription from the response
    let transcription = json_value["transcription"]
        .as_str()
        .ok_or_else(|| "No transcription in response".to_string())?
        .to_string();

    let duration = start_time.elapsed();
    log::info!("Transcription completed in {:?}", duration);

    Ok(transcription)
}
