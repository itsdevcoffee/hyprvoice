//! Audio capture using CPAL (Cross-Platform Audio Library)
//!
//! Replaces PipeWire-specific code with cross-platform CPAL implementation

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Capture audio from default microphone for fixed duration
///
/// Returns f32 PCM samples at 16kHz mono (Whisper requirement)
pub fn capture(duration_secs: u32, sample_rate: u32) -> Result<Vec<f32>> {
    info!("Starting audio capture: {}s", duration_secs);

    // Get default input device
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .context("No input device available. Check microphone permissions.")?;

    info!(
        "Using audio device: {}",
        device.name().unwrap_or_else(|_| "Unknown".to_string())
    );

    // Configure for mono f32 at requested sample rate
    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: cpal::SampleRate(sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    // Pre-allocate buffer based on expected duration
    let expected_samples = (sample_rate * duration_secs) as usize;
    let buffer = Arc::new(Mutex::new(Vec::with_capacity(expected_samples)));
    let buffer_clone = buffer.clone();

    let start_time = Arc::new(Mutex::new(None::<Instant>));
    let start_clone = start_time.clone();

    // Build input stream
    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            // Initialize start time on first callback
            let mut start = start_clone.lock().unwrap();
            if start.is_none() {
                *start = Some(Instant::now());
                info!("Recording started - speak now!");
            }

            // Collect samples
            buffer_clone.lock().unwrap().extend_from_slice(data);
        },
        |err| eprintln!("Stream error: {}", err),
        None,
    )?;

    // Start recording
    stream.play()?;

    // Wait for duration
    std::thread::sleep(Duration::from_secs(duration_secs as u64));

    // Stop stream (drops automatically)
    drop(stream);

    // Extract collected samples
    let samples = Arc::try_unwrap(buffer)
        .map(|mutex| mutex.into_inner().unwrap())
        .unwrap_or_else(|arc| arc.lock().unwrap().clone());

    let actual_duration = samples.len() as f32 / sample_rate as f32;
    info!(
        "Captured {} samples ({:.2}s at {}Hz)",
        samples.len(),
        actual_duration,
        sample_rate
    );

    // Resample to 16kHz if needed
    finalize_audio_samples(samples, sample_rate, 16000)
}

/// Capture in toggle mode - stops when signal received or max duration
pub fn capture_toggle(max_duration_secs: u32, sample_rate: u32) -> Result<Vec<f32>> {
    use crate::state::toggle::should_stop;

    info!("Starting toggle mode capture (max {}s)", max_duration_secs);

    let host = cpal::default_host();
    let device = host.default_input_device().context("No input device")?;

    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: cpal::SampleRate(sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    let expected_samples = (sample_rate * max_duration_secs) as usize;
    let buffer = Arc::new(Mutex::new(Vec::with_capacity(expected_samples)));
    let buffer_clone = buffer.clone();

    let start_time = Arc::new(Mutex::new(None::<Instant>));
    let start_clone = start_time.clone();

    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _| {
            let mut start = start_clone.lock().unwrap();
            if start.is_none() {
                *start = Some(Instant::now());
                info!("Recording started - speak now!");
            }

            buffer_clone.lock().unwrap().extend_from_slice(data);
        },
        |err| eprintln!("Stream error: {}", err),
        None,
    )?;

    stream.play()?;

    // Poll for stop signal or timeout
    let poll_interval = Duration::from_millis(100);
    let max_duration = Duration::from_secs(max_duration_secs as u64);
    let start = Instant::now();

    loop {
        std::thread::sleep(poll_interval);

        if should_stop() {
            info!("Stop signal received");
            break;
        }

        if start.elapsed() >= max_duration {
            info!("Max duration reached ({}s)", max_duration_secs);
            break;
        }
    }

    // Continue recording for 1 second after stop (buffer trailing words)
    info!("Buffering trailing audio (1s)...");
    std::thread::sleep(Duration::from_secs(1));

    drop(stream);

    let samples = Arc::try_unwrap(buffer)
        .map(|mutex| mutex.into_inner().unwrap())
        .unwrap_or_else(|arc| arc.lock().unwrap().clone());

    info!("Captured {} samples", samples.len());

    // Resample to 16kHz if needed
    finalize_audio_samples(samples, sample_rate, 16000)
}

