use crate::audio::{
    wav_spec_from_config, write_input_data
};
use crate::audio::macos::volume::{get_default_output_device, get_device_volume};
use crate::state::AppState;
use crate::state::RecordingState;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::WavWriter;
use reqwest::multipart::{Form, Part};
use serde_json::Value;
use std::sync::mpsc::channel;
use std::sync::{atomic::Ordering, Arc, Mutex};
use std::{thread, time::{Instant, Duration}};
use tauri::{Emitter, EventTarget};
use tempfile::Builder;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

async fn transcribe_audio(
    user_id: String,
    token: String,
    file_path: String,
    refine: bool,
) -> Result<String, String> {
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

    // Log the file size and first few bytes for debugging
    log::info!("Audio file size: {} bytes", buffer.len());

    // Create the multipart form
    let part = Part::bytes(buffer)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| format!("Failed to create form part: {}", e))?;

    let form = Form::new()
        .part("file", part)
        .text("refine", refine.to_string())
        .text("userId", user_id.clone());

    log::info!("Sending transcription request for user: {}", user_id);

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

    // Log the response status
    log::info!("Response status: {}", response.status());

    let response_text = response.text().await.map_err(|e| {
        log::error!("Failed to get response text: {}", e);
        e.to_string()
    })?;

    log::info!("Raw response: {}", response_text);

    let json_value: Value = serde_json::from_str(&response_text).map_err(|e| {
        log::error!("Failed to parse JSON: {}", e);
        e.to_string()
    })?;

    // Check for error in response
    if let Some(error) = json_value.get("error") {
        let error_msg = error.as_str().unwrap_or("Unknown error");
        log::error!("Server returned error: {}", error_msg);
        return Err(error_msg.to_string());
    }

    // Check for "no dialog" message
    if let Some(message) = json_value.get("message") {
        if message.as_str() == Some("No dialog detected") {
            log::info!("No dialog detected in audio");
            return Ok("".to_string());
        }
    }

    // Extract the transcription
    let transcription = if refine {
        json_value["refined"]
            .as_str()
            .ok_or_else(|| "No refined transcription in response".to_string())?
            .to_string()
    } else {
        json_value["transcription"]
            .as_str()
            .ok_or_else(|| "No transcription in response".to_string())?
            .to_string()
    };

    let duration = start_time.elapsed();
    log::info!("Transcription completed in {:?}", duration);

    Ok(transcription)
}


#[tauri::command]
pub async fn start_recording(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut recording_state = state.recording_state.lock().map_err(|e| e.to_string())?;

    match *recording_state {
        RecordingState::Stopped => {
            *recording_state = RecordingState::Recording;

            log::info!("Starting recording");

            // Get and store the default output device
            let device_id = get_default_output_device().map_err(|e| format!("Failed to get default output device: {}", e))?;
            *state.audio_device_id.lock().map_err(|e| e.to_string())? = Some(device_id);

            // Get and store the current volume
            let current_volume = get_device_volume(device_id).map_err(|e| format!("Failed to get device volume: {}", e))?;
            *state.original_volume.lock().map_err(|e| e.to_string())? = Some(current_volume);

             // Introduce a small delay so that our sound effect is heard
            thread::sleep(Duration::from_millis(300));

            // Set volume to 0
            // fade_volume(device_id, current_volume, 0.0, 6, 100); // 6 steps, 100ms between steps

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

            log::info!("Selected input device: {}", device.name().map_err(|e| e.to_string())?);

            if let Ok(configs) = device.supported_input_configs() {
                for config in configs {
                    log::info!(
                        "  Rate: {:?}-{:?}, Channels: {}, Format: {:?}",
                        config.min_sample_rate(),
                        config.max_sample_rate(),
                        config.channels(),
                        config.sample_format()
                    );
                }
            }

            let config = device.default_input_config().map_err(|e| e.to_string())?;

            let spec = wav_spec_from_config(&config);
            let writer = WavWriter::create(&output_path, spec).map_err(|e| e.to_string())?;
            let path_str = output_path.to_string_lossy().to_string();

            let writer = Arc::new(Mutex::new(Some((writer, path_str.clone()))));
            let writer_clone = Arc::clone(&writer);

            let (sender, receiver) = channel();
            *state.recording_sender.lock().map_err(|e| e.to_string())? = Some(sender);

            *state.audio_writer.lock().map_err(|e| e.to_string())? = Some(Arc::clone(&writer));

            state.is_recording.store(true, Ordering::SeqCst);
            let recording_flag = Arc::clone(&state.is_recording);

            // Store temp_file handle in state to prevent premature deletion
            *state.temp_file.lock().map_err(|e| e.to_string())? = Some(temp_file);

            // Start recording
            std::thread::spawn(move || {
                let recording_flag_stream = Arc::clone(&recording_flag);
                let stream = device
                    .build_input_stream(
                        &config.into(),
                        move |data: &[f32], _| {
                            if recording_flag_stream.load(Ordering::SeqCst) {
                                let max_amplitude = data.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
                                let has_signal = max_amplitude > 0.00001;

                                if has_signal {
                                    write_input_data(data, &writer_clone);
                                }
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
    refine: bool,
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

    // Restore the original volume
    // if let (Some(device_id), Some(original_volume)) = (
    //     *state.audio_device_id.lock().map_err(|e| e.to_string())?,
    //     *state.original_volume.lock().map_err(|e| e.to_string())?,
    // ) {
    //     fade_volume(device_id, 0.0, original_volume, 6, 100); // 6 steps, 100ms between steps
    // }

    // Signal the recording thread to stop
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

    // Handle transcription if we have a file
    if let Some(file_path) = audio_file_path {
        log::info!("Transcribing audio file: {}", file_path);

        // Get the user ID before any async operations
        let user_id: String = {
            let user_guard = state.existing_user.lock().map_err(|e| e.to_string())?;
            user_guard
                .as_ref()
                .and_then(|u| Some(u.id.clone()))
                .ok_or_else(|| "User not authenticated".to_string())?
        };

        let transcription_result = transcribe_audio(user_id, token, file_path.clone(), refine).await;

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

        log::info!("Transcription result: {:?}", transcription_result);

        match transcription_result {
            Ok(transcript) => {
                // Only emit if transcript is not empty
                if !transcript.is_empty() {
                    let event_name = if refine {
                        "refined-transcription-complete"
                    } else {
                        "transcription-complete"
                    };

                    state
                        .app_handle
                        .emit_to(EventTarget::any(), event_name, Some(transcript))
                        .map_err(|e| e.to_string())?;
                } else {
                    log::info!("Skipping event emission for empty transcript");
                }
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
