use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

/// Request to daemon (matches main hyprvoice protocol)
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DaemonRequest {
    #[serde(rename = "start_recording")]
    StartRecording { max_duration: u32 },
    #[serde(rename = "stop_recording")]
    StopRecording,
    #[serde(rename = "shutdown")]
    Shutdown,
    #[serde(rename = "ping")]
    Ping,
}

/// Response from daemon
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum DaemonResponse {
    #[serde(rename = "ok")]
    Ok { message: String },
    #[serde(rename = "recording")]
    Recording,
    #[serde(rename = "success")]
    Success { text: String },
    #[serde(rename = "error")]
    Error { message: String },
}

/// Get the daemon socket path
fn get_socket_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME env var not set")?;
    let state_dir = PathBuf::from(home)
        .join(".local")
        .join("state")
        .join("hyprvoice");
    Ok(state_dir.join("daemon.sock"))
}

/// Send a request to the daemon and get response
pub fn send_request(request: DaemonRequest) -> Result<DaemonResponse> {
    let socket_path = get_socket_path()?;

    if !socket_path.exists() {
        anyhow::bail!("Daemon socket not found. Is daemon running?");
    }

    // Connect to daemon
    let mut stream = UnixStream::connect(&socket_path)
        .context("Failed to connect to daemon socket")?;

    // Serialize request
    let request_json = serde_json::to_string(&request)
        .context("Failed to serialize request")?;

    // Send request (newline-delimited)
    stream.write_all(request_json.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    // Read response
    let mut reader = BufReader::new(stream);
    let mut response_line = String::new();
    reader.read_line(&mut response_line)
        .context("Failed to read response from daemon")?;

    // Parse response
    let response: DaemonResponse = serde_json::from_str(response_line.trim())
        .context("Failed to parse daemon response")?;

    Ok(response)
}

/// Check if daemon is running
pub fn is_daemon_running() -> bool {
    let socket_path = match get_socket_path() {
        Ok(p) => p,
        Err(_) => return false,
    };

    if !socket_path.exists() {
        return false;
    }

    // Try to ping
    send_request(DaemonRequest::Ping).is_ok()
}

/// Get daemon status with detailed info
pub fn get_status() -> Result<DaemonStatusInfo> {
    if !is_daemon_running() {
        return Ok(DaemonStatusInfo {
            running: false,
            model_loaded: false,
            gpu_enabled: false,
            gpu_name: None,
        });
    }

    // Ping daemon to verify it's responsive
    send_request(DaemonRequest::Ping)?;

    // TODO: Add status command to daemon protocol to get actual model/GPU info
    // For now, assume if daemon is running, model is loaded
    Ok(DaemonStatusInfo {
        running: true,
        model_loaded: true,
        gpu_enabled: true, // TODO: Query actual GPU status
        gpu_name: Some("Unknown".to_string()), // TODO: Get from daemon
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonStatusInfo {
    pub running: bool,
    pub model_loaded: bool,
    pub gpu_enabled: bool,
    pub gpu_name: Option<String>,
}
