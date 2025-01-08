use hound::WavSpec;
use log;
use std::fs::File;
use std::io::BufWriter;
use std::sync::Arc;
use std::sync::Mutex;

pub fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> WavSpec {
    WavSpec {
        channels: config.channels() as u16,
        sample_rate: config.sample_rate().0,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    }
}

pub fn write_input_data(
    input: &[f32],
    writer: &Arc<Mutex<Option<(hound::WavWriter<BufWriter<File>>, String)>>>,
) {
    match writer.lock() {
        Ok(mut guard) => {
            if let Some((writer, _)) = guard.as_mut() {
                for &sample in input.iter() {
                    let converted_sample = (sample * i16::MAX as f32) as i16;
                    match writer.write_sample(converted_sample) {
                        Ok(_) => {},
                        Err(e) => {
                            log::error!("Failed to write sample: {}", e);
                            return;
                        }
                    }
                }
            } else {
                log::error!("WAV writer is not available");
            }
        }
        Err(e) => {
            log::error!("Failed to acquire lock on WAV writer: {}", e);
        }
    }
}
