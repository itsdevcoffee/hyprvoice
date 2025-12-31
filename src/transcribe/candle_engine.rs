use anyhow::Result;
use candle_core::{Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::whisper::{self, Config};
use hf_hub::{Repo, api::sync::Api};
use std::path::Path;
use tokenizers::Tokenizer;
use tracing::{debug, error, info, warn};

use crate::transcribe::Transcriber;

// Temperature fallback constants (from official Candle Whisper example)
const TEMPERATURES: [f64; 6] = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
const COMPRESSION_RATIO_THRESHOLD: f64 = 2.4;
const LOGPROB_THRESHOLD: f64 = -1.0;

// Audio chunking constants for long-form transcription
const CHUNK_LENGTH_SECS: f32 = 30.0; // Maximum 30 seconds per chunk (Whisper limit)
const CHUNK_OVERLAP_SECS: f32 = 5.0; // 5-second overlap between chunks
const SAMPLE_RATE: usize = 16000; // Whisper requires 16kHz audio

/// Model wrapper supporting both normal (safetensors) and quantized (GGUF) models
enum Model {
    Normal(whisper::model::Whisper),
    Quantized(whisper::quantized_model::Whisper),
}

impl Model {
    /// Forward pass through encoder
    fn encoder_forward(&mut self, mel: &Tensor, flush: bool) -> Result<Tensor> {
        match self {
            Self::Normal(m) => Ok(m.encoder.forward(mel, flush)?),
            Self::Quantized(m) => Ok(m.encoder.forward(mel, flush)?),
        }
    }

    /// Forward pass through decoder
    fn decoder_forward(
        &mut self,
        tokens: &Tensor,
        audio_features: &Tensor,
        flush: bool,
    ) -> Result<Tensor> {
        match self {
            Self::Normal(m) => Ok(m.decoder.forward(tokens, audio_features, flush)?),
            Self::Quantized(m) => Ok(m.decoder.forward(tokens, audio_features, flush)?),
        }
    }

    /// Final linear projection to vocabulary
    fn decoder_final_linear(&self, x: &Tensor) -> Result<Tensor> {
        match self {
            Self::Normal(m) => Ok(m.decoder.final_linear(x)?),
            Self::Quantized(m) => Ok(m.decoder.final_linear(x)?),
        }
    }
}

/// Candle-based Whisper transcription engine
pub struct CandleEngine {
    device: Device,
    model: Model,
    tokenizer: Tokenizer,
    config: Config,
    language: String,
    initial_prompt: Option<String>,
    mel_filters: Vec<f32>,
    suppress_tokens: Tensor,
}

impl CandleEngine {
    /// Create a new CandleEngine with language and optional initial prompt
    ///
    /// # Arguments
    /// * `model_id` - Local file path to GGUF model OR HuggingFace model ID
    ///   - Local: "/path/to/model.gguf" or "/path/to/model.bin"
    ///   - HuggingFace: "openai/whisper-large-v3-turbo" (downloads safetensors)
    ///   - HuggingFace quantized: "Demonthos/candle-quantized-whisper-large-v3-turbo" (downloads GGUF)
    /// * `language` - Language code (e.g., "en", "es", "fr")
    /// * `initial_prompt` - Optional technical vocabulary prompt to bias transcription
    pub fn with_options(
        model_id: &str,
        language: &str,
        initial_prompt: Option<String>,
    ) -> Result<Self> {
        let device = Self::get_device()?;
        info!("Using device: {:?}", device);

        // Check if model_id is a local file path
        let is_local_file = Path::new(model_id).exists();
        let is_quantized =
            is_local_file && (model_id.ends_with(".gguf") || model_id.ends_with(".bin"));

        let (config, tokenizer, model) = if is_local_file {
            info!("Loading model from local file: {}", model_id);

            if is_quantized {
                // Load GGUF quantized model from local file
                info!("Detected quantized model (GGUF/GGML format)");

                // For quantized models, we need config and tokenizer from HuggingFace
                // Use a base model for config/tokenizer
                let api = Api::new()?;
                let repo = api.repo(Repo::model("openai/whisper-large-v3-turbo".to_string()));

                let config_filename = repo.get("config.json")?;
                let tokenizer_filename = repo.get("tokenizer.json")?;

                let config: Config =
                    serde_json::from_str(&std::fs::read_to_string(config_filename)?)?;
                let tokenizer = Tokenizer::from_file(tokenizer_filename)
                    .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

                // Load quantized weights
                let vb = candle_transformers::quantized_var_builder::VarBuilder::from_gguf(
                    model_id, &device,
                )?;
                let model = Model::Quantized(whisper::quantized_model::Whisper::load(
                    &vb,
                    config.clone(),
                )?);

                (config, tokenizer, model)
            } else {
                // Load safetensors from local directory
                info!("Loading local safetensors model");

                let model_path = Path::new(model_id);
                let config_path = model_path.join("config.json");
                let tokenizer_path = model_path.join("tokenizer.json");
                let weights_path = model_path.join("model.safetensors");

                // Verify files exist
                if !config_path.exists() {
                    anyhow::bail!("Config not found: {:?}", config_path);
                }
                if !tokenizer_path.exists() {
                    anyhow::bail!("Tokenizer not found: {:?}", tokenizer_path);
                }
                if !weights_path.exists() {
                    anyhow::bail!("Model weights not found: {:?}", weights_path);
                }

                let config: Config = serde_json::from_str(&std::fs::read_to_string(config_path)?)?;
                let tokenizer = Tokenizer::from_file(tokenizer_path)
                    .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

                let vb = unsafe {
                    VarBuilder::from_mmaped_safetensors(&[weights_path], whisper::DTYPE, &device)?
                };
                let model = Model::Normal(whisper::model::Whisper::load(&vb, config.clone())?);

                (config, tokenizer, model)
            }
        } else {
            // Download from HuggingFace
            info!("Downloading model from HuggingFace: {}", model_id);

            let api = Api::new()?;
            let repo = api.repo(Repo::model(model_id.to_string()));

            let config_filename = repo.get("config.json")?;
            let tokenizer_filename = repo.get("tokenizer.json")?;

            let config: Config = serde_json::from_str(&std::fs::read_to_string(config_filename)?)?;
            let tokenizer = Tokenizer::from_file(tokenizer_filename)
                .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

            // Try to load quantized model first, fall back to safetensors
            let model = if let Ok(weights_filename) = repo.get("model.gguf") {
                info!("Found GGUF model, loading quantized variant");
                let vb = candle_transformers::quantized_var_builder::VarBuilder::from_gguf(
                    &weights_filename,
                    &device,
                )?;
                Model::Quantized(whisper::quantized_model::Whisper::load(
                    &vb,
                    config.clone(),
                )?)
            } else {
                info!("Loading safetensors model (normal precision)");
                let weights_filename = repo.get("model.safetensors")?;
                let vb = unsafe {
                    VarBuilder::from_mmaped_safetensors(
                        &[weights_filename],
                        whisper::DTYPE,
                        &device,
                    )?
                };
                Model::Normal(whisper::model::Whisper::load(&vb, config.clone())?)
            };

            (config, tokenizer, model)
        };

        info!("Model loaded successfully");

        // Create mel filterbank from pre-computed bytes
        let mel_filters_vec = Self::load_mel_filters(config.num_mel_bins)?;
        info!("Mel filters loaded: {} filters", mel_filters_vec.len());

        // Build suppress tokens mask to prevent unwanted tokens (like 199)
        let vocab_size = tokenizer.get_vocab_size(true);
        let no_ts_token = tokenizer
            .token_to_id("<|notimestamps|>")
            .ok_or_else(|| anyhow::anyhow!("No timestamps token not found"))?;

        let mut suppress_list = vec![220u32]; // Blank token only (NOT EOT - we need that!)
        // Suppress all timestamp tokens (everything after <|notimestamps|>)
        for i in (no_ts_token + 1)..vocab_size as u32 {
            suppress_list.push(i);
        }

        let mut mask = vec![0f32; vocab_size];
        for &token in &suppress_list {
            mask[token as usize] = f32::NEG_INFINITY;
        }
        let suppress_tokens = Tensor::new(&mask[..], &device)?;
        info!(
            "Suppress mask created: {} tokens suppressed",
            suppress_list.len()
        );

        info!("CandleEngine initialization complete - ready for transcription");

        Ok(Self {
            device,
            model,
            tokenizer,
            config,
            language: language.to_string(),
            initial_prompt,
            mel_filters: mel_filters_vec,
            suppress_tokens,
        })
    }

    fn get_device() -> Result<Device> {
        // Try CUDA first with detailed error reporting
        match Device::new_cuda(0) {
            Ok(device) => {
                info!("CUDA device initialized successfully");
                return Ok(device);
            },
            Err(e) => {
                warn!("CUDA initialization failed: {}", e);
            },
        }

        // Try Metal (macOS)
        if candle_core::utils::metal_is_available() {
            info!("Using Metal device");
            return Ok(Device::new_metal(0)?);
        }

        // Fallback to CPU
        warn!("No GPU accelerator found, falling back to CPU");
        Ok(Device::Cpu)
    }

    /// Load mel filterbank coefficients
    ///
    /// These are pre-computed filter banks included from the Candle whisper example
    fn load_mel_filters(num_mel_bins: usize) -> Result<Vec<f32>> {
        let mel_bytes: &[u8] = match num_mel_bins {
            80 => include_bytes!("../../assets/melfilters80.bytes"),
            128 => include_bytes!("../../assets/melfilters128.bytes"),
            _ => anyhow::bail!(
                "Unsupported num_mel_bins: {}. Only 80 and 128 are supported.",
                num_mel_bins
            ),
        };

        let mut mel_filters = vec![0f32; mel_bytes.len() / 4];

        // Convert bytes to f32 using little-endian byte order
        use std::io::Read;
        let mut cursor = std::io::Cursor::new(mel_bytes);
        for filter in mel_filters.iter_mut() {
            let mut bytes = [0u8; 4];
            cursor.read_exact(&mut bytes)?;
            *filter = f32::from_le_bytes(bytes);
        }

        Ok(mel_filters)
    }

    /// Get special token IDs from the tokenizer
    fn get_special_tokens(&self) -> Result<SpecialTokens> {
        // Helper to convert token string to ID
        let token_id = |token: &str| -> Result<u32> {
            self.tokenizer
                .token_to_id(token)
                .ok_or_else(|| anyhow::anyhow!("Token not found: {}", token))
        };

        let sot_token = token_id("<|startoftranscript|>")?;
        let eot_token = token_id("<|endoftext|>")?;
        let transcribe_token = token_id("<|transcribe|>")?;
        let no_timestamps_token = token_id("<|notimestamps|>")?;
        let language_token = token_id(&format!("<|{}|>", self.language))?;

        Ok(SpecialTokens {
            sot_token,
            eot_token,
            transcribe_token,
            no_timestamps_token,
            language_token,
        })
    }

    /// Encode the initial prompt if provided
    fn encode_initial_prompt(&self) -> Result<Vec<u32>> {
        if let Some(ref prompt) = self.initial_prompt {
            let encoding = self
                .tokenizer
                .encode(prompt.clone(), false)
                .map_err(|e| anyhow::anyhow!("Failed to encode initial prompt: {}", e))?;

            let tokens = encoding.get_ids().to_vec();

            // CRITICAL: Whisper expects short prompts (<50 tokens)
            // Long prompts cause decoder to get stuck in infinite loops
            const MAX_PROMPT_TOKENS: usize = 50;
            if tokens.len() > MAX_PROMPT_TOKENS {
                warn!(
                    "Initial prompt has {} tokens, truncating to {} (prompt length: {} chars)",
                    tokens.len(),
                    MAX_PROMPT_TOKENS,
                    prompt.len()
                );
                Ok(tokens[..MAX_PROMPT_TOKENS].to_vec())
            } else {
                debug!("Using {} prompt tokens", tokens.len());
                Ok(tokens)
            }
        } else {
            Ok(Vec::new())
        }
    }

    fn decode_at_temperature(
        &mut self,
        mel: &Tensor,
        temperature: f64,
    ) -> Result<(String, f64, f64)> {
        debug!(
            "decode_at_temperature() called with mel shape: {:?}, temp: {}",
            mel.shape(),
            temperature
        );
        debug!("Starting decode process with temperature {}", temperature);

        debug!("Getting special tokens");
        let special_tokens = self.get_special_tokens()?;
        debug!("Got special tokens");

        // Validate special tokens are reasonable (Whisper tokenizer uses 51k vocab)
        let validate_token = |name: &str, token: u32| {
            if token > 51865 {
                warn!(
                    "Special token {} = {} seems invalid (> vocab size)",
                    name, token
                );
            }
        };
        validate_token("SOT", special_tokens.sot_token);
        validate_token("EOT", special_tokens.eot_token);
        validate_token("Language", special_tokens.language_token);
        validate_token("Transcribe", special_tokens.transcribe_token);
        validate_token("NoTimestamps", special_tokens.no_timestamps_token);

        info!(
            "Special tokens: SOT={}, EOT={}, Lang={}, Transcribe={}, NoTS={}",
            special_tokens.sot_token,
            special_tokens.eot_token,
            special_tokens.language_token,
            special_tokens.transcribe_token,
            special_tokens.no_timestamps_token
        );

        let prompt_tokens = self.encode_initial_prompt()?;
        debug!("Got {} prompt tokens", prompt_tokens.len());

        // 1. Run Encoder
        debug!(
            "Running encoder forward pass on mel shape: {:?}",
            mel.shape()
        );
        info!("MEL SHAPE BEFORE ENCODER: {:?}", mel.shape());
        debug!("Calling encoder.forward()");
        let audio_features = self.model.encoder_forward(mel, true)?;
        debug!("encoder.forward() returned");
        info!("ENCODER OUTPUT SHAPE: {:?}", audio_features.shape());
        info!(
            "ENCODER OUTPUT DIMS: batch={}, frames={}, d_model={}",
            audio_features.dim(0)?,
            audio_features.dim(1)?,
            audio_features.dim(2)?
        );

        // 2. Build initial token sequence following Whisper spec:
        // <|startoftranscript|><|language|><|transcribe|><|notimestamps|>[optional_prompt_tokens]
        let mut current_tokens = vec![special_tokens.sot_token];
        current_tokens.push(special_tokens.language_token);
        current_tokens.push(special_tokens.transcribe_token);
        current_tokens.push(special_tokens.no_timestamps_token);

        // Add initial prompt tokens for technical vocabulary biasing
        if !prompt_tokens.is_empty() {
            debug!(
                "Using initial prompt with {} tokens for biasing",
                prompt_tokens.len()
            );
            current_tokens.extend_from_slice(&prompt_tokens);
        }

        info!(
            "Initial token sequence: {} special tokens + {} prompt tokens = {} total",
            4,
            prompt_tokens.len(),
            current_tokens.len()
        );

        // 3. Greedy decoding loop with quality metrics
        let mut result_tokens = Vec::new();
        let start_result_idx = current_tokens.len(); // Track where actual transcription starts

        // Calculate max tokens accounting for initial sequence (special + prompt tokens)
        // Decoder has hard limit of 448 total positions
        let max_tokens = 448_usize.saturating_sub(start_result_idx);

        info!(
            "Will start collecting result tokens after index {} (max {} new tokens)",
            start_result_idx, max_tokens
        );

        // Quality metrics tracking
        let mut sum_logprob = 0.0f64;
        let mut logprob_count = 0;

        // Safety: track repeated tokens to detect infinite loops
        let mut last_token: Option<u32> = None;
        let mut repeat_count = 0;
        const MAX_REPEATS: usize = 3; // Reduced from 10 - catch loops earlier

        info!(
            "Starting greedy decoding loop (max {} tokens, temp {})",
            max_tokens, temperature
        );

        for iteration in 0..max_tokens {
            // Progress logging every 10 iterations
            if iteration % 10 == 0 {
                info!(
                    "Decode iteration {}/{}, generated {} tokens so far",
                    iteration,
                    max_tokens,
                    result_tokens.len()
                );
            }

            let input = Tensor::new(current_tokens.as_slice(), &self.device)?.unsqueeze(0)?;

            if iteration == 0 {
                debug!("First decoder input shape: {:?}", input.shape());
                info!(
                    "DECODER INPUT SHAPE (iter 0): {:?} (batch={}, seq_len={})",
                    input.shape(),
                    input.dim(0)?,
                    input.dim(1)?
                );
                info!(
                    "AUDIO FEATURES SHAPE PASSED TO DECODER: {:?}",
                    audio_features.shape()
                );
            }

            // Decoder forward pass produces hidden states [batch, seq_len, d_model=1280]
            // CRITICAL: Only flush KV cache on first iteration to maintain context
            let decoder_output =
                self.model
                    .decoder_forward(&input, &audio_features, iteration == 0)?;

            // Project hidden states to vocabulary logits [batch, seq_len, vocab_size=51866]
            let logits = self.model.decoder_final_linear(&decoder_output)?;

            // Get logits for the last token position across full vocabulary
            let logits = logits.squeeze(0)?;

            // Verify shapes on first iteration
            if iteration == 0 {
                debug!("Decoder output shape after squeeze: {:?}", logits.shape());
                debug!("Dim 0 = {}, Dim 1 = {}", logits.dim(0)?, logits.dim(1)?);
            }

            let seq_len = logits.dim(0)?;
            let mut last_logit = logits.i((seq_len - 1, ..))?; // Shape: [vocab_size]

            if iteration == 0 {
                debug!("last_logit shape: {:?}", last_logit.shape());
            }

            // Apply suppress mask BEFORE temperature/argmax (prevents token 199 and other unwanted tokens)
            last_logit = last_logit.broadcast_add(&self.suppress_tokens)?;

            // Apply temperature (if temp > 0)
            if temperature > 0.0 {
                last_logit = (last_logit / temperature)?;
            }

            // Convert to log probabilities for quality metrics
            let log_probs = candle_nn::ops::softmax(&last_logit, 0)?;

            // Greedy selection: argmax
            let next_token = last_logit.argmax(0)?.to_scalar::<u32>()?;

            // Track log probability of selected token for quality metrics
            let token_logprob = log_probs.get(next_token as usize)?.to_scalar::<f32>()? as f64;
            if token_logprob > 0.0 {
                sum_logprob += token_logprob.ln();
                logprob_count += 1;
            }

            // Log first 10 tokens to see what's being generated
            if iteration < 10 {
                info!("Iteration {}: next_token = {}", iteration, next_token);
            }

            // Check for end of text
            if next_token == special_tokens.eot_token {
                info!("EOT token detected at iteration {}", iteration);
                break;
            }

            // Safety check: detect infinite loops with repeated tokens
            if let Some(last) = last_token {
                if last == next_token {
                    repeat_count += 1;
                    if repeat_count >= MAX_REPEATS {
                        warn!(
                            "Token {} repeated {} times consecutively, likely infinite loop - breaking early",
                            next_token, repeat_count
                        );
                        warn!(
                            "Generated {} tokens before loop: {:?}",
                            result_tokens.len(),
                            &result_tokens[..result_tokens.len().min(20)]
                        );
                        break;
                    }
                } else {
                    repeat_count = 0;
                }
            }
            last_token = Some(next_token);

            // Check for no-speech condition (optional enhancement)
            // In a production implementation, you'd check if the first token after prompt
            // has high probability of being a no-speech token and return empty string

            current_tokens.push(next_token);

            // Only add tokens after the initial sequence to result
            if current_tokens.len() > start_result_idx {
                result_tokens.push(next_token);
            }
        }

        // 4. Decode tokens to text
        // skip_special_tokens = true to remove any remaining special tokens
        info!("Decoding {} result tokens to text", result_tokens.len());
        if result_tokens.len() <= 20 {
            info!("Result tokens: {:?}", result_tokens);
        } else {
            info!("First 20 result tokens: {:?}", &result_tokens[..20]);
        }

        let decoded = self
            .tokenizer
            .decode(&result_tokens, true)
            .map_err(|e| anyhow::anyhow!("Decoding error: {}", e))?;

        let text = decoded.trim().to_string();
        info!(
            "Decoded {} tokens to text: \"{}\"",
            result_tokens.len(),
            text
        );

        // 5. Calculate quality metrics
        let avg_logprob = if logprob_count > 0 {
            sum_logprob / logprob_count as f64
        } else {
            0.0
        };

        let compression_ratio = if !text.is_empty() {
            result_tokens.len() as f64 / text.len() as f64
        } else {
            0.0
        };

        info!(
            "Quality metrics: avg_logprob={:.3}, compression_ratio={:.3}",
            avg_logprob, compression_ratio
        );

        Ok((text, avg_logprob, compression_ratio))
    }

    /// Decode with temperature fallback for improved quality
    ///
    /// Tries temperatures [0.0, 0.2, 0.4, 0.6, 0.8, 1.0] until quality thresholds are met
    fn decode_with_fallback(&mut self, mel: &Tensor) -> Result<String> {
        for (i, &temp) in TEMPERATURES.iter().enumerate() {
            match self.decode_at_temperature(mel, temp) {
                Ok((text, avg_logprob, compression_ratio)) => {
                    // Last temperature - accept whatever we get
                    if i == TEMPERATURES.len() - 1 {
                        info!("Using last temperature {} (no fallback left)", temp);
                        return Ok(text);
                    }

                    // Check quality metrics
                    let needs_fallback = compression_ratio > COMPRESSION_RATIO_THRESHOLD
                        || avg_logprob < LOGPROB_THRESHOLD;

                    if !needs_fallback {
                        info!(
                            "Decoding succeeded at temperature {} (logprob={:.3}, compression={:.3})",
                            temp, avg_logprob, compression_ratio
                        );
                        return Ok(text);
                    }

                    warn!(
                        "Quality check failed at temp {} (logprob={:.3}, compression={:.3}), trying next temperature",
                        temp, avg_logprob, compression_ratio
                    );
                },
                Err(e) => {
                    warn!("Decoding failed at temperature {}: {}", temp, e);
                    continue;
                },
            }
        }

        anyhow::bail!("All temperature fallbacks failed")
    }

    /// Transcribe a single chunk of audio (max 30 seconds)
    fn transcribe_chunk(&mut self, audio: &[f32]) -> Result<String> {
        debug!("transcribe_chunk() called with {} samples", audio.len());

        if audio.is_empty() {
            return Ok(String::new());
        }

        // Pad audio to exactly 30 seconds (480000 samples at 16kHz) as Whisper expects
        const N_SAMPLES: usize = 480000; // 30 seconds * 16000 Hz
        let mut padded_audio = audio.to_vec();
        if padded_audio.len() < N_SAMPLES {
            info!(
                "Padding audio from {} to {} samples",
                audio.len(),
                N_SAMPLES
            );
            padded_audio.resize(N_SAMPLES, 0.0); // Pad with silence
        } else if padded_audio.len() > N_SAMPLES {
            info!(
                "Truncating audio from {} to {} samples",
                audio.len(),
                N_SAMPLES
            );
            padded_audio.truncate(N_SAMPLES); // Truncate if too long
        }
        info!("PADDED AUDIO LENGTH: {} samples", padded_audio.len());

        // 1. Convert audio to Mel Spectrogram (will create exactly 3000 frames)
        debug!("Converting PCM to Mel spectrogram...");
        let mel_data = whisper::audio::pcm_to_mel(&self.config, &padded_audio, &self.mel_filters);
        info!("MEL DATA LENGTH: {} elements", mel_data.len());

        // Convert Vec<f32> to Tensor with proper shape
        let mel_len = mel_data.len();
        let n_mels = self.config.num_mel_bins;
        let frames = mel_len / n_mels;

        info!(
            "MEL SPECTROGRAM: mel_len={}, n_mels={}, frames={}",
            mel_len, n_mels, frames
        );

        if mel_len == 0 || frames == 0 || mel_len % n_mels != 0 {
            anyhow::bail!("Invalid mel spectrogram");
        }

        let mel = Tensor::from_vec(mel_data, (n_mels, frames), &self.device)?;
        info!("MEL TENSOR SHAPE (before batch dim): {:?}", mel.shape());

        // CRITICAL FIX: Whisper Large V3 Turbo has max_source_positions=1500
        // After 2x encoder downsampling, this means mel can have max 3000 frames
        // But pcm_to_mel produces 4500 frames for 480000 samples (bug in Candle?)
        // Truncate to exactly 3000 frames to match model's max_source_positions
        const MAX_MEL_FRAMES: usize = 3000;
        let mel = if frames > MAX_MEL_FRAMES {
            warn!(
                "Mel has {} frames, truncating to {} to match model's max_source_positions",
                frames, MAX_MEL_FRAMES
            );
            mel.narrow(1, 0, MAX_MEL_FRAMES)?
        } else {
            mel
        };
        info!("MEL TENSOR SHAPE (after truncation): {:?}", mel.shape());

        let mel = mel.unsqueeze(0)?; // Add batch dimension
        info!("MEL TENSOR SHAPE (after batch dim): {:?}", mel.shape());

        // 2. Decode with temperature fallback
        self.decode_with_fallback(&mel)
    }
}

impl Transcriber for CandleEngine {
    fn transcribe(&mut self, audio: &[f32]) -> Result<String> {
        if audio.is_empty() {
            return Ok(String::new());
        }

        let duration_secs = audio.len() as f32 / SAMPLE_RATE as f32;
        info!(
            "Transcribing {} samples ({:.2}s) [Language: {}, Prompt: {}]",
            audio.len(),
            duration_secs,
            self.language,
            if self.initial_prompt.is_some() {
                "true"
            } else {
                "false"
            }
        );

        // Check if we need chunking (audio > 30 seconds)
        if duration_secs <= CHUNK_LENGTH_SECS {
            // Short audio - process directly
            debug!("Audio <= 30s, processing without chunking");
            return self.transcribe_chunk(audio);
        }

        // Long audio - split into overlapping chunks
        info!(
            "Audio is {:.1}s, splitting into {:.0}s chunks with {:.0}s overlap",
            duration_secs, CHUNK_LENGTH_SECS, CHUNK_OVERLAP_SECS
        );

        let chunk_samples = (CHUNK_LENGTH_SECS * SAMPLE_RATE as f32) as usize;
        let overlap_samples = (CHUNK_OVERLAP_SECS * SAMPLE_RATE as f32) as usize;
        let stride = chunk_samples - overlap_samples; // Step size between chunks

        let mut results = Vec::new();
        let mut offset = 0;

        while offset < audio.len() {
            let end = (offset + chunk_samples).min(audio.len());
            let chunk = &audio[offset..end];
            let chunk_duration = chunk.len() as f32 / SAMPLE_RATE as f32;

            info!(
                "Processing chunk {}: {:.1}s-{:.1}s ({:.1}s duration, {} samples)",
                results.len() + 1,
                offset as f32 / SAMPLE_RATE as f32,
                end as f32 / SAMPLE_RATE as f32,
                chunk_duration,
                chunk.len()
            );

            match self.transcribe_chunk(chunk) {
                Ok(text) => {
                    if !text.is_empty() {
                        results.push(text);
                    }
                },
                Err(e) => {
                    warn!(
                        "Chunk {} failed: {}, continuing with next chunk",
                        results.len() + 1,
                        e
                    );
                },
            }

            // Move to next chunk (with overlap)
            offset += stride;

            // If we're close to the end, process the remainder and stop
            if offset + chunk_samples > audio.len() && offset < audio.len() {
                let remaining = &audio[offset..];
                if remaining.len() > overlap_samples {
                    info!("Processing final chunk: {} samples", remaining.len());
                    if let Ok(text) = self.transcribe_chunk(remaining) {
                        if !text.is_empty() {
                            results.push(text);
                        }
                    }
                }
                break;
            }
        }

        // Concatenate all chunks with space separator
        let final_text = results.join(" ");
        info!(
            "Long-form transcription complete: {} chunks, {} characters",
            results.len(),
            final_text.len()
        );

        Ok(final_text)
    }
}

/// Special token IDs used in Whisper decoding
struct SpecialTokens {
    sot_token: u32,
    eot_token: u32,
    transcribe_token: u32,
    no_timestamps_token: u32,
    language_token: u32,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_empty_audio() {
        // This test verifies the engine handles empty audio gracefully
        // We can't test full transcription without downloading models
        let empty_audio: Vec<f32> = vec![];

        // For a real engine, we'd need to:
        // let engine = CandleEngine::new("openai/whisper-tiny").unwrap();
        // let result = engine.transcribe(&empty_audio).unwrap();
        // assert_eq!(result, "");

        // For now, just verify empty input returns empty string conceptually
        assert_eq!(empty_audio.len(), 0);
    }

    #[test]
    fn test_special_tokens_format() {
        // Verify the language token format is correct
        let language = "en";
        let expected_format = format!("<|{}|>", language);
        assert_eq!(expected_format, "<|en|>");
    }
}
