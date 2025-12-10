use anyhow::{Context, Result};
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy)]
pub enum DisplayServer {
    Wayland,
    X11,
}

impl DisplayServer {
    /// Auto-detect the current display server
    /// Checks XDG_SESSION_TYPE first (more reliable), falls back to WAYLAND_DISPLAY
    pub fn detect() -> Self {
        // XDG_SESSION_TYPE is the most reliable indicator
        if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
            match session_type.as_str() {
                "wayland" => return Self::Wayland,
                "x11" => return Self::X11,
                _ => {} // Fall through to other checks
            }
        }

        // Fallback: check for Wayland display socket
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            Self::Wayland
        } else {
            Self::X11
        }
    }
}

/// How to output transcribed text
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputMode {
    /// Type text at cursor position (default)
    #[default]
    Type,
    /// Copy text to clipboard
    Clipboard,
}

impl OutputMode {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "type" | "inject" => Some(Self::Type),
            "clipboard" | "copy" => Some(Self::Clipboard),
            _ => None,
        }
    }
}

/// Output text using the specified mode
pub fn output_text(text: &str, mode: OutputMode, display: &DisplayServer) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }

    match mode {
        OutputMode::Type => inject_text(text, display),
        OutputMode::Clipboard => copy_to_clipboard(text, display),
    }
}

/// Inject text at the current cursor position
pub fn inject_text(text: &str, display: &DisplayServer) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }

    match display {
        DisplayServer::Wayland => inject_wayland(text),
        DisplayServer::X11 => inject_x11(text),
    }
}

/// Copy text to clipboard
pub fn copy_to_clipboard(text: &str, display: &DisplayServer) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }

    match display {
        DisplayServer::Wayland => copy_wayland(text),
        DisplayServer::X11 => copy_x11(text),
    }
}

fn inject_wayland(text: &str) -> Result<()> {
    let status = Command::new("wtype")
        .arg("-d")  // delay between keystrokes in ms
        .arg("1")   // 1ms
        .arg("--")
        .arg(text)
        .status()
        .context("Failed to execute wtype. Is it installed? (sudo dnf install wtype)")?;

    if !status.success() {
        anyhow::bail!("wtype exited with status: {}", status);
    }

    Ok(())
}

fn inject_x11(text: &str) -> Result<()> {
    let status = Command::new("xdotool")
        .args(["type", "--clearmodifiers", "--", text])
        .status()
        .context("Failed to execute xdotool. Is it installed? (sudo dnf install xdotool)")?;

    if !status.success() {
        anyhow::bail!("xdotool exited with status: {}", status);
    }

    Ok(())
}

fn copy_wayland(text: &str) -> Result<()> {
    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to execute wl-copy. Is it installed? (sudo dnf install wl-clipboard)")?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(text.as_bytes())?;
    }

    let status = child.wait()?;
    if !status.success() {
        anyhow::bail!("wl-copy exited with status: {}", status);
    }

    Ok(())
}

fn copy_x11(text: &str) -> Result<()> {
    let mut child = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to execute xclip. Is it installed? (sudo dnf install xclip)")?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(text.as_bytes())?;
    }

    let status = child.wait()?;
    if !status.success() {
        anyhow::bail!("xclip exited with status: {}", status);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_detection() {
        // This test just ensures the function doesn't panic
        let _display = DisplayServer::detect();
    }

    #[test]
    fn test_output_mode_parsing() {
        assert_eq!(OutputMode::from_str("type"), Some(OutputMode::Type));
        assert_eq!(OutputMode::from_str("clipboard"), Some(OutputMode::Clipboard));
        assert_eq!(OutputMode::from_str("copy"), Some(OutputMode::Clipboard));
        assert_eq!(OutputMode::from_str("invalid"), None);
    }
}
