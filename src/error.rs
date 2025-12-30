use std::path::PathBuf;
use thiserror::Error;

/// Custom error types for hyprvoice
/// Currently using anyhow, but this provides a foundation for future custom error handling
#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum DevVoiceError {
    #[error("Model not found: {path}")]
    ModelNotFound { path: PathBuf },

    #[error("Model download failed: {0}")]
    DownloadFailed(String),

    #[error("Checksum verification failed for {model}")]
    ChecksumMismatch { model: String },

    #[error("Audio capture failed: {0}")]
    AudioCapture(String),

    #[error("Transcription failed: {0}")]
    Transcription(String),

    #[error("Text injection failed: {0}")]
    TextInjection(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Display server {server:?} requires {tool} which is not installed")]
    MissingTool { server: String, tool: String },

    #[error("Unknown model: {name}. Available: {available}")]
    UnknownModel { name: String, available: String },
}

#[allow(dead_code)]
impl DevVoiceError {
    pub fn model_not_found(path: impl Into<PathBuf>) -> Self {
        Self::ModelNotFound { path: path.into() }
    }

    pub fn unknown_model(name: impl Into<String>, available: &[&str]) -> Self {
        Self::UnknownModel {
            name: name.into(),
            available: available.join(", "),
        }
    }

    pub fn missing_tool(server: impl Into<String>, tool: impl Into<String>) -> Self {
        Self::MissingTool {
            server: server.into(),
            tool: tool.into(),
        }
    }
}
