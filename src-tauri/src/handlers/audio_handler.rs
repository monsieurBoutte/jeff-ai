use crate::state::AppState;
use crate::audio::{wav_spec_from_config, write_input_data};
use chrono::Local;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{atomic::Ordering, Arc, Mutex};
use std::sync::mpsc::channel;
use tauri::{Emitter, EventTarget};
use crate::state::RecordingState;
use dirs;
use hound::WavWriter;
use std::time::Instant;

#[tauri::command]
pub async fn start_recording(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut recording_state = state.recording_state.lock().map_err(|e| e.to_string())?;

    match *recording_state {
        RecordingState::Stopped => {
            *recording_state = RecordingState::Recording;

            log::info!("Starting recording");

            let desktop_dir = dirs::desktop_dir()
                .ok_or_else(|| "Could not find desktop directory".to_string())?;

            let timestamp = Local::now().format("%Y%m%d_%H%M%S");
            let output_path = desktop_dir.join(format!("recording_{}.wav", timestamp));

            log::info!("Recording to: {:?}", output_path);

            let host = cpal::default_host();
            let device = host.default_input_device()
                .ok_or_else(|| "No input device available".to_string())?;

            let config = device.default_input_config()
                .map_err(|e| e.to_string())?;

            let spec = wav_spec_from_config(&config);
            let writer = WavWriter::create(&output_path, spec)
                .map_err(|e| e.to_string())?;
            let path_str = output_path.to_string_lossy().to_string();

            let writer = Arc::new(Mutex::new(Some((writer, path_str))));
            let writer_clone = Arc::clone(&writer);

            let (sender, receiver) = channel();
            *state.recording_sender.lock().map_err(|e| e.to_string())? = Some(sender);

            *state.audio_writer.lock().map_err(|e| e.to_string())? = Some(Arc::clone(&writer));

            state.is_recording.store(true, Ordering::SeqCst);
            let recording_flag = Arc::clone(&state.is_recording);

            std::thread::spawn(move || {
                let recording_flag_stream = Arc::clone(&recording_flag);
                let stream = device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _| {
                        if recording_flag_stream.load(Ordering::SeqCst) {
                            write_input_data(data, &writer_clone);
                        }
                    },
                    move |err| {
                        log::error!("Error in audio stream: {}", err);
                    },
                    None
                ).unwrap();

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
        _ => Err("Recording already in progress".to_string())
    }
}

#[tauri::command]
pub async fn stop_recording(state: tauri::State<'_, AppState>, _app_handle: tauri::AppHandle) -> Result<(), String> {
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

    if let Some(sender) = state.recording_sender.lock().map_err(|e| e.to_string())?.as_ref() {
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
        match transcribe_audio(file_path).await {
            Ok(transcript) => {
                state.app_handle.emit_to(EventTarget::any(), "transcription-complete", Some(transcript))
                    .map_err(|e| e.to_string())?;
            }
            Err(e) => {
                state.app_handle.emit_to(EventTarget::any(), "transcription-error", Some(e))
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}

async fn transcribe_audio(file_path: String) -> Result<String, String> {
    use deepgram::{
        Deepgram,
        common::{
            audio_source::AudioSource,
            options::{Language, Options},
        },
    };
    use tokio::fs::File as TokioFile;

    let deepgram_api_key = std::env::var("DEEPGRAM_API_KEY")
    .expect("DEEPGRAM_API_KEY not set in .env file");

    let dg_client = Deepgram::new(&deepgram_api_key)
        .map_err(|e| format!("Failed to create Deepgram client: {}", e))?;

    let file = TokioFile::open(&file_path).await.unwrap();
    let source = AudioSource::from_buffer_with_mime_type(file, "audio/wav");

    let options = Options::builder()
        .punctuate(true)
        .language(Language::en_US)
        .build();

    let start_time = Instant::now();
    log::info!("Starting transcription for file: {}", file_path);

    let response = dg_client
        .transcription()
        .prerecorded(source, &options)
        .await
        .map_err(|e| format!("Transcription failed: {}", e))?;

    let duration = start_time.elapsed();
    log::info!("Transcription completed in {:.2?}", duration);

    let transcript = &response.results.channels[0].alternatives[0].transcript;
    log::info!("Transcript: {}", transcript);

    Ok(transcript.to_string())
}