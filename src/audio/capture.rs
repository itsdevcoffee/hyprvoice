use anyhow::{Context, Result};
use libspa as spa_lib;
use pipewire as pw;
use pw::properties::properties;
use pw::spa;
use pw::spa::pod::Pod;
use pw::stream::StreamFlags;
use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// PipeWire components needed for audio capture
struct PipeWireContext {
    mainloop: pw::main_loop::MainLoopRc,
    core: pw::core::CoreRc,
}

/// Initialize PipeWire main loop, context, and core
fn init_pipewire() -> Result<PipeWireContext> {
    pw::init();

    let mainloop = pw::main_loop::MainLoopRc::new(None)
        .context("Failed to create PipeWire main loop")?;
    let context = pw::context::ContextRc::new(&mainloop, None)
        .context("Failed to create PipeWire context")?;
    let core = context
        .connect_rc(None)
        .context("Failed to connect to PipeWire")?;

    Ok(PipeWireContext { mainloop, core })
}

/// Create stream properties for audio capture
fn create_stream_properties() -> pw::properties::PropertiesBox {
    properties! {
        *pw::keys::MEDIA_TYPE => "Audio",
        *pw::keys::MEDIA_CATEGORY => "Capture",
        *pw::keys::MEDIA_ROLE => "Communication",
        *pw::keys::NODE_NAME => "dev-voice-capture",
        *pw::keys::AUDIO_CHANNELS => "1",
        *pw::keys::AUDIO_FORMAT => "F32LE",
    }
}

/// Build audio format params for F32LE mono
fn build_audio_params() -> Result<Vec<u8>> {
    let obj = pw::spa::pod::object!(
        pw::spa::utils::SpaTypes::ObjectParamFormat,
        pw::spa::param::ParamType::EnumFormat,
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::MediaType,
            Id,
            pw::spa::param::format::MediaType::Audio
        ),
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::MediaSubtype,
            Id,
            pw::spa::param::format::MediaSubtype::Raw
        ),
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::AudioFormat,
            Id,
            spa_lib::param::audio::AudioFormat::F32LE
        ),
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::AudioChannels,
            Int,
            1
        ),
    );

    let values = pw::spa::pod::serialize::PodSerializer::serialize(
        Cursor::new(Vec::new()),
        &pw::spa::pod::Value::Object(obj),
    )
    .map_err(|e| anyhow::anyhow!("Failed to serialize audio params: {:?}", e))?
    .0
    .into_inner();

    Ok(values)
}

/// Process audio buffer - extract samples from PipeWire buffer and convert stereo to mono
fn process_audio_buffer(
    stream: &pw::stream::Stream,
    audio_buffer: &Rc<RefCell<Vec<f32>>>,
) {
    if let Some(mut buffer) = stream.dequeue_buffer() {
        let datas = buffer.datas_mut();
        if let Some(data) = datas.first_mut() {
            let chunk = data.chunk();
            let offset = chunk.offset() as usize;
            let size = chunk.size() as usize;

            if let Some(slice) = data.data() {
                let actual_data = if offset + size <= slice.len() {
                    &slice[offset..offset + size]
                } else {
                    slice
                };

                let raw_samples = bytes_to_f32_samples(actual_data);
                // Convert stereo to mono (mix down)
                let samples: Vec<f32> = raw_samples
                    .chunks(2)
                    .map(|chunk| {
                        if chunk.len() == 2 {
                            (chunk[0] + chunk[1]) / 2.0
                        } else {
                            chunk[0]
                        }
                    })
                    .collect();
                if !samples.is_empty() {
                    audio_buffer.borrow_mut().extend_from_slice(&samples);
                    debug!("Captured {} mono samples", samples.len());
                }
            }
        }
    }
}

/// Perform post-capture resampling if needed
fn finalize_audio_samples(
    raw_samples: Vec<f32>,
    capture_duration: Duration,
) -> Result<Vec<f32>> {
    if raw_samples.is_empty() {
        warn!("No audio captured - check microphone permissions");
        return Ok(Vec::new());
    }

    // Calculate actual sample rate from captured data
    let actual_duration_secs = capture_duration.as_secs_f32();
    let detected_rate = (raw_samples.len() as f32 / actual_duration_secs) as u32;
    info!(
        "Captured {} samples in {:.2}s (detected ~{}Hz)",
        raw_samples.len(),
        actual_duration_secs,
        detected_rate
    );

    // Resample to 16kHz for Whisper
    let target_rate = 16000u32;
    let samples = if detected_rate > target_rate + 1000 || detected_rate < target_rate - 1000 {
        info!("Resampling from ~{}Hz to {}Hz", detected_rate, target_rate);
        resample(&raw_samples, detected_rate, target_rate)
    } else {
        raw_samples
    };

    let final_duration = samples.len() as f32 / 16000.0;
    info!(
        "Final audio: {} samples ({:.2}s at 16kHz)",
        samples.len(),
        final_duration
    );

    Ok(samples)
}

