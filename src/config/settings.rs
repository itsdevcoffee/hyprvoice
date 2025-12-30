use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const APP_NAME: &str = "hyprvoice";

const DEFAULT_PROMPT: &str = "async, await, impl, struct, enum, pub, static, btreemap, hashmap, kubernetes, k8s, docker, container, pod, lifecycle, workflow, ci/cd, yaml, json, rustlang, python, javascript, typescript, bash, git, repo, branch, commit, push, pull, merge, rebase, upstream, downstream, middleware, database, sql, postgres, redis, api, endpoint, graphql, rest, grpc, protobuf, systemd, journalctl, flatpak, wayland, nix, cargo.";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub model: ModelConfig,
    pub audio: AudioConfig,
    pub output: OutputConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Path to whisper model file (legacy, kept for backwards compatibility)
    pub path: PathBuf,
    /// HuggingFace model ID for Candle engine (e.g., "openai/whisper-large-v3-turbo")
    pub model_id: String,
    /// Path to optional draft model file for speculative decoding
    pub draft_model_path: Option<PathBuf>,
    /// Language code (e.g., "en")
    pub language: String,
    /// Optional prompt to bias the model vocabulary (technical terms)
    pub prompt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Sample rate in Hz (whisper requires 16000)
    pub sample_rate: u32,
    /// Recording timeout in seconds (0 = no timeout)
    pub timeout_secs: u32,
    /// Save audio recordings to disk
    #[serde(default)]
    pub save_audio_clips: bool,
    /// Directory to save audio clips (WAV format with timestamps)
    #[serde(default = "default_audio_clips_path")]
    pub audio_clips_path: PathBuf,
}

fn default_audio_clips_path() -> PathBuf {
    directories::BaseDirs::new()
        .map(|dirs| dirs.data_local_dir().join(APP_NAME).join("recordings"))
        .unwrap_or_else(|| PathBuf::from("./recordings"))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Force display server type: "wayland", "x11", or null for auto-detect
    pub display_server: Option<String>,
    /// Add a space after injected text
    pub append_space: bool,
    /// Command to refresh status bar UI (e.g., "pkill -RTMIN+8 waybar")
    pub refresh_command: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = directories::BaseDirs::new()
            .map(|dirs| dirs.data_local_dir().join(APP_NAME))
            .unwrap_or_else(|| PathBuf::from("."));

        Self {
            model: ModelConfig {
                path: data_dir.join("models/whisper-large-v3-turbo-safetensors"),
                model_id: "openai/whisper-large-v3-turbo".to_string(),
                draft_model_path: Some(data_dir.join("models/ggml-tiny.en.bin")),
                language: "en".to_string(),
                prompt: None,  // Disabled by default - causes decoder issues when enabled
            },
            audio: AudioConfig {
                sample_rate: 16000,
                timeout_secs: 30,
                save_audio_clips: false,
                audio_clips_path: default_audio_clips_path(),
            },
            output: OutputConfig {
                display_server: None,
                append_space: true,
                refresh_command: Some("pkill -RTMIN+8 waybar".to_string()),
            },
        }
    }
}

/// Load configuration from disk, creating default if not exists
pub fn load() -> Result<Config> {
    let config: Config = confy::load(APP_NAME, "config")?;
    Ok(config)
}

/// Save configuration to disk
pub fn save(config: &Config) -> Result<()> {
    confy::store(APP_NAME, "config", config)?;
    Ok(())
}

/// Get the configuration file path
pub fn config_path() -> Result<PathBuf> {
    let path = confy::get_configuration_file_path(APP_NAME, "config")?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.audio.sample_rate, 16000);
        assert_eq!(config.model.language, "en");
    }
}
