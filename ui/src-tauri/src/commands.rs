use serde::{Deserialize, Serialize};
use crate::daemon_client;

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub model_loaded: bool,
    pub gpu_enabled: bool,
    pub gpu_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscriptionEntry {
    pub id: String,
    pub text: String,
    pub timestamp: i64,
    pub duration_ms: u64,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu_cores: usize,
    pub total_ram_gb: f32,
    pub gpu_available: bool,
    pub gpu_vram_mb: Option<u32>,
    pub platform: String,
}

/// Check if the hyprvoice daemon is running
#[tauri::command]
pub async fn get_daemon_status() -> Result<DaemonStatus, String> {
    match daemon_client::get_status() {
        Ok(status) => Ok(DaemonStatus {
            running: status.running,
            model_loaded: status.model_loaded,
            gpu_enabled: status.gpu_enabled,
            gpu_name: status.gpu_name,
        }),
        Err(e) => {
            eprintln!("Failed to get daemon status: {}", e);
            // Return offline status on error
            Ok(DaemonStatus {
                running: false,
                model_loaded: false,
                gpu_enabled: false,
                gpu_name: None,
            })
        }
    }
}

/// Start recording audio
#[tauri::command]
pub async fn start_recording() -> Result<(), String> {
    let request = daemon_client::DaemonRequest::StartRecording {
        max_duration: 300, // 5 minutes max
    };

    match daemon_client::send_request(request) {
        Ok(response) => match response {
            daemon_client::DaemonResponse::Recording => {
                println!("Recording started");
                Ok(())
            }
            daemon_client::DaemonResponse::Error { message } => {
                Err(format!("Daemon error: {}", message))
            }
            _ => Err("Unexpected response from daemon".to_string()),
        },
        Err(e) => Err(format!("Failed to start recording: {}", e)),
    }
}

/// Stop recording and transcribe
#[tauri::command]
pub async fn stop_recording() -> Result<String, String> {
    let request = daemon_client::DaemonRequest::StopRecording;

    match daemon_client::send_request(request) {
        Ok(response) => match response {
            daemon_client::DaemonResponse::Success { text } => {
                println!("Transcription: {}", text);

                // Trigger status bar refresh (reads user's config)
                refresh_statusbar();

                Ok(text)
            }
            daemon_client::DaemonResponse::Error { message } => {
                Err(format!("Daemon error: {}", message))
            }
            _ => Err("Unexpected response from daemon".to_string()),
        },
        Err(e) => Err(format!("Failed to stop recording: {}", e)),
    }
}

/// Refresh status bar (execute user-configured refresh_command)
fn refresh_statusbar() {
    // Read config to get refresh_command
    let config_path = dirs::config_dir()
        .map(|dir| dir.join("hyprvoice").join("config.toml"));

    let Some(config_path) = config_path else {
        eprintln!("Could not determine config path");
        return;
    };

    // Read and parse config
    let Ok(config_str) = std::fs::read_to_string(&config_path) else {
        eprintln!("Could not read config file");
        return;
    };

    let Ok(config) = toml::from_str::<toml::Value>(&config_str) else {
        eprintln!("Could not parse config");
        return;
    };

    // Get refresh_command from config.output.refresh_command
    let Some(refresh_cmd) = config
        .get("output")
        .and_then(|o| o.get("refresh_command"))
        .and_then(|c| c.as_str())
    else {
        // No refresh command configured, skip silently
        return;
    };

    // Parse command string into program + args (e.g., "pkill -RTMIN+8 waybar")
    let parts: Vec<&str> = refresh_cmd.split_whitespace().collect();
    if let Some((program, args)) = parts.split_first() {
        let _ = std::process::Command::new(program)
            .args(args)
            .spawn()
            .map(|mut child| {
                // Detach immediately (non-blocking)
                let _ = child.stdin.take();
                let _ = child.stdout.take();
                let _ = child.stderr.take();
            });
    }
}

/// Get transcription history
#[tauri::command]
pub async fn get_transcription_history() -> Result<Vec<TranscriptionEntry>, String> {
    // TODO: Query transcription history from daemon or local DB
    Ok(vec![
        TranscriptionEntry {
            id: "1".to_string(),
            text: "This is a test transcription from earlier".to_string(),
            timestamp: 1704067200,
            duration_ms: 1500,
            model: "whisper-large-v3-turbo".to_string(),
        },
    ])
}

/// Download a Whisper model
#[tauri::command]
pub async fn download_model(model_name: String) -> Result<(), String> {
    // TODO: Call hyprvoice download command
    println!("Downloading model: {}", model_name);
    Ok(())
}