/// Capture audio from the default microphone
///
/// Returns f32 PCM samples at 16kHz mono (Whisper requirement)
pub fn capture(duration_secs: u32, _sample_rate: u32) -> Result<Vec<f32>> {
    info!(
        "Starting audio capture: {}s",
        if duration_secs == 0 {
            "unlimited".to_string()
        } else {
            duration_secs.to_string()
        }
    );

    let pw_ctx = init_pipewire()?;

    // Pre-allocate buffer based on expected duration (16kHz mono = 16000 samples/sec)
    let expected_samples = if duration_secs > 0 {
        16000 * duration_secs as usize
    } else {
        16000 * 30 // Default 30 seconds for unlimited mode
    };
    let audio_buffer: Rc<RefCell<Vec<f32>>> =
        Rc::new(RefCell::new(Vec::with_capacity(expected_samples)));
    let start_time: Rc<RefCell<Option<Instant>>> = Rc::new(RefCell::new(None));

    let audio_buffer_clone = audio_buffer.clone();
    let start_time_clone = start_time.clone();
    let mainloop_weak = pw_ctx.mainloop.downgrade();

    let props = create_stream_properties();
    let stream = pw::stream::StreamBox::new(&pw_ctx.core, "dev-voice", props)
        .context("Failed to create stream")?;

    // Set up listener for stream events
    let _listener = stream
        .add_local_listener_with_user_data(())
        .state_changed(|_, _, old, new| {
            info!("Stream state changed: {:?} -> {:?}", old, new);
        })
        .param_changed(|_, _, id, pod| {
            if let Some(_pod) = pod {
                if id == spa::param::ParamType::Format.as_raw() {
                    info!("Format negotiated");
                }
            }
        })
        .process(move |stream, _| {
            // Initialize start time on first process call
            {
                let mut start = start_time_clone.borrow_mut();
                if start.is_none() {
                    *start = Some(Instant::now());
                    info!("Recording started - speak now!");
                }
            }

            // Check duration limit
            if duration_secs > 0 {
                let start = start_time_clone.borrow();
                if let Some(start_instant) = *start {
                    if start_instant.elapsed() >= Duration::from_secs(duration_secs as u64) {
                        info!("Recording duration reached");
                        if let Some(ml) = mainloop_weak.upgrade() {
                            ml.quit();
                        }
                        return;
                    }
                }
            }

            process_audio_buffer(stream, &audio_buffer_clone);
        })
        .register()?;

    let values = build_audio_params()?;
    let mut params = [Pod::from_bytes(&values)
        .ok_or_else(|| anyhow::anyhow!("Failed to create Pod from serialized bytes"))?];

    stream
        .connect(
            spa::utils::Direction::Input,
            None,
            StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS,
            &mut params,
        )
        .context("Failed to connect stream")?;

    info!(
        "Listening for {}...",
        if duration_secs == 0 {
            "audio (press Ctrl+C to stop)".to_string()
        } else {
            format!("{} seconds", duration_secs)
        }
    );

    // Run the main loop
    let capture_start = Instant::now();
    pw_ctx.mainloop.run();
    let capture_duration = capture_start.elapsed();

    let raw_samples = audio_buffer.borrow().clone();
    finalize_audio_samples(raw_samples, capture_duration)
}

