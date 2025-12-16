# enigo Wayland Paste Issue

**Status:** Blocked - Key simulation failing on Wayland/Hyprland

## Problem

Migrated from wtype/xdotool to enigo v0.6 for cross-platform text injection. Clipboard operations work perfectly, but paste keyboard shortcuts fail.

### Symptoms

- ✅ Clipboard read/write works (verified in clipboard history)
- ✅ Clipboard preservation works
- ❌ Paste shortcut types plain 'v' instead of Ctrl+Shift+V
- ❌ Modifier keys (Ctrl, Shift) not held during keypress

### Environment

- **OS:** Fedora 42 (Wayland)
- **Compositor:** Hyprland
- **Terminal:** kitty
- **enigo:** v0.6.1 with features `["wayland", "x11rb"]`
- **arboard:** v3.6.1 with features `["wayland-data-control"]`

## Current Implementation

**File:** `src/output/mod.rs:154-225`

```rust
fn paste_with_shift(enigo: &mut Enigo) -> Result<()> {
    // Press Ctrl
    enigo.key(Key::Control, Direction::Press)?;
    thread::sleep(Duration::from_millis(50));

    // Press Shift
    enigo.key(Key::Shift, Direction::Press)?;
    thread::sleep(Duration::from_millis(50));

    // Press+Release 'v'
    enigo.key(Key::Unicode('v'), Direction::Press)?;
    thread::sleep(Duration::from_millis(50));
    enigo.key(Key::Unicode('v'), Direction::Release)?;
    thread::sleep(Duration::from_millis(50));

    // Release Shift
    enigo.key(Key::Shift, Direction::Release)?;
    thread::sleep(Duration::from_millis(50));

    // Release Ctrl
    enigo.key(Key::Control, Direction::Release)?;

    Ok(())
}
```

### What We've Tried

1. **Initial:** Used `Direction::Click` for 'v' → Typed plain 'v'
2. **Fix 1:** Changed to explicit Press/Release → Still typed 'v'
3. **Fix 2:** Added 20ms delays between operations → Still typed 'v'
4. **Fix 3:** Increased delays to 50ms → Still typed 'v'

### Log Output

```
2025-12-16T10:07:05.148422Z  INFO Using Ctrl+Shift+V for terminal paste
v2025-12-16T10:07:05.226117Z  INFO Pasted 25 chars at cursor
```

Note the stray 'v' character - modifiers not recognized.

## Working Code (Previous Implementation)

**File:** `src/output/inject.rs.wtype-backup:92-155`

Used wtype subprocess:
```rust
Command::new("wtype")
    .args(["-M", "ctrl", "-M", "shift", "-k", "v",
           "-m", "shift", "-m", "ctrl"])
    .status()?;
```

This worked reliably on Wayland.

## Root Cause Hypothesis

enigo's Wayland backend doesn't properly synchronize modifier key states with the compositor. Hyprland likely processes key events asynchronously, causing the 'v' keypress to execute before seeing the Ctrl+Shift state.

## Possible Solutions

### Option A: Hybrid Approach (wtype fallback)
```rust
#[cfg(target_os = "linux")]
fn paste_with_shift_linux() -> Result<()> {
    // Try enigo first
    if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
        // ... enigo implementation ...
        return Ok(());
    }

    // Fallback to wtype on Wayland
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        Command::new("wtype")
            .args(["-M", "ctrl", "-M", "shift", "-k", "v",
                   "-m", "shift", "-m", "ctrl"])
            .status()?;
    }
}
```

**Pros:** Known to work, keeps cross-platform for macOS/Windows
**Cons:** External dependency on Linux, defeats purpose of enigo

### Option B: Direct Text Typing
```rust
fn paste_at_cursor(text: &str) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default())?;
    enigo.text(text)?;  // Type directly, no clipboard
    Ok(())
}
```

**Pros:** Simple, reliable, cross-platform
**Cons:** Loses clipboard preservation, might break on special chars

### Option C: Try Different Key Representation
```rust
// Instead of Key::Unicode('v')
enigo.key(Key::Layout('v'), Direction::Press)?;
// or
enigo.key(Key::Raw(25), Direction::Press)?;  // V keycode on X11
```

**Pros:** Might fix enigo sync issue
**Cons:** Keycode differs by platform, not in enigo v0.6 docs

## Questions for Review

1. Is enigo v0.6 Wayland support production-ready?
2. Should we use wtype as fallback or switch to direct typing?
3. Are there enigo configuration options we're missing?
4. Does enigo work on macOS for comparison testing?

## Files Changed

- `Cargo.toml` - Added enigo + arboard dependencies
- `src/output/mod.rs` - New enigo implementation (330 lines)
- `src/output/inject.rs` - Backed up as `inject.rs.wtype-backup`
- `src/main.rs` - Updated API calls (removed DisplayServer param)

## Test Commands

```bash
# Test Type mode (shows the bug)
RUST_LOG=debug cargo run -- start -d 3

# Test Clipboard mode (works fine)
RUST_LOG=debug cargo run -- start -d 3 -c
```
