use crate::audio::{
    wav_spec_from_config, write_input_data
};
use crate::audio::macos::{
    volume::*,
    aggregate_device::*,
    helpers::*
};
use crate::state::AppState;
use crate::state::RecordingState;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{HostId, SampleRate};
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
use uuid::Uuid;

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
                                let has_signal = max_amplitude > 0.0000001;

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

#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn start_aggregate_recording(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut recording_state = state.recording_state.lock().map_err(|e| e.to_string())?;

    match *recording_state {
        RecordingState::Stopped => {
            *recording_state = RecordingState::Recording;
            log::info!("Starting aggregate recording");

            // 1. If an aggregate device with this name/UID already exists, remove it
            let agg_device_name = "AI-Jeff-recorder";
            let agg_device_uid = Uuid::new_v4().to_string();
            if check_device_exists(agg_device_name) {
                log::info!("Found existing aggregate device, removing it...");
                if let Some(agg_device_id) =
                    *state.system_agg_device_id.lock().map_err(|e| e.to_string())?
                {
                    remove_aggregate_device(agg_device_id).map_err(|e| e.to_string())?;
                }
            }

            // 2. Identify default input & output devices
            let host = cpal::host_from_id(HostId::CoreAudio)
                .map_err(|e| e.to_string())?;

            let default_output_device = host
                .default_output_device()
                .ok_or("No default output device found!")?;
            let output_device_name = default_output_device
                .name()
                .map_err(|_| "Failed to get default output device name")?;
            log::info!("Default output device: {}", output_device_name);

            let default_input_device = host
                .default_input_device()
                .ok_or("No default input device found!")?;
            let default_input_device_name = default_input_device
                .name()
                .map_err(|_| "Failed to get default input device name")?;
            log::info!("Default input device: {}", default_input_device_name);

            // The low-level AudioDeviceID for the default output
            let output_device_id = get_default_output_device()
                .map_err(|e| format!("Failed to get default output device: {}", e))?;
            log::info!("Default device ID: {}", output_device_id);

            // The low-level AudioDeviceID for the default input
            let input_device_id = get_default_input_device()
                .map_err(|e| format!("Failed to get default input device: {}", e))?;
            log::info!("Default input device ID: {}", input_device_id);

            // 3. Pull all device info and locate the UIDs for our chosen devices
            let devices = all_devices_info();
            let output_info = devices.iter().find(|d| d.device_id == output_device_id);
            let input_info  = devices.iter().find(|d| d.device_id == input_device_id);

            // Fallback to names if for some reason device_id wasn't found
            // but typically device_id should suffice
            let output_uid = output_info.map(|d| d.device_uid.clone())
                .unwrap_or(output_device_name.clone());
            let input_uid  = input_info.map(|d| d.device_uid.clone())
                .unwrap_or(default_input_device_name.clone());

            log::info!("Output UID for aggregator: {}", output_uid);
            log::info!("Input UID for aggregator: {}", input_uid);

            // (Optional) Log available configurations for debugging
            log::info!("Default output device configurations:");
            if let Ok(configs) = default_output_device.supported_output_configs() {
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

            // 4. Create the new aggregate device
            thread::sleep(Duration::from_millis(400));
            let create_result = create_aggregate_device(
                &input_uid,
                &output_uid,
                agg_device_name,
                &agg_device_uid,
            ).map_err(|e| format!("Failed to create aggregate device: {:?}", e))?;

            log::info!(
                "Successfully created aggregate device. ID: {}, Tap ID: {}",
                create_result.aggregate_device_id,
                create_result.tap_id
            );

            // 5. Store this ID so we can remove it later
            *state.system_agg_device_id.lock().map_err(|e| e.to_string())? =
                Some(create_result.aggregate_device_id);

            // 6. Wait a moment, then set aggregator as the default device
            thread::sleep(Duration::from_millis(600));
            set_default_device(agg_device_name).map_err(|e| {
                log::error!("Failed to set aggregate device as default: {}", e);
                e
            })?;
            log::info!("Successfully set aggregate device as default output");

            // 7. Now open it in CPAL for input, etc...
            //    (Same logic you already have to record, spawn thread, etc.)

            Ok(())
        }
        _ => Err("Recording system output already in progress".to_string()),
    }
}

