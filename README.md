# dev-voice

Voice dictation for Linux developers. Capture speech, transcribe with Whisper, inject text at cursor.

## Requirements

### System Dependencies

**All platforms:**
- Rust 1.85+ (for cargo build)
- Clang (for whisper-rs build)

**Runtime dependencies:**
- **Type mode:** None - fully self-contained
- **Clipboard mode (Linux only):**
  - Wayland: `wl-clipboard` (install: `sudo dnf install wl-clipboard`)
  - X11: `xclip` (install: `sudo dnf install xclip`)
- **macOS/Windows:** No dependencies for clipboard mode

## Build

### Standard Build (Wayland on Linux, native on macOS/Windows)

```bash
# Clone
git clone https://github.com/yourusername/dev-voice.git
cd dev-voice

# Build release
cargo build --release

# Binary at ./target/release/dev-voice
```

### Platform-Specific Notes

**Linux (Wayland):** Default configuration, works out of box on modern distros.

**Linux (X11):** Edit `Cargo.toml` line 81, change:
```toml
# FROM:
features = ["wayland"]
# TO:
features = ["x11rb"]
```
Then rebuild with `cargo build --release`.

**macOS:** Uses native CoreGraphics, no configuration needed.

**Windows:** Uses native SendInput API, no configuration needed.

### GPU Acceleration (optional)

```bash
# NVIDIA CUDA
cargo build --release --features cuda

# AMD ROCm
cargo build --release --features rocm

# Vulkan (cross-platform)
cargo build --release --features vulkan
```

## Usage

```bash
# Download a whisper model
dev-voice download base.en

# Check system readiness
dev-voice doctor

# Start recording (5 seconds)
dev-voice start --duration 5

# View config
dev-voice config
```

## Hyprland Integration

Add to `~/.config/hypr/hyprland.conf`:

```ini
bind = SUPER, V, exec, dev-voice start --duration 5
```

## Breaking Changes in v0.2.0 (Phase 4)

**Type mode no longer preserves clipboard:**
- Previous versions (Phase 3) used paste shortcuts (Ctrl+V) which required saving/restoring clipboard
- Current version types text directly at the cursor using `enigo.text()` - more reliable but doesn't touch clipboard
- **Migration:** If you need clipboard-based workflow, use clipboard mode: `dev-voice start -c`

**Cross-platform text injection:**
- Migrated from Linux-only tools (wtype/xdotool) to cross-platform enigo library
- Type mode works on Linux (Wayland/X11), macOS, and Windows
- Clipboard mode requires `wl-clipboard` (Wayland) or `xclip` (X11) on Linux

## Configuration

Config file: `~/.config/dev-voice/config.toml`

```toml
[model]
path = "~/.local/share/dev-voice/models/ggml-base.en.bin"
language = "en"

[audio]
sample_rate = 16000
timeout_secs = 30

[output]
append_space = true
```

## License

MIT
