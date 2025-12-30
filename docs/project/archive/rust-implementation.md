# Dev-Voice: Rust Implementation Plan

**Status:** Planning
**Language:** Rust
**Target:** Linux (Fedora 42, Wayland/Hyprland, PipeWire)

---

## Verified Crate Stack

| Crate | Version | Purpose | Source |
|-------|---------|---------|--------|
| [whisper-rs](https://codeberg.org/tazz4843/whisper-rs) | 0.15.1 | Speech-to-text via whisper.cpp | [crates.io](https://crates.io/crates/whisper-rs) |
| [pipewire](https://pipewire.pages.freedesktop.org/pipewire-rs/pipewire/) | 0.9.2 | Audio capture | [crates.io](https://crates.io/crates/pipewire) |
| [clap](https://docs.rs/clap/latest/clap/) | 4.5.x | CLI argument parsing | [crates.io](https://crates.io/crates/clap) |
| [confy](https://github.com/rust-cli/confy) | 0.6.x | Config file management (TOML) | [crates.io](https://crates.io/crates/confy) |
| [directories](https://lib.rs/crates/directories) | 5.x | XDG path resolution | [crates.io](https://crates.io/crates/directories) |

**Note:** whisper-rs GitHub is archived — active development at [Codeberg](https://codeberg.org/tazz4843/whisper-rs)

---

## Project Structure

```
hyprvoice/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point, CLI handling
│   ├── lib.rs            # Library root
│   ├── audio/
│   │   ├── mod.rs
│   │   └── capture.rs    # PipeWire audio capture
│   ├── transcribe/
│   │   ├── mod.rs
│   │   └── whisper.rs    # Whisper integration
│   ├── output/
│   │   ├── mod.rs
│   │   └── inject.rs     # wtype/xdotool text injection
│   └── config/
│       ├── mod.rs
│       └── settings.rs   # Config structs + confy
├── models/               # Whisper models (git-ignored)
└── config.example.toml   # Example configuration
```

---

## Cargo.toml

```toml
[package]
name = "hyprvoice"
version = "0.1.0"
edition = "2024"
rust-version = "1.77"
description = "Voice dictation for Linux developers"
license = "MIT"

[dependencies]
# Speech recognition
whisper-rs = "0.15"

# Audio capture
pipewire = "0.9"
libspa = "0.9"

# CLI
clap = { version = "4.5", features = ["derive"] }

# Config
confy = "0.6"
serde = { version = "1.0", features = ["derive"] }

# Utilities
directories = "5"
anyhow = "1.0"
thiserror = "2.0"

[features]
default = []
cuda = ["whisper-rs/cuda"]
rocm = ["whisper-rs/hipblas"]
vulkan = ["whisper-rs/vulkan"]

[profile.release]
lto = true
codegen-units = 1
strip = true
```

---

## Core Components

### 1. Audio Capture (`src/audio/capture.rs`)

```rust
use pipewire::{self as pw, stream::Stream, properties};
use std::sync::mpsc;

pub struct AudioCapture {
    // Audio buffer channel
    tx: mpsc::Sender<Vec<f32>>,
}

impl AudioCapture {
    pub fn new(tx: mpsc::Sender<Vec<f32>>) -> Self {
        Self { tx }
    }

    pub fn start(&self) -> anyhow::Result<()> {
        pw::init();
        let mainloop = pw::main_loop::MainLoop::new(None)?;
        let context = pw::context::Context::new(&mainloop)?;
        let core = context.connect(None)?;

        let props = properties! {
            *pw::keys::MEDIA_TYPE => "Audio",
            *pw::keys::MEDIA_CATEGORY => "Capture",
            *pw::keys::MEDIA_ROLE => "Communication",
        };

        let stream = Stream::new(&core, "hyprvoice-capture", props)?;

        // Configure stream for 16kHz mono (Whisper requirement)
        // Connect with Direction::Input, AUTOCONNECT, MAP_BUFFERS

        mainloop.run();
        Ok(())
    }
}
```

**Audio format for Whisper:**
- Sample rate: 16kHz
- Channels: Mono
- Format: 32-bit float PCM

---

### 2. Transcription (`src/transcribe/whisper.rs`)

```rust
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};
use std::path::Path;

pub struct Transcriber {
    ctx: WhisperContext,
}

impl Transcriber {
    pub fn new(model_path: &Path) -> anyhow::Result<Self> {
        let params = WhisperContextParameters::default();
        let ctx = WhisperContext::new_with_params(
            model_path.to_str().unwrap(),
            params,
        )?;
        Ok(Self { ctx })
    }

    pub fn transcribe(&self, audio: &[f32]) -> anyhow::Result<String> {
        let mut state = self.ctx.create_state()?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        state.full(params, audio)?;

        let num_segments = state.full_n_segments()?;
        let mut result = String::new();

        for i in 0..num_segments {
            if let Ok(segment) = state.full_get_segment_text(i) {
                result.push_str(&segment);
            }
        }

        Ok(result.trim().to_string())
    }
}
```

**Model recommendations:**
| Model | Size | Speed | Accuracy | Use Case |
|-------|------|-------|----------|----------|
| `base.en` | 142MB | Fast | Good | MVP default |
| `small.en` | 466MB | Medium | Better | Quality option |
| `medium.en` | 1.5GB | Slow | Best | High-end hardware |

---

### 3. Text Output (`src/output/inject.rs`)

```rust
use std::process::Command;

pub enum DisplayServer {
    Wayland,
    X11,
}

impl DisplayServer {
    pub fn detect() -> Self {
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            Self::Wayland
        } else {
            Self::X11
        }
    }
}

pub fn inject_text(text: &str, display: &DisplayServer) -> anyhow::Result<()> {
    match display {
        DisplayServer::Wayland => {
            Command::new("wtype")
                .arg("--")
                .arg(text)
                .spawn()?
                .wait()?;
        }
        DisplayServer::X11 => {
            Command::new("xdotool")
                .args(["type", "--clearmodifiers", "--", text])
                .spawn()?
                .wait()?;
        }
    }
    Ok(())
}
```

**Dependencies:**
- Wayland: `wtype` — `sudo dnf install wtype`
- X11: `xdotool` — `sudo dnf install xdotool`

---

### 4. Configuration (`src/config/settings.rs`)

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub model: ModelConfig,
    pub audio: AudioConfig,
    pub output: OutputConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Path to whisper model file
    pub path: PathBuf,
    /// Language code (e.g., "en")
    pub language: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Sample rate (default: 16000)
    pub sample_rate: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Force display server type (auto-detect if None)
    pub display_server: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let dirs = directories::ProjectDirs::from("com", "devvoice", "hyprvoice")
            .expect("Failed to get project dirs");

        Self {
            model: ModelConfig {
                path: dirs.data_dir().join("models/ggml-base.en.bin"),
                language: "en".to_string(),
            },
            audio: AudioConfig {
                sample_rate: 16000,
            },
            output: OutputConfig {
                display_server: None,
            },
        }
    }
}

pub fn load_config() -> anyhow::Result<Config> {
    Ok(confy::load("hyprvoice", "config")?)
}

pub fn save_config(config: &Config) -> anyhow::Result<()> {
    Ok(confy::store("hyprvoice", "config", config)?)
}
```

**Config location:** `~/.config/hyprvoice/config.toml`

---

### 5. CLI (`src/main.rs`)

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hyprvoice")]
#[command(about = "Voice dictation for Linux developers")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start listening for voice input
    Start {
        /// Override model path
        #[arg(short, long)]
        model: Option<String>,
    },
    /// Stop the running daemon
    Stop,
    /// Download a whisper model
    Download {
        /// Model size: tiny, base, small, medium, large
        #[arg(default_value = "base.en")]
        model: String,
    },
    /// Show current configuration
    Config,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { model } => {
            // Load config, initialize audio, start transcription loop
        }
        Commands::Stop => {
            // Send stop signal to daemon
        }
        Commands::Download { model } => {
            // Download model from Hugging Face
        }
        Commands::Config => {
            // Print current config
        }
    }

    Ok(())
}
```

---

## MVP Flow

```
┌─────────────────────────────────────────────────────────┐
│  Hyprland: bind = SUPER, V, exec, hyprvoice start       │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│  1. Load config from ~/.config/hyprvoice/config.toml    │
│  2. Initialize PipeWire stream (16kHz mono capture)     │
│  3. Load Whisper model into memory                      │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│  Recording Loop:                                        │
│  - Capture audio until silence detected (VAD)           │
│  - Or fixed duration (e.g., 10 seconds)                 │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│  Transcribe audio buffer with whisper-rs                │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│  Inject text via wtype (Wayland)                        │
└─────────────────────────────────────────────────────────┘
```

---

## Build & Run

```bash
# Clone with submodules (whisper.cpp)
git clone --recursive https://github.com/youruser/hyprvoice.git
cd hyprvoice

# Build (release)
cargo build --release

# Download model
./target/release/hyprvoice download base.en

# Run
./target/release/hyprvoice start
```

**System dependencies (Fedora):**
```bash
sudo dnf install \
    pipewire-devel \
    clang \
    cmake \
    wtype
```

---

## GPU Acceleration

```bash
# CUDA (NVIDIA)
cargo build --release --features cuda

# ROCm (AMD) — Linux only
cargo build --release --features rocm

# Vulkan (cross-platform)
cargo build --release --features vulkan
```

---

## Implementation Order

| Phase | Task | Complexity |
|-------|------|------------|
| 1 | Project setup, CLI skeleton | Low |
| 2 | Config system with confy | Low |
| 3 | Whisper integration (file input test) | Medium |
| 4 | PipeWire audio capture | Medium |
| 5 | Text injection (wtype) | Low |
| 6 | End-to-end integration | Medium |
| 7 | VAD / silence detection | Medium |
| 8 | Model download command | Low |

---

## References

- [whisper-rs docs](https://docs.rs/whisper-rs/latest/whisper_rs/)
- [whisper-rs Codeberg](https://codeberg.org/tazz4843/whisper-rs)
- [pipewire-rs docs](https://pipewire.pages.freedesktop.org/pipewire-rs/pipewire/)
- [clap derive tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/chapter_0/index.html)
- [confy examples](https://github.com/rust-cli/confy/blob/master/examples/simple.rs)
- [Whisper models (Hugging Face)](https://huggingface.co/ggerganov/whisper.cpp)