/// Capture audio in toggle mode - stops when signal received or timeout
pub fn capture_toggle(max_duration_secs: u32, _sample_rate: u32) -> Result<Vec<f32>> {
    use crate::state::toggle::should_stop;

    info!("Starting toggle mode capture (max {}s)", max_duration_secs);

    let pw_ctx = init_pipewire()?;

    // Pre-allocate for max duration
    let expected_samples = 16000 * max_duration_secs as usize;
    let audio_buffer: Rc<RefCell<Vec<f32>>> =
        Rc::new(RefCell::new(Vec::with_capacity(expected_samples)));
    let start_time: Rc<RefCell<Option<Instant>>> = Rc::new(RefCell::new(None));

    let audio_buffer_clone = audio_buffer.clone();
    let start_time_clone = start_time.clone();
    let mainloop_weak = pw_ctx.mainloop.downgrade();

    let props = create_stream_properties();
    let stream = pw::stream::StreamBox::new(&pw_ctx.core, "dev-voice", props)
        .context("Failed to create stream")?;

    let _listener = stream
        .add_local_listener_with_user_data(())
        .state_changed(|_, _, old, new| {
            info!("Stream state changed: {:?} -> {:?}", old, new);
        })
        .param_changed(|_, _, id, pod| {
            if let Some(_pod) = pod {
                if id == spa::param::ParamType::Format.as_raw() {
                    info!("Format negotiated");
                }
            }
        })
        .process(move |stream, _| {
            // Initialize start time on first process call
            {
                let mut start = start_time_clone.borrow_mut();
                if start.is_none() {
                    *start = Some(Instant::now());
                    info!("Recording started - speak now!");
                }
            }

            // Check for stop signal
            if should_stop() {
                info!("Stop signal received");
                if let Some(ml) = mainloop_weak.upgrade() {
                    ml.quit();
                }
                return;
            }

            // Check timeout
            let start = start_time_clone.borrow();
            if let Some(start_instant) = *start {
                if start_instant.elapsed() >= Duration::from_secs(max_duration_secs as u64) {
                    info!("Toggle mode timeout reached ({}s)", max_duration_secs);
                    if let Some(ml) = mainloop_weak.upgrade() {
                        ml.quit();
                    }
                    return;
                }
            }

            process_audio_buffer(stream, &audio_buffer_clone);
        })
        .register()?;

    let values = build_audio_params()?;
    let mut params = [Pod::from_bytes(&values)
        .ok_or_else(|| anyhow::anyhow!("Failed to create Pod from serialized bytes"))?];

    stream
        .connect(
            spa::utils::Direction::Input,
            None,
            StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS,
            &mut params,
        )
        .context("Failed to connect stream")?;

    info!("Listening... (run 'dev-voice stop' to finish)");

    // Add timer to check stop signal every 100ms with 1-second buffer
    let mainloop_for_timer = pw_ctx.mainloop.downgrade();
    let stop_time: Rc<RefCell<Option<Instant>>> = Rc::new(RefCell::new(None));
    let stop_time_clone = stop_time.clone();

    let timer = pw_ctx.mainloop.loop_().add_timer(move |_| {
        if should_stop() {
            let mut stop = stop_time_clone.borrow_mut();
            if stop.is_none() {
                *stop = Some(Instant::now());
                info!(
                    "Stop signal detected - recording for 1 more second to capture trailing audio"
                );
            } else if let Some(stop_instant) = *stop {
                if stop_instant.elapsed() >= Duration::from_secs(1) {
                    info!("Buffer period complete - shutting down");
                    if let Some(ml) = mainloop_for_timer.upgrade() {
                        ml.quit();
                    }
                }
            }
        }
    });
    timer.update_timer(
        Some(std::time::Duration::from_millis(100)),
        Some(std::time::Duration::from_millis(100)),
    );

    let capture_start = Instant::now();
    pw_ctx.mainloop.run();
    let capture_duration = capture_start.elapsed();

    let raw_samples = audio_buffer.borrow().clone();
    finalize_audio_samples(raw_samples, capture_duration)
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

/// Convert raw bytes to f32 samples
/// Handles both f32 and i16 formats
fn bytes_to_f32_samples(bytes: &[u8]) -> Vec<f32> {
    // Try interpreting as f32 first (most common with PipeWire)
    if bytes.len() % 4 == 0 {
        let (prefix, samples, suffix) = unsafe { bytes.align_to::<f32>() };
        if prefix.is_empty() && suffix.is_empty() {
            return samples.to_vec();
        }
    }

    // Fall back to i16 interpretation
    if bytes.len() % 2 == 0 {
        let (prefix, samples, suffix) = unsafe { bytes.align_to::<i16>() };
        if prefix.is_empty() && suffix.is_empty() {
            return samples
                .iter()
                .map(|&s| s as f32 / i16::MAX as f32)
                .collect();
        }
    }

    // Unable to interpret
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_f32_from_f32() {
        let floats: Vec<f32> = vec![0.5, -0.5, 1.0, -1.0];
        let bytes: &[u8] =
            unsafe { std::slice::from_raw_parts(floats.as_ptr() as *const u8, floats.len() * 4) };
        let result = bytes_to_f32_samples(bytes);
        assert_eq!(result, floats);
    }

    #[test]
    fn test_bytes_to_f32_from_i16() {
        let samples: Vec<i16> = vec![0, i16::MAX, i16::MIN];
        let bytes: &[u8] =
            unsafe { std::slice::from_raw_parts(samples.as_ptr() as *const u8, samples.len() * 2) };
        let result = bytes_to_f32_samples(bytes);
        assert_eq!(result.len(), 3);
        assert!((result[0] - 0.0).abs() < 0.001);
        assert!((result[1] - 1.0).abs() < 0.001);
    }
}
