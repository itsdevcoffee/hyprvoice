# hyprvoice

<div align="center">

**Voice-to-text for developers who think faster than they type.**

Fast • Local • Private • GPU-Accelerated

[Quick Start](#-quick-start) • [Features](#-features) • [Roadmap](#-roadmap) • [Documentation](#-documentation)

</div>

---

## 🎯 What is hyprvoice?

**hyprvoice** is a lightning-fast, privacy-first voice dictation tool built for developers. Press a hotkey, speak naturally, and your words appear instantly at your cursor—no cloud, no latency, no compromises.

### The Problem

Typing code comments, documentation, commit messages, and chat responses is slow. Cloud-based voice tools are either:
- **Too slow** (network latency kills flow state)
- **Too intrusive** (your code goes to someone else's servers)
- **Too generic** (can't handle technical vocabulary like "async fn", "kubectl", or "GraphQL")

### The Solution

hyprvoice runs **100% locally** on your machine with **GPU acceleration**, delivering transcription in under 500ms. It understands technical terminology out of the box and works offline. Built in Rust, powered by OpenAI Whisper.

---

## ✨ Features

### 🚀 **Blazing Fast**
- **GPU-accelerated** transcription with CUDA (NVIDIA), Metal (Apple Silicon), or ROCm (AMD)
- **5-10x faster** than CPU-only solutions
- **Sub-second latency** for typical voice commands

### 🔒 **Privacy-First**
- **100% local processing** — your voice never leaves your machine
- **No cloud dependencies** — works completely offline
- **No telemetry** — we don't track anything

### 🧠 **Developer-Aware**
- Understands technical vocabulary: `async/await`, `kubernetes`, `GraphQL`, `flatpak`, `systemd`
- Customizable prompts to bias toward your tech stack
- Language detection (English, Spanish, French, and more)

### ⚡ **Cross-Platform**
- **Linux:** Wayland (Hyprland, Sway, KDE) and X11
- **macOS:** Intel and Apple Silicon (with Metal acceleration)
- **Windows:** Coming soon

### 🎨 **Desktop Integration**
- **Waybar module** with real-time status (idle/recording/processing)
- Polybar support coming soon
- Systemd service for always-on daemon mode

### 🛠️ **Built for Power Users**
- **Daemon mode** for instant response
- **Toggle mode** (press once to start, again to stop)
- **Clipboard mode** for manual pasting
- **Keyboard shortcuts** via Hyprland/Sway bindings

---

## 🚀 Quick Start

### 1. Download

Grab the latest binary for your platform:

```bash
# Linux (NVIDIA GPU)
wget https://github.com/itsdevcoffee/hyprvoice/releases/download/v0.2.0/hyprvoice-linux-x64-cuda
chmod +x hyprvoice-linux-x64-cuda
mv hyprvoice-linux-x64-cuda ~/.local/bin/hyprvoice

# macOS (Apple Silicon with Metal)
wget https://github.com/itsdevcoffee/hyprvoice/releases/download/v0.2.0/hyprvoice-macos-arm64-metal
chmod +x hyprvoice-macos-arm64-metal
mv hyprvoice-macos-arm64-metal ~/.local/bin/hyprvoice
```

### 2. Download a Model

```bash
hyprvoice download base.en  # 148MB, balanced speed/accuracy
```

### 3. Start the Daemon

```bash
hyprvoice daemon
```

### 4. Use It

```bash
# In another terminal (or bind to a hotkey)
hyprvoice start    # Begin recording
# Speak: "This is a test of voice dictation"
hyprvoice stop     # Transcribe and inject text
```

**Text appears at your cursor!**

---

## 🗺️ Roadmap

### ✅ **v0.2.0 - Current** (Cross-Platform Foundation)
- [x] Candle-based Whisper engine (Rust-native, Python-free)
- [x] GPU acceleration (CUDA, Metal)
- [x] Cross-platform audio (CPAL)
- [x] macOS and Linux support
- [x] Waybar integration

### 🚧 **v0.3.0 - Next** (Performance & Polish)
- [ ] **Flash Attention v2** for 2x faster inference
- [ ] **Speculative decoding** with draft models (30-50% speedup)
- [ ] Polybar integration (X11/i3 users)
- [ ] Automated model downloads on first run
- [ ] Performance benchmarking suite

### 🔮 **v0.4.0 - Future** (Advanced Features)
- [ ] Context-aware vocabulary (detect `.rs`, `.py`, `.ts` files, bias accordingly)
- [ ] DeepFilterNet noise cancellation (handle keyboard/fan noise)
- [ ] AT-SPI2 integration (pull active window context for better accuracy)
- [ ] Multi-language testing (Spanish, French, German)
- [ ] Custom wake words for hands-free mode

### 🌟 **v1.0.0 - Vision** (Production Ready)
- [ ] IDE plugins (VSCode, Neovim, JetBrains)
- [ ] Voice commands ("undo last", "format code", "new line")
- [ ] Project-specific vocabulary learning
- [ ] Mobile companion app (trigger from phone)

[Full roadmap →](docs/project/todos/roadmap.md)

---

## 📚 Documentation

### Installation Guides
- **[Linux Setup](docs/setup-linux.md)** (Fedora, Ubuntu, Arch)
- **[macOS Setup](docs/setup-macos.md)** (Intel and Apple Silicon)
- **[GPU Acceleration](docs/gpu-setup.md)** (CUDA, Metal, ROCm)
- **[Building from Source](docs/build.md)**

### Integration
- **[Waybar Module](integrations/waybar/README.md)** (Live status widget)
- **[Hyprland Keybinds](docs/keybinds-hyprland.md)**
- **[Systemd Service](docs/systemd.md)** (Auto-start daemon)

### Advanced
- **[Architecture Overview](docs/context/current-implementation-state.md)**
- **[Model Selection Guide](docs/models.md)**
- **[Performance Tuning](docs/performance.md)**
- **[Troubleshooting](docs/troubleshooting.md)**

---

## 🏗️ How It Works

```
┌─────────────────────────────────────────────────────────────┐
│  1. Press Hotkey (Super+V)                                  │
│     ↓                                                        │
│  2. Audio Capture (CPAL) → 44.1kHz stereo                   │
│     ↓                                                        │
│  3. Resample to 16kHz mono (Rubato)                         │
│     ↓                                                        │
│  4. Whisper Transcription                                   │
│     ├─ Encoder (GPU/CPU) → Audio features                   │
│     └─ Decoder (Greedy/Beam) → Text tokens                  │
│     ↓                                                        │
│  5. Text Injection (Enigo) → Types at cursor                │
│     OR Clipboard (wl-copy/arboard) → Paste manually         │
└─────────────────────────────────────────────────────────────┘
```

**Key Technologies:**
- **Rust** - Memory-safe, zero-cost abstractions
- **Candle** - Pure Rust ML framework (no Python!)
- **Whisper Large V3 Turbo** - 809M params, 4 decoder layers
- **CPAL** - Cross-platform audio
- **Enigo** - Cross-platform keyboard injection

---

## 🎛️ Model Options

| Model | Size | Speed | Accuracy | Best For |
|-------|------|-------|----------|----------|
| **tiny.en** | 78 MB | ⚡⚡⚡ | ⭐⭐ | Testing, instant feedback |
| **base.en** | 148 MB | ⚡⚡ | ⭐⭐⭐ | **Recommended** - Balanced |
| **small.en** | 488 MB | ⚡ | ⭐⭐⭐⭐ | Higher accuracy |
| **large-v3-turbo** | 1.6 GB | ⚡⚡ | ⭐⭐⭐⭐⭐ | Maximum quality |

**Recommendation:** Start with `base.en` (148MB). Upgrade to `large-v3-turbo` if you need near-perfect accuracy.

---

## 🖥️ Platform Support

| OS | Architecture | GPU | Status |
|----|--------------|-----|--------|
| **Linux** | x86_64 | CUDA (NVIDIA) | ✅ **Tested** |
| **Linux** | x86_64 | ROCm (AMD) | 🟡 **Untested** |
| **macOS** | Apple Silicon | Metal | ✅ **Tested** |
| **macOS** | Intel | None | ✅ **Tested** |
| **Windows** | x86_64 | None | 🟡 **Code Ready** |

**Tested Environments:**
- Fedora 42 (Wayland/Hyprland)
- Ubuntu 24.04 (Wayland/GNOME)
- macOS 14-26 (Intel & Apple Silicon)

---

## 🔧 Configuration Example

**`~/.config/hyprvoice/config.toml`**

```toml
[model]
model_id = "openai/whisper-large-v3-turbo"
language = "en"
prompt = "async, await, rust, cargo, kubernetes, docker, typescript"

[audio]
sample_rate = 16000    # Auto-resamples from device default
timeout_secs = 30      # Max recording duration

[output]
append_space = true
refresh_command = "pkill -RTMIN+8 waybar"  # Update Waybar status
```

---

## 🤝 Contributing

We welcome contributions! hyprvoice is **open source** (MIT license) and community-driven.

### Ways to Contribute
- 🐛 **Report bugs** via [GitHub Issues](https://github.com/itsdevcoffee/hyprvoice/issues)
- 💡 **Suggest features** on our [Discussions](https://github.com/itsdevcoffee/hyprvoice/discussions)
- 📝 **Improve docs** (setup guides, troubleshooting, translations)
- 🔌 **Build integrations** (Polybar, i3status, GNOME extension)
- 🧪 **Test on your platform** and share results

### Development Setup

```bash
git clone https://github.com/itsdevcoffee/hyprvoice.git
cd hyprvoice
cargo build --release --features cuda  # or 'metal' for macOS

# Run tests
cargo test

# Lint and format
cargo clippy
cargo fmt --all
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

---

## 📊 Performance Benchmarks

**Whisper Base Model, 10-second audio clip:**

| Hardware | Time | Speedup |
|----------|------|---------|
| AMD Ryzen 7 (CPU) | 3.0s | 1x |
| Apple M1 (CPU) | 2.2s | 1.4x |
| **NVIDIA RTX 4090 (CUDA)** | **0.5s** | **6x** |
| **Apple M2 (Metal)** | **1.0s** | **3x** |

*Results may vary based on model size and audio complexity.*

---

## 🙏 Acknowledgments

Built on the shoulders of giants:

- **[OpenAI Whisper](https://github.com/openai/whisper)** - State-of-the-art speech recognition
- **[Candle](https://github.com/huggingface/candle)** - Minimalist ML framework in Rust
- **[CPAL](https://github.com/RustAudio/cpal)** - Cross-platform audio library
- **[Enigo](https://github.com/enigo-rs/enigo)** - Cross-platform input simulation

Special thanks to the Hyprland and Rust communities for inspiration and support.

---

## 📄 License

**MIT License** - See [LICENSE](LICENSE) for details.

Free and open source forever. Use it, fork it, contribute back.

---

## 🌟 Why We Built This

We're developers who got tired of:
- Typing the same technical terms over and over
- Slow cloud transcription breaking our flow
- Privacy concerns with commercial voice tools
- Lack of Linux-first voice solutions

hyprvoice is our answer: a tool that respects your privacy, runs at the speed of thought, and understands the language you actually speak.

**If you think faster than you type, hyprvoice is for you.**

---

<div align="center">

**[⬆ Back to Top](#hyprvoice)**

Made with ❤️ for developers who value speed, privacy, and control.

**Star us on GitHub** if you find this useful!

</div>