// #[cfg(target_os = "macos")]
// #[tauri::command]
// pub async fn start_aggregate_recording(state: tauri::State<'_, AppState>) -> Result<(), String> {
//     let mut recording_state = state.recording_state.lock().map_err(|e| e.to_string())?;

//     match *recording_state {
//         RecordingState::Stopped => {
//             *recording_state = RecordingState::Recording;
//             log::info!("Starting aggregate recording");

//             let agg_device_name = "AI-Jeff-recorder";
//             let agg_device_uid = Uuid::new_v4().to_string();

//             // 1. If an aggregate device with this name/UID already exists, remove it
//             if check_device_exists(agg_device_name) {
//                 log::info!("Found existing aggregate device, removing it...");
//                 if let Some(agg_device_id) =
//                     *state.system_agg_device_id.lock().map_err(|e| e.to_string())?
//                 {
//                     remove_aggregate_device(agg_device_id).map_err(|e| e.to_string())?;
//                 }
//             }

//             // 2. Identify default input & output devices (for the aggregator)
//             let host = cpal::host_from_id(HostId::CoreAudio).map_err(|e| e.to_string())?;
//             let default_output_device = host
//                 .default_output_device()
//                 .ok_or("No default output device found!")?;

//             let device_id = get_default_output_device().map_err(|e| format!("Failed to get default output device: {}", e))?;
//             log::info!("Default device ID: {}", device_id);

//             // Log the default output device name
//             let output_device_name = default_output_device
//                 .name()
//                 .map_err(|_| "Failed to get default output device name")?;
//             log::info!("Default output device: {}", output_device_name);

//             let default_input_device = host
//                 .default_input_device()
//                 .ok_or("No default input device found!")?;

//             // Log the default input device name
//             let default_input_device_id = default_input_device
//                 .name()
//                 .map_err(|_| "Failed to get default input device name")?;
//             log::info!("Default input device: {}", default_input_device_id);

//             let all_device_uids = all_device_uids();
//             log::info!("All device UIDs: {:?}", all_device_uids);

//             let default_output_device_id = default_output_device
//                 .name()
//                 .map_err(|_| "Failed to get default output device name")?;

//             // (Optional) Log configurations for debugging
//             log::info!("Default output device configurations:");
//             if let Ok(configs) = default_output_device.supported_output_configs() {
//                 for config in configs {
//                     log::info!(
//                         "  Rate: {:?}-{:?}, Channels: {}, Format: {:?}",
//                         config.min_sample_rate(),
//                         config.max_sample_rate(),
//                         config.channels(),
//                         config.sample_format()
//                     );
//                 }
//             }

//             // 3. Create the new aggregate device
//             let create_result = {
//                 // Add a small delay to reduce CoreAudio flakiness
//                 thread::sleep(Duration::from_millis(400));

//                 create_aggregate_device(
//                     &default_input_device_id,
//                     &default_output_device_id,
//                     // &input_uid,
//                     // &output_uid,
//                     agg_device_name,
//                     &agg_device_uid,
//                 ).map_err(|e| format!("Failed to create aggregate device: {:?}", e))?
//             };

//             log::info!(
//                 "Successfully created aggregate device. ID: {}, Tap ID: {}",
//                 create_result.aggregate_device_id,
//                 create_result.tap_id
//             );

//             // 4. Store this ID so we can remove later
//             *state.system_agg_device_id.lock().map_err(|e| e.to_string())? =
//                 Some(create_result.aggregate_device_id);

//             // 5. Wait a moment, then set the aggregator as the default device
//             log::info!("Waiting for device to initialize...");
//             thread::sleep(Duration::from_millis(600));

//             set_default_device(agg_device_name).map_err(|e| {
//                 log::error!("Failed to set aggregate device as default: {}", e);
//                 e
//             })?;
//             log::info!("Successfully set aggregate device as default output");

//             // 6. Now open it in CPAL for input
//             thread::sleep(Duration::from_millis(600));

//             let aggregate_device = host
//                 .devices()
//                 .map_err(|e| e.to_string())?
//                 .find(|d| {
//                     d.name()
//                      .map(|name| name == agg_device_name)
//                      .unwrap_or(false)
//                 })
//                 .ok_or("Unable to find newly created aggregate device via CPAL")?;

