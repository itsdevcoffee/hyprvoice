use anyhow::{Context, Result};
use arboard::Clipboard;
use enigo::{Enigo, Keyboard, Settings};
use tracing::info;

/// How to output transcribed text
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputMode {
    /// Type text at cursor position (default)
    #[default]
    Type,
    /// Copy text to clipboard only
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

/// Inject text using the specified mode
///
/// # Arguments
/// * `text` - The text to output
/// * `mode` - How to output the text (Type or Clipboard)
pub fn inject_text(text: &str, mode: OutputMode) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }

    match mode {
        OutputMode::Clipboard => {
            copy_to_clipboard(text)?;
            info!("Copied to clipboard: {} chars", text.len());
            Ok(())
        }
        OutputMode::Type => {
            type_text(text)?;
            info!("Typed {} chars at cursor", text.len());
            Ok(())
        }
    }
}

/// Copy text to clipboard only
fn copy_to_clipboard(text: &str) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        // On Linux, use wl-copy (Wayland) or xclip (X11) for reliable clipboard persistence
        // arboard has issues with Wayland clipboard managers
        use std::io::Write;
        use std::process::{Command, Stdio};

        // Try wl-copy first (Wayland)
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            let mut child = Command::new("wl-copy")
                .stdin(Stdio::piped())
                .spawn()
                .context("Failed to spawn wl-copy. Install with: sudo dnf install wl-clipboard")?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(text.as_bytes())?;
            }

            let status = child.wait()?;
            if !status.success() {
                anyhow::bail!("wl-copy exited with status: {}", status);
            }
        } else {
            // Fallback to xclip (X11)
            let mut child = Command::new("xclip")
                .args(["-selection", "clipboard"])
                .stdin(Stdio::piped())
                .spawn()
                .context("Failed to spawn xclip. Install with: sudo dnf install xclip")?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(text.as_bytes())?;
            }

            let status = child.wait()?;
            if !status.success() {
                anyhow::bail!("xclip exited with status: {}", status);
            }
        }

        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    {
        // On macOS/Windows, arboard works fine
        let mut clipboard = Clipboard::new().context("Failed to access clipboard")?;
        clipboard
            .set_text(text)
            .context("Failed to set clipboard text")?;
        Ok(())
    }
}

/// Type text directly at cursor using enigo
///
/// Uses the input_method protocol on Wayland and equivalent on X11/macOS/Windows.
/// This bypasses clipboard entirely and works reliably across platforms.
fn type_text(text: &str) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default())
        .context("Failed to initialize enigo")?;

    enigo.text(text)
        .context("Failed to type text")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_empty_text() {
        let result = inject_text("", OutputMode::Type);
        assert!(result.is_ok());
    }
}
