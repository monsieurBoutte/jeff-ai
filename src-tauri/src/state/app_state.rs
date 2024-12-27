use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;
use std::io::BufWriter;
use std::fs::File;
use hound::WavWriter;
use serde::{Deserialize, Serialize};
use crate::models::User;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RecordingState {
    Stopped,
    Recording,
    Paused,
}

pub struct AppState {
    pub user: Mutex<Option<User>>,
    pub recording_state: Mutex<RecordingState>,
    pub is_recording: Arc<AtomicBool>,
    pub audio_writer: Mutex<Option<Arc<Mutex<Option<(WavWriter<BufWriter<File>>, String)>>>>>,
    pub recording_sender: Arc<Mutex<Option<Sender<()>>>>,
    pub app_handle: tauri::AppHandle,
}