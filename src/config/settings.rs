use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const APP_NAME: &str = "dev-voice";

const DEFAULT_PROMPT: &str = "async, await, impl, struct, enum, pub, static, btreemap, hashmap, kubernetes, k8s, docker, container, pod, lifecycle, workflow, ci/cd, yaml, json, rustlang, python, javascript, typescript, bash, git, repo, branch, commit, push, pull, merge, rebase, upstream, downstream, middleware, database, sql, postgres, redis, api, endpoint, graphql, rest, grpc, protobuf, systemd, journalctl, flatpak, wayland, nix, cargo.";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub model: ModelConfig,
    pub audio: AudioConfig,
    pub output: OutputConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Path to whisper model file
    pub path: PathBuf,
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Force display server type: "wayland", "x11", or null for auto-detect
    pub display_server: Option<String>,
    /// Add a space after injected text
    pub append_space: bool,
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = directories::BaseDirs::new()
            .map(|dirs| dirs.data_local_dir().join(APP_NAME))
            .unwrap_or_else(|| PathBuf::from("."));

        Self {
            model: ModelConfig {
                path: data_dir.join("models/ggml-large-v3-turbo.bin"),
                draft_model_path: Some(data_dir.join("models/ggml-tiny.en.bin")),
                language: "en".to_string(),
                prompt: Some(DEFAULT_PROMPT.to_string()),
            },
            audio: AudioConfig {
                sample_rate: 16000,
                timeout_secs: 30,
            },
            output: OutputConfig {
                display_server: None,
                append_space: true,
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