/// Get system information
#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    Ok(SystemInfo {
        cpu_cores: num_cpus::get(),
        total_ram_gb: 32.0, // TODO: Get actual RAM
        gpu_available: true,
        gpu_vram_mb: Some(24576), // TODO: Query actual VRAM
        platform: std::env::consts::OS.to_string(),
    })
}

/// Configuration structure matching config.toml
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub model: ModelConfig,
    pub audio: AudioConfig,
    pub output: OutputConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelConfig {
    pub path: String,
    pub model_id: String,
    pub draft_model_path: Option<String>,
    pub language: String,
    pub prompt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub timeout_secs: u32,
    pub save_audio_clips: bool,
    pub audio_clips_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutputConfig {
    pub append_space: bool,
    pub refresh_command: Option<String>,
}

/// Get current configuration
#[tauri::command]
pub async fn get_config() -> Result<AppConfig, String> {
    let config_path = dirs::config_dir()
        .ok_or("Could not determine config directory")?
        .join("hyprvoice")
        .join("config.toml");

    let config_str = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let config: AppConfig = toml::from_str(&config_str)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    Ok(config)
}

/// Save configuration
#[tauri::command]
pub async fn save_config(config: AppConfig) -> Result<(), String> {
    let config_path = dirs::config_dir()
        .ok_or("Could not determine config directory")?
        .join("hyprvoice")
        .join("config.toml");

    let config_str = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    std::fs::write(&config_path, config_str)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
}

/// Restart the hyprvoice daemon with new configuration
#[tauri::command]
pub async fn restart_daemon() -> Result<(), String> {
    // 1. Send shutdown command to daemon
    let shutdown_request = daemon_client::DaemonRequest::Shutdown;
    match daemon_client::send_request(shutdown_request) {
        Ok(_) => {
            eprintln!("Shutdown command sent to daemon");
        }
        Err(e) => {
            eprintln!("Failed to send shutdown (daemon might be down): {}", e);
        }
    }

    // 2. Wait for daemon to fully shut down (max 3 seconds)
    for i in 0..30 {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        if !daemon_client::is_daemon_running() {
            eprintln!("Daemon shut down after {}ms", i * 100);
            break;
        }
    }

    // 3. Start new daemon process
    // Find hyprvoice binary (prefer -test, -cuda, or default)
    let binary = if let Ok(home) = std::env::var("HOME") {
        let bin_dir = format!("{}/.local/bin", home);
        ["hyprvoice-test", "hyprvoice-cuda", "hyprvoice"]
            .iter()
            .map(|name| format!("{}/{}", bin_dir, name))
            .find(|path| std::path::Path::new(path).exists())
            .unwrap_or_else(|| "hyprvoice".to_string())
    } else {
        "hyprvoice".to_string()
    };

    eprintln!("Starting daemon with binary: {}", binary);

    std::process::Command::new(&binary)
        .arg("daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start daemon: {}", e))?;

    // 4. Wait for daemon to be ready (max 5 seconds)
    for i in 0..50 {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        if daemon_client::is_daemon_running() {
            eprintln!("Daemon started successfully after {}ms", i * 100);
            return Ok(());
        }
    }

    Err("Daemon failed to start within 5 seconds".to_string())
}

/// Validate if a path exists and return its type
#[tauri::command]
pub async fn validate_path(path: String) -> Result<PathValidation, String> {
    // Expand ~ to home directory
    let expanded_path = if path.starts_with('~') {
        if let Ok(home) = std::env::var("HOME") {
            path.replacen('~', &home, 1)
        } else {
            path.clone()
        }
    } else {
        path.clone()
    };

    let path_obj = std::path::Path::new(&expanded_path);

    if !path_obj.exists() {
        return Ok(PathValidation {
            valid: false,
            exists: false,
            is_file: false,
            is_directory: false,
            expanded_path: expanded_path.clone(),
            message: "Path does not exist".to_string(),
        });
    }

    let is_file = path_obj.is_file();
    let is_directory = path_obj.is_dir();

    Ok(PathValidation {
        valid: true,
        exists: true,
        is_file,
        is_directory,
        expanded_path: expanded_path.clone(),
        message: if is_file {
            "Valid file path".to_string()
        } else if is_directory {
            "Valid directory path".to_string()
        } else {
            "Path exists".to_string()
        },
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathValidation {
    pub valid: bool,
    pub exists: bool,
    pub is_file: bool,
    pub is_directory: bool,
    pub expanded_path: String,
    pub message: String,
}
