use anyhow::{Context, Result};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::fs;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;

use super::paths::get_pid_file;

/// Global flag to signal recording should stop
pub static STOP_RECORDING: AtomicBool = AtomicBool::new(false);

/// Recording state information
#[derive(Debug)]
pub struct RecordingState {
    pub pid: u32,
    pub started_at: u64,
}

/// Check if a recording is currently in progress
pub fn is_recording() -> Result<Option<RecordingState>> {
    let pid_file = get_pid_file()?;

    if !pid_file.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&pid_file)?;
    let mut lines = content.lines();

    let pid: u32 = lines
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let started_at: u64 = lines
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if pid == 0 {
        // Invalid PID file, clean up
        let _ = fs::remove_file(&pid_file);
        return Ok(None);
    }

    // Check if process is still running
    let process_exists = signal::kill(Pid::from_raw(pid as i32), None).is_ok();

    if !process_exists {
        // Stale PID file, clean up
        info!("Cleaning up stale PID file (process {} not running)", pid);
        let _ = fs::remove_file(&pid_file);
        return Ok(None);
    }

    Ok(Some(RecordingState { pid, started_at }))
}

/// Mark recording as started (create PID file)
pub fn start_recording() -> Result<()> {
    let pid_file = get_pid_file()?;
    let pid = std::process::id();
    let started_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut file = fs::File::create(&pid_file)
        .context("Failed to create PID file")?;

    writeln!(file, "{}", pid)?;
    writeln!(file, "{}", started_at)?;

    info!("Recording started (PID: {}, file: {})", pid, pid_file.display());
    Ok(())
}

/// Stop a running recording by sending SIGUSR1
pub fn stop_recording(state: &RecordingState) -> Result<()> {
    info!("Sending stop signal to recording process (PID: {})", state.pid);

    signal::kill(Pid::from_raw(state.pid as i32), Signal::SIGUSR1)
        .context("Failed to send stop signal to recording process")?;

    Ok(())
}

/// Clean up PID file (called when recording ends)
pub fn cleanup_recording() -> Result<()> {
    let pid_file = get_pid_file()?;
    if pid_file.exists() {
        fs::remove_file(&pid_file)?;
        info!("Cleaned up PID file");
    }
    Ok(())
}

/// Set up signal handler for SIGUSR1
pub fn setup_signal_handler() -> Result<()> {
    // Register handler for SIGUSR1
    unsafe {
        signal::signal(Signal::SIGUSR1, signal::SigHandler::Handler(handle_stop_signal))
            .context("Failed to set up signal handler")?;
    }
    Ok(())
}

/// Signal handler function
extern "C" fn handle_stop_signal(_: i32) {
    STOP_RECORDING.store(true, Ordering::SeqCst);
}

/// Check if stop was requested
pub fn should_stop() -> bool {
    STOP_RECORDING.load(Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_state_lifecycle() {
        // Clean up any existing state
        let _ = cleanup_recording();

        // Should not be recording initially
        assert!(is_recording().unwrap().is_none());

        // Start recording
        start_recording().unwrap();

        // Should be recording now
        let state = is_recording().unwrap();
        assert!(state.is_some());
        assert_eq!(state.unwrap().pid, std::process::id());

        // Clean up
        cleanup_recording().unwrap();

        // Should not be recording after cleanup
        assert!(is_recording().unwrap().is_none());
    }
}
