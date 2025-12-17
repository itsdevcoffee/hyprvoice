use anyhow::{Context, Result};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use tracing::{debug, error, info};

use crate::audio::capture_toggle;
use crate::daemon::protocol::{DaemonRequest, DaemonResponse};
use crate::state;
use crate::transcribe::Transcriber;

/// Get the path to the daemon socket
pub fn get_socket_path() -> Result<PathBuf> {
    let state_dir = state::paths::get_state_dir()?;
    Ok(state_dir.join("daemon.sock"))
}

/// Check if daemon is running by pinging it
pub fn is_daemon_running() -> bool {
    use crate::daemon::protocol::{DaemonRequest, DaemonResponse};
    use std::io::{BufRead, BufReader, Write};

    let socket_path = match get_socket_path() {
        Ok(p) => p,
        Err(_) => return false,
    };

    if !socket_path.exists() {
        return false;
    }

    // Try to ping the daemon
    let mut stream = match UnixStream::connect(&socket_path) {
        Ok(s) => s,
        Err(_) => return false,
    };

    // Send ping request (serializing Ping should never fail)
    let Ok(ping) = serde_json::to_string(&DaemonRequest::Ping) else {
        return false;
    };
    if stream.write_all(ping.as_bytes()).is_err() {
        return false;
    }
    if stream.write_all(b"\n").is_err() {
        return false;
    }
    if stream.flush().is_err() {
        return false;
    }

    // Try to read response
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return false;
    }

    // Check if we got a valid pong response
    serde_json::from_str::<DaemonResponse>(line.trim()).is_ok()
}

/// Shared state for async recording
struct RecordingState {
    handle: Option<JoinHandle<Result<Vec<f32>>>>,
    audio: Option<Vec<f32>>,
}

/// Daemon server state
struct DaemonServer {
    transcriber: Arc<Mutex<Transcriber>>,
    recording_state: Arc<Mutex<RecordingState>>,
    shutdown: Arc<AtomicBool>,
}

impl DaemonServer {
    fn new(model_path: &Path) -> Result<Self> {
        info!("Loading whisper model into GPU memory...");
        let transcriber = Transcriber::new(model_path).context("Failed to load whisper model")?;
        info!("Model loaded and resident in GPU VRAM");

        Ok(Self {
            transcriber: Arc::new(Mutex::new(transcriber)),
            recording_state: Arc::new(Mutex::new(RecordingState {
                handle: None,
                audio: None,
            })),
            shutdown: Arc::new(AtomicBool::new(false)),
        })
    }

    fn handle_client(&self, mut stream: UnixStream) -> Result<()> {
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut line = String::new();

        reader.read_line(&mut line)?;
        info!("Received from client: {}", line.trim());

        let request: DaemonRequest =
            serde_json::from_str(line.trim()).context("Failed to parse request")?;

        debug!("Parsed request: {:?}", request);

        let response = match request {
            DaemonRequest::Ping => DaemonResponse::Ok {
                message: "pong".to_string(),
            },
            DaemonRequest::StartRecording { max_duration } => {
                self.handle_start_recording(max_duration)?
            },
            DaemonRequest::StopRecording => self.handle_stop_recording()?,
            DaemonRequest::Shutdown => {
                info!("Shutdown requested");
                self.shutdown.store(true, Ordering::SeqCst);
                DaemonResponse::Ok {
                    message: "shutting down".to_string(),
                }
            },
        };

        let response_json = serde_json::to_string(&response)?;
        stream.write_all(response_json.as_bytes())?;
        stream.write_all(b"\n")?;
        stream.flush()?;

        Ok(())
    }

    fn handle_start_recording(&self, max_duration: u32) -> Result<DaemonResponse> {
        // Atomic check-and-set: mutex ensures no race between check and state update
        let mut state = self
            .recording_state
            .lock()
            .map_err(|e| anyhow::anyhow!("Recording state mutex poisoned: {}", e))?;

        // Check if already recording
        if state.handle.is_some() {
            return Ok(DaemonResponse::Error {
                message: "Already recording".to_string(),
            });
        }

        info!("Starting background recording (max {}s)", max_duration);

        // Create PID file for UI state (Waybar uses this)
        state::toggle::start_recording()?;

        // Set up signal handler for this recording session
        state::toggle::setup_signal_handler()?;

        // Spawn recording thread
        let handle = thread::spawn(move || capture_toggle(max_duration, 16000));

        state.handle = Some(handle);
        state.audio = None;

        Ok(DaemonResponse::Recording)
    }

    fn handle_stop_recording(&self) -> Result<DaemonResponse> {
        let mut state = self
            .recording_state
            .lock()
            .map_err(|e| anyhow::anyhow!("Recording state mutex poisoned: {}", e))?;

        // Check if recording
        let handle = match state.handle.take() {
            Some(h) => h,
            None => {
                return Ok(DaemonResponse::Error {
                    message: "Not recording".to_string(),
                });
            },
        };

        info!("Stop requested - signaling recording thread");

        // Send stop signal
        state::toggle::STOP_RECORDING.store(true, Ordering::SeqCst);

        // Wait for recording thread to finish
        drop(state); // Release lock while waiting
        let samples = handle
            .join()
            .map_err(|_| anyhow::anyhow!("Recording thread panicked"))??;

        // Reset stop flag for next recording
        state::toggle::STOP_RECORDING.store(false, Ordering::SeqCst);

        info!("Captured {} samples", samples.len());

        if samples.is_empty() {
            return Ok(DaemonResponse::Error {
                message: "No audio captured".to_string(),
            });
        }

        // Transcribe with the persistent model
        info!("Transcribing {} samples...", samples.len());
        let transcriber = self
            .transcriber
            .lock()
            .map_err(|e| anyhow::anyhow!("Transcriber mutex poisoned: {}", e))?;
        let text = transcriber
            .transcribe(&samples)
            .context("Transcription failed")?;

        if text.is_empty() {
            return Ok(DaemonResponse::Error {
                message: "No speech detected".to_string(),
            });
        }

        info!("Transcribed: {}", text);

        // Clean up PID file (recording complete)
        state::toggle::cleanup_recording()?;

        Ok(DaemonResponse::Success { text })
    }
}

/// Run the daemon server
pub fn run_daemon(model_path: &Path) -> Result<()> {
    let socket_path = get_socket_path()?;

    // Check if daemon is already running before removing socket
    if socket_path.exists() {
        if is_daemon_running() {
            anyhow::bail!("Daemon is already running. Stop it first or use the existing daemon.");
        }
        // Socket exists but daemon not responding - it's stale, safe to remove
        info!("Removing stale socket file");
        fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path).context("Failed to bind Unix socket")?;

    info!("Daemon listening on {}", socket_path.display());

    let server = DaemonServer::new(model_path)?;

    for stream in listener.incoming() {
        if server.shutdown.load(Ordering::SeqCst) {
            info!("Shutdown flag set, exiting");
            break;
        }

        match stream {
            Ok(stream) => {
                if let Err(e) = server.handle_client(stream) {
                    error!("Error handling client: {}", e);
                }
            },
            Err(e) => {
                error!("Error accepting connection: {}", e);
            },
        }
    }

    // Clean up socket on exit
    if socket_path.exists() {
        fs::remove_file(&socket_path)?;
    }

    info!("Daemon shut down");
    Ok(())
}
