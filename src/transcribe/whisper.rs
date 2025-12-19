use anyhow::{Context, Result};
use std::path::Path;
use tracing::{debug, info};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct Transcriber {
    ctx: WhisperContext,
    draft_ctx: Option<WhisperContext>,
    language: String,
    prompt: Option<String>,
}

impl Transcriber {
    /// Create a new transcriber with the given model path
    pub fn new(model_path: &Path) -> Result<Self> {
        Self::with_language(model_path, None, "en", None)
    }

    /// Create a new transcriber with an optional draft model for speculative decoding
    pub fn with_draft(
        model_path: &Path,
        draft_model_path: Option<&Path>,
        prompt: Option<String>,
    ) -> Result<Self> {
        Self::with_language(model_path, draft_model_path, "en", prompt)
    }

    /// Create a new transcriber with a specific language and optional draft model
    pub fn with_language(
        model_path: &Path,
        draft_model_path: Option<&Path>,
        language: &str,
        prompt: Option<String>,
    ) -> Result<Self> {
        let params = WhisperContextParameters::default();

        let ctx = WhisperContext::new_with_params(
            model_path.to_str().context("Invalid model path encoding")?,
            params,
        )
        .context("Failed to load target whisper model")?;

        let draft_ctx = if let Some(path) = draft_model_path {
            if path.exists() {
                info!("Loading draft model for speculative decoding: {}", path.display());
                Some(
                    WhisperContext::new_with_params(
                        path.to_str().context("Invalid draft model path encoding")?,
                        params,
                    )
                    .context("Failed to load draft whisper model")?,
                )
            } else {
                info!("Draft model path does not exist, skipping speculative decoding");
                None
            }
        } else {
            None
        };

        Ok(Self {
            ctx,
            draft_ctx,
            language: language.to_string(),
            prompt,
        })
    }

    /// Transcribe audio data to text
    ///
    /// Audio must be:
    /// - 16kHz sample rate
    /// - Mono channel
    /// - f32 PCM format
    pub fn transcribe(&self, audio: &[f32]) -> Result<String> {
        if audio.is_empty() {
            return Ok(String::new());
        }

        debug!(
            "Transcribing {} samples ({:.2}s) [Speculative: {}, Prompt: {}]",
            audio.len(),
            audio.len() as f32 / 16000.0,
            self.draft_ctx.is_some(),
            self.prompt.is_some()
        );

        let mut state = self
            .ctx
            .create_state()
            .context("Failed to create whisper state")?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Enable speculative decoding if draft model is available
        if let Some(ref d_ctx) = self.draft_ctx {
            params.set_encoder_begin_callback(d_ctx);
        }

        // Apply technical vocabulary prompt if available
        if let Some(ref prompt) = self.prompt {
            params.set_initial_prompt(prompt);
        }

        // Configure for dictation use case
        params.set_language(Some(&self.language));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_no_context(true);
        params.set_single_segment(false);

        // Run inference
        state
            .full(params, audio)
            .context("Whisper inference failed")?;

        // Get segment count - returns i32 directly
        let num_segments = state.full_n_segments();
        debug!("Got {} segments", num_segments);

        // Collect all segments using get_segment() and to_str()
        let mut result = String::new();
        for i in 0..num_segments {
            if let Some(segment) = state.get_segment(i) {
                if let Ok(text) = segment.to_str() {
                    result.push_str(text);
                }
            }
        }

        let text = result.trim().to_string();
        info!("Transcribed: \"{}\"", text);

        Ok(text)
    }
}

/// Convert i16 audio samples to f32 (normalized to -1.0 to 1.0)
#[cfg(test)]
pub fn convert_i16_to_f32(samples: &[i16]) -> Vec<f32> {
    samples
        .iter()
        .map(|&s| s as f32 / i16::MAX as f32)
        .collect()
}

/// Convert stereo audio to mono by averaging channels
#[cfg(test)]
pub fn convert_stereo_to_mono(stereo: &[f32]) -> Vec<f32> {
    stereo
        .chunks(2)
        .map(|chunk| {
            if chunk.len() == 2 {
                (chunk[0] + chunk[1]) / 2.0
            } else {
                chunk[0]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i16_to_f32_conversion() {
        let samples: Vec<i16> = vec![0, i16::MAX, i16::MIN];
        let converted = convert_i16_to_f32(&samples);

        assert!((converted[0] - 0.0).abs() < 0.001);
        assert!((converted[1] - 1.0).abs() < 0.001);
        assert!((converted[2] - (-1.0)).abs() < 0.01);
    }

    #[test]
    fn test_stereo_to_mono() {
        let stereo = vec![0.5, 0.3, 0.8, 0.2];
        let mono = convert_stereo_to_mono(&stereo);

        assert_eq!(mono.len(), 2);
        assert!((mono[0] - 0.4).abs() < 0.001);
        assert!((mono[1] - 0.5).abs() < 0.001);
    }
}