//             // Choose or negotiate a suitable input config
//             let config = aggregate_device
//                 .supported_input_configs()
//                 .map_err(|e| format!("Error getting supported configs: {}", e))?
//                 .find(|c| {
//                     (c.channels() == 2 || c.channels() == 4)
//                         && c.min_sample_rate() <= SampleRate(48000)
//                         && c.max_sample_rate() >= SampleRate(48000)
//                 })
//                 .map(|c| c.with_sample_rate(SampleRate(48000)))
//                 .ok_or("No suitable input config found")?;

//             log::info!(
//                 "Selected config - Sample rate: {}, Channels: {}, Format: {:?}",
//                 config.sample_rate().0,
//                 config.channels(),
//                 config.sample_format()
//             );

//             // Create a temporary file and WAV writer
//             let temp_file = Builder::new()
//                 .prefix("system_output_recording_")
//                 .suffix(".wav")
//                 .tempfile()
//                 .map_err(|e| format!("Failed to create temp file: {}", e))?;

//             let output_path = temp_file.path().to_path_buf();
//             log::info!("Recording system audio to: {:?}", output_path);

//             let spec = wav_spec_from_config(&config);
//             let writer = WavWriter::create(&output_path, spec)
//                 .map_err(|e| format!("Failed to create WavWriter: {}", e))?;
//             let path_str = output_path.to_string_lossy().to_string();

//             // Arc for concurrency
//             let writer = Arc::new(Mutex::new(Some((writer, path_str.clone()))));

//             // Set up channels and state
//             let (sender, receiver) = channel();
//             *state.recording_sender.lock().map_err(|e| e.to_string())? = Some(sender);
//             *state.audio_writer.lock().map_err(|e| e.to_string())? = Some(Arc::clone(&writer));
//             *state.temp_file.lock().map_err(|e| e.to_string())? = Some(temp_file);

//             state.is_recording.store(true, Ordering::SeqCst);
//             let recording_flag = Arc::clone(&state.is_recording);

//             *state.thread_completed.lock().map_err(|e| e.to_string())? = false;
//             let thread_completed_clone = Arc::clone(&state.thread_completed);

//             // 7. Build and play the input stream in a separate thread
//             std::thread::spawn(move || {
//                 let recording_flag_stream = Arc::clone(&recording_flag);
//                 let writer_clone = Arc::clone(&writer);
//                 let stream = aggregate_device.build_input_stream(
//                     &config.into(),
//                     move |data: &[f32], _| {
//                         if recording_flag_stream.load(Ordering::SeqCst) {
//                             // let max_amplitude = data.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
//                             // if max_amplitude > 0.0000001 {
//                             //     write_input_data(data, &writer_clone);
//                             // }
//                             write_input_data(data, &writer_clone);
//                         }
//                     },
//                     move |err| {
//                         log::error!("Error in system output audio stream: {}", err);
//                     },
//                     None,
//                 ).expect("Failed to build input stream");

//                 stream.play().unwrap();

//                 // Wait for stop signal
//                 while recording_flag.load(Ordering::SeqCst) {
//                     if receiver.recv().is_err() {
//                         log::info!("Recording thread received stop signal");
//                         break;
//                     }
//                 }

//                 // Close the stream
//                 drop(stream);

//                 log::info!("Finalizing aggregate recording...");
//                 if let Ok(mut writer_guard) = writer.lock() {
//                     if let Some((writer, _)) = writer_guard.take() {
//                         if let Err(e) = writer.finalize() {
//                             log::error!("Error finalizing WAV writer: {}", e);
//                         }
//                     }
//                 } else {
//                     log::error!("Failed to acquire lock on WAV writer for finalization");
//                 }

//                 // Mark thread completed
//                 if let Ok(mut completed) = thread_completed_clone.lock() {
//                     *completed = true;
//                     log::info!("Aggregate recording thread completed");
//                 }
//             });

