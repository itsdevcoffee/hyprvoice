//! Enigo keyboard/clipboard testing command
//!
//! Provides a dedicated test command for validating enigo functionality
//! without requiring full voice recording and transcription.

use anyhow::Result;
use tracing::info;

use crate::output::{self, OutputMode};

/// Run enigo test suite
///
/// Tests clipboard operations and text injection.
/// Provides a 3-second countdown to allow focusing the target window.
pub fn run(text: &str, clipboard: bool) -> Result<()> {
    println!("\n=== Enigo Test Suite ===\n");

    let mode = if clipboard {
        OutputMode::Clipboard
    } else {
        OutputMode::Type
    };
    println!("Mode: {:?}", mode);
    println!("Test text: {:?}\n", text);

    // Test clipboard read/write
    test_clipboard_operations()?;

    // Give user time to focus window
    countdown_to_paste();

    // Execute the operation
    execute_injection(text, mode)?;

    println!("\n=== Test Complete ===\n");
    Ok(())
}

/// Test clipboard read/write operations
fn test_clipboard_operations() -> Result<()> {
    println!("Testing clipboard operations...");

    match arboard::Clipboard::new().and_then(|mut cb| cb.get_text()) {
        Ok(content) => {
            println!("✓ Clipboard read successful ({} chars)", content.len());
        }
        Err(e) => {
            println!("✗ Clipboard read failed: {}", e);
        }
    }

    match arboard::Clipboard::new().and_then(|mut cb| cb.set_text("enigo-test")) {
        Ok(()) => {
            println!("✓ Clipboard write successful");
        }
        Err(e) => {
            println!("✗ Clipboard write failed: {}", e);
        }
    }

    Ok(())
}

/// Display countdown to give user time to focus target window
fn countdown_to_paste() {
    println!("\n>>> Focus your target window now! Typing in 3 seconds...");
    std::thread::sleep(std::time::Duration::from_secs(1));
    println!(">>> 2...");
    std::thread::sleep(std::time::Duration::from_secs(1));
    println!(">>> 1...");
    std::thread::sleep(std::time::Duration::from_secs(1));
}

/// Execute the text injection and report results
fn execute_injection(text: &str, mode: OutputMode) -> Result<()> {
    println!("\nExecuting text injection...");
    info!("Calling inject_text with mode: {:?}", mode);

    match output::inject_text(text, mode) {
        Ok(()) => {
            println!("✓ inject_text completed successfully");
            if mode == OutputMode::Clipboard {
                println!("\n✓ Text copied to clipboard!");
            } else {
                println!("\n✓ Text typed at cursor!");
            }
            Ok(())
        }
        Err(e) => {
            println!("✗ inject_text failed: {}", e);
            Err(e)
        }
    }
}