/// Perform post-capture resampling if needed
fn finalize_audio_samples(
    raw_samples: Vec<f32>,
    source_rate: u32,
    target_rate: u32,
) -> Result<Vec<f32>> {
    if raw_samples.is_empty() {
        warn!("No audio captured - check microphone permissions");
        return Ok(Vec::new());
    }

    // Resample to target rate if needed (with tolerance)
    let samples = if source_rate > target_rate + 1000 || source_rate < target_rate - 1000 {
        info!("Resampling from {}Hz to {}Hz", source_rate, target_rate);
        resample(&raw_samples, source_rate, target_rate)
    } else {
        raw_samples
    };

    let final_duration = samples.len() as f32 / target_rate as f32;
    info!(
        "Final audio: {} samples ({:.2}s at {}Hz)",
        samples.len(),
        final_duration,
        target_rate
    );

    Ok(samples)
}

/// High-quality resampling using rubato (sinc interpolation)
fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    use rubato::{FftFixedIn, Resampler};

    // rubato works with f64, convert
    let samples_f64: Vec<f64> = samples.iter().map(|&s| s as f64).collect();

    // Create resampler: chunk size of 1024 is a good balance
    let chunk_size = 1024;
    let mut resampler = match FftFixedIn::<f64>::new(
        from_rate as usize,
        to_rate as usize,
        chunk_size,
        2, // sub_chunks
        1, // channels (mono)
    ) {
        Ok(r) => r,
        Err(e) => {
            warn!("Failed to create resampler: {}, using linear fallback", e);
            return resample_linear(samples, from_rate as f32 / to_rate as f32);
        },
    };

    let mut output_f64 = Vec::new();

    // Process in chunks
    for chunk in samples_f64.chunks(chunk_size) {
        let input = vec![chunk.to_vec()];

        // Pad last chunk if needed
        let input = if chunk.len() < chunk_size {
            let mut padded = chunk.to_vec();
            padded.resize(chunk_size, 0.0);
            vec![padded]
        } else {
            input
        };

        match resampler.process(&input, None) {
            Ok(output) => {
                if let Some(channel) = output.first() {
                    output_f64.extend(channel);
                }
            },
            Err(e) => {
                warn!("Resampling error: {}, using linear fallback", e);
                return resample_linear(samples, from_rate as f32 / to_rate as f32);
            },
        }
    }

    // Convert back to f32
    output_f64.iter().map(|&s: &f64| s as f32).collect()
}

/// Fallback linear resampling (used if rubato fails)
fn resample_linear(samples: &[f32], ratio: f32) -> Vec<f32> {
    let output_len = (samples.len() as f32 / ratio) as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_pos = i as f32 * ratio;
        let src_idx = src_pos as usize;
        let frac = src_pos - src_idx as f32;

        if src_idx + 1 < samples.len() {
            let sample = samples[src_idx] * (1.0 - frac) + samples[src_idx + 1] * frac;
            output.push(sample);
        } else if src_idx < samples.len() {
            output.push(samples[src_idx]);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resample_linear_upsampling() {
        let samples = vec![0.0, 1.0, 0.0, -1.0];
        let result = resample_linear(&samples, 0.5); // 2x upsampling
        assert!(result.len() > samples.len());
    }

    #[test]
    fn test_resample_linear_downsampling() {
        let samples: Vec<f32> = (0..100).map(|x| (x as f32).sin()).collect();
        let result = resample_linear(&samples, 2.0); // 2x downsampling
        assert!(result.len() < samples.len());
    }
}
