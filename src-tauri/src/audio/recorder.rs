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
    log::info!("Writing {} samples to WAV file", input.len());

    // Count non-zero samples for debugging
    let non_zero_count = input.iter().filter(|&&x| x.abs() > 1e-7).count();
    log::info!("Non-zero samples: {}/{}", non_zero_count, input.len());

    // Preview both raw and amplified values
    let preview_samples: Vec<(f32, f32)> = input[..5.min(input.len())]
        .iter()
        .map(|&x| (x, x * 10.0)) // Show both original and amplified values
        .collect();
    log::info!("Sample preview (original, amplified): {:?}", preview_samples);

    match writer.lock() {
        Ok(mut guard) => {
            if let Some((writer, _)) = guard.as_mut() {
                let mut samples_written = 0;
                for &sample in input.iter() {
                    // Apply gain to increase volume (adjust multiplier as needed)
                    let amplified_sample = sample * 10.0; // Increase gain by 10x

                    // Convert to i16 with proper scaling
                    let converted_sample = (amplified_sample * i16::MAX as f32).clamp(
                        i16::MIN as f32,
                        i16::MAX as f32
                    ) as i16;

                    match writer.write_sample(converted_sample) {
                        Ok(_) => samples_written += 1,
                        Err(e) => {
                            log::error!("Failed to write sample: {}", e);
                            return;
                        }
                    }
                }
                log::info!("Successfully wrote {} samples (after amplification)", samples_written);
            } else {
                log::error!("WAV writer is not available");
            }
        }
        Err(e) => {
            log::error!("Failed to acquire lock on WAV writer: {}", e);
        }
    }
}
