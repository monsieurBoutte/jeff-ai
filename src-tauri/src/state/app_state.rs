use crate::models::{ExistingUser, User};
use hound::WavWriter;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufWriter;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RecordingState {
    Stopped,
    Recording,
    Paused,
}

pub struct AppState {
    pub user: Mutex<Option<User>>,
    pub existing_user: Mutex<Option<ExistingUser>>,
    pub recording_state: Mutex<RecordingState>,
    pub is_recording: Arc<AtomicBool>,
    pub audio_writer: Mutex<Option<Arc<Mutex<Option<(WavWriter<BufWriter<File>>, String)>>>>>,
    pub recording_sender: Arc<Mutex<Option<Sender<()>>>>,
    pub app_handle: tauri::AppHandle,
    pub temp_file: Arc<Mutex<Option<NamedTempFile>>>,
}