//             // 8. Log out tap info, purely for debugging
//             if let Ok(tap_format) = get_tap_stream_audio_description(create_result.tap_id) {
//                 log::info!(
//                     "Tap device format - Sample rate: {}, Channels: {}, Bits per channel: {}, Format ID: {}",
//                     tap_format.mSampleRate,
//                     tap_format.mChannelsPerFrame,
//                     tap_format.mBitsPerChannel,
//                     tap_format.mFormatID
//                 );
//             } else {
//                 log::warn!("Could not get tap device audio format");
//             }

//             Ok(())
//         }
//         _ => Err("Recording system output already in progress".to_string()),
//     }
// }

#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn stop_aggregate_recording(
    state: tauri::State<'_, AppState>,
    token: String,
    refine: bool,
) -> Result<(), String> {
    // 1. Update recording state
    {
        let mut recording_state = state.recording_state.lock().map_err(|e| e.to_string())?;
        match *recording_state {
            RecordingState::Recording | RecordingState::Paused => {
                *recording_state = RecordingState::Stopped;
            }
            RecordingState::Stopped => return Err("Recording not started".to_string()),
        }
    }

    log::info!("Stopping aggregate recording");

    // 2. Try to restore the original default output device
    let host = cpal::default_host();
    let default_output_device = host
        .default_output_device()
        .ok_or_else(|| "No default output device found".to_string())?;

    let original_device_name = default_output_device
        .name()
        .map_err(|e| format!("Failed to get original device name: {}", e))?;

    log::info!("original_device_name: {}", original_device_name);

    if let Err(e) = set_default_device(&original_device_name) {
        log::warn!("Failed to restore original output device: {}", e);
    }

    // Wait for the device switch to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

    // 3. Signal the recording thread to stop
    state.is_recording.store(false, Ordering::SeqCst);
    if let Some(sender) = state.recording_sender.lock().map_err(|e| e.to_string())?.take() {
        sender.send(()).map_err(|e| e.to_string())?;
    }

    // 4. Wait for the thread to finalize the WAV
    while !state.thread_completed.lock().map_err(|e| e.to_string())?.clone() {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // 5. Retrieve the recorded file path
    let audio_file_path = {
        let audio_writer_guard = state.audio_writer.lock().map_err(|e| e.to_string())?;
        if let Some(writer_arc) = audio_writer_guard.as_ref() {
            match writer_arc.lock() {
                Ok(writer_guard) => {
                    writer_guard.as_ref().map(|(_, p)| p.clone())
                },
                Err(_) => {
                    log::error!("Failed to lock writer guard");
                    None
                }
            }
        } else {
            None
        }
    }
    // As a fallback, try the temp file handle
    .or_else(|| {
        state.temp_file.lock().ok()
            .and_then(|guard| guard.as_ref()
                .map(|temp_file| {
                    temp_file.path().to_string_lossy().to_string()
                }))
    });

    // 6. If we have an audio path, transcribe
    if let Some(file_path) = audio_file_path {
        log::info!("Transcribing system output audio file: {}", file_path);

        let user_id = {
            let user_guard = state.existing_user.lock().map_err(|e| e.to_string())?;
            user_guard
                .as_ref()
                .ok_or_else(|| "User not authenticated".to_string())?
                .id
                .clone()
        };

        let transcription_result =
            transcribe_audio(user_id, token, file_path.clone(), refine).await;

        // Clean up the temp file
        if let Ok(mut temp_file_guard) = state.temp_file.lock() {
            let path = temp_file_guard.as_ref().map(|f| f.path().to_owned());
            temp_file_guard.take(); // remove reference, triggers deletion

            if let Some(path) = path {
                if path.exists() {
                    log::warn!("Temporary file still exists: {:?}", path);
                } else {
                    log::info!("Successfully deleted temporary file at: {:?}", path);
                }
            }
        }

        log::info!("Transcription result: {:?}", transcription_result);

        match transcription_result {
            Ok(transcript) => {
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

    // 7. Finally, remove the aggregate device
    if let Some(agg_device_id) = *state.system_agg_device_id.lock().map_err(|e| e.to_string())? {
        if let Err(e) = remove_aggregate_device(agg_device_id) {
            log::warn!("Failed to remove aggregate device: {:?}", e);
        }
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
pub fn start_aggregate_recording(
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
pub fn stop_aggregate_recording(
    state: tauri::State<'_, AppState>,
    token: String,
    refine: bool,
) -> Result<(), i32> {
    Ok(())
}