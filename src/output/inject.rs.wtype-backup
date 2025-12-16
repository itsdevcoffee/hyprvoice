use anyhow::{Context, Result};
use std::io::Write;
use std::process::{Command, Stdio};
use tracing::{debug, info};

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
                _ => {}, // Fall through to other checks
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
    /// Parse from string (used in tests)
    #[cfg(test)]
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
    // Save current clipboard to restore later
    let saved_clipboard = Command::new("wl-paste").output().ok().and_then(|output| {
        if output.status.success() {
            Some(output.stdout)
        } else {
            None
        }
    });

    // Copy transcription to clipboard
    info!("Copying text to clipboard ({} chars)", text.len());
    copy_wayland(text)?;
    info!("Clipboard copy successful");

    // Small delay to ensure clipboard is set
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Detect if focused window is a terminal (needs Ctrl+Shift+V)
    let (use_shift, window_class) = is_terminal_focused_with_class();
    info!(
        "Focused window class: {:?}, use_shift: {}",
        window_class, use_shift
    );

    // Simulate paste: Ctrl+V or Ctrl+Shift+V for terminals
    let status = if use_shift {
        info!("Using Ctrl+Shift+V for terminal paste");
        Command::new("wtype")
            .args([
                "-M", "ctrl", "-M", "shift", "-k", "v", "-m", "shift", "-m", "ctrl",
            ])
            .status()
    } else {
        info!("Using Ctrl+V for standard paste");
        Command::new("wtype")
            .args(["-M", "ctrl", "-k", "v", "-m", "ctrl"])
            .status()
    }
    .context("Failed to execute wtype. Is it installed? (sudo dnf install wtype)")?;

    info!("wtype exit status: {:?}", status);

    if !status.success() {
        anyhow::bail!("wtype exited with status: {}", status);
    }

    // Restore original clipboard
    if let Some(saved) = saved_clipboard {
        std::thread::sleep(std::time::Duration::from_millis(50)); // Wait for paste to complete
        let mut child = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .spawn()
            .context("Failed to spawn wl-copy for clipboard restore")?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(&saved).ok();
        }
        child.wait().ok();
        info!("Restored original clipboard ({} bytes)", saved.len());
    }

    Ok(())
}

/// Check if the focused window is a terminal (requires Ctrl+Shift+V to paste)
/// Returns (is_terminal, window_class)
fn is_terminal_focused_with_class() -> (bool, Option<String>) {
    let output = Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .output();

    let Ok(output) = output else {
        debug!("hyprctl command failed");
        return (false, None);
    };

    let Ok(json_str) = std::str::from_utf8(&output.stdout) else {
        debug!("Failed to parse hyprctl output as UTF-8");
        return (false, None);
    };

    debug!("hyprctl output: {}", json_str);

    // Extract class from JSON using simple parsing
    let class = extract_json_string(json_str, "class");
    debug!("Extracted class: {:?}", class);

    // Known terminal window classes
    const TERMINALS: &[&str] = &[
        "kitty",
        "alacritty",
        "foot",
        "wezterm",
        "gnome-terminal",
        "konsole",
        "xfce4-terminal",
        "terminator",
        "tilix",
        "st",
        "urxvt",
        "xterm",
    ];

    let is_terminal = class
        .as_ref()
        .is_some_and(|c| TERMINALS.iter().any(|t| c.to_lowercase() == *t));

    (is_terminal, class)
}

/// Extract a string value from JSON (simple parser to avoid serde dependency)
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    // Handle both "key": "value" and "key":"value" formats
    let key_pattern = format!("\"{}\":", key);
    let key_pos = json.find(&key_pattern)?;
    let after_key = &json[key_pos + key_pattern.len()..];

    // Skip whitespace and find opening quote
    let trimmed = after_key.trim_start();
    if !trimmed.starts_with('"') {
        return None;
    }

    // Find the value between quotes
    let value_start = 1; // skip opening quote
    let value_end = trimmed[value_start..].find('"')?;
    Some(trimmed[value_start..value_start + value_end].to_string())
}

fn inject_x11(text: &str) -> Result<()> {
    // Type text character-by-character (no delay between keystrokes)
    let status = Command::new("xdotool")
        .args(["type", "--clearmodifiers", "--delay", "0", "--", text])
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
        assert_eq!(
            OutputMode::from_str("clipboard"),
            Some(OutputMode::Clipboard)
        );
        assert_eq!(OutputMode::from_str("copy"), Some(OutputMode::Clipboard));
        assert_eq!(OutputMode::from_str("invalid"), None);
    }
}
