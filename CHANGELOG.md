# Changelog

All notable changes to hyprvoice will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-12-16 (Phase 4)

### Added
- **Cross-platform support:** macOS, Windows, and X11 in addition to Wayland
- **Clipboard mode flag:** `hyprvoice start -c` to copy text to clipboard instead of typing
- **Native HTTP downloads:** Replaced `curl` subprocess with `ureq` library for model downloads
- **Checksum verification:** SHA256 verification for downloaded Whisper models
- **Better resampling:** Migrated from linear interpolation to `rubato` library
- **enigo-test command:** Quick validation tool for testing text injection without full voice recording
- **Cross-platform audio:** Migrated from PipeWire to CPAL for Linux/macOS/Windows support

### Changed
- **BREAKING:** Type mode no longer preserves clipboard contents
  - Previous versions used paste shortcuts (Ctrl+V) which required saving/restoring clipboard
  - Current version types text directly at cursor using `enigo.text()` - more reliable but doesn't interact with clipboard
  - Migration: Use `hyprvoice start -c` for clipboard-based workflow
- **BREAKING:** Migrated from Linux-only tools (wtype/xdotool) to cross-platform enigo library
- **BREAKING:** Audio capture migrated from PipeWire to CPAL (cross-platform)
- Simplified output module from 330+ lines to ~120 lines (-63% code reduction)
- Improved error messages with installation instructions for missing dependencies
- Better display server detection (XDG_SESSION_TYPE environment variable)

### Removed
- Terminal detection logic (~100 lines)
- Clipboard preservation in type mode (~200 lines)
- Paste keyboard shortcut simulation (Ctrl+V, Ctrl+Shift+V)
- PipeWire-specific audio capture code
- Direct dependencies on wtype/xdotool (now optional runtime dependencies for legacy support)

### Technical Details
- enigo v0.6.1 for cross-platform text injection
- arboard v3.6.1 for clipboard (macOS/Windows only)
- cpal v0.16.0 for cross-platform audio capture
- rubato v0.16.2 for high-quality audio resampling
- ureq v2.9 for HTTP downloads
- sha2 v0.10 for checksum verification

### Platform-Specific Notes
- **Linux Wayland:** Default configuration, uses `wl-clipboard` for clipboard mode
- **Linux X11:** Requires manual edit of `Cargo.toml` line 80, uses `xclip` for clipboard mode
- **macOS:** Uses native CoreGraphics for typing, native clipboard API
- **Windows:** Uses native SendInput API for typing, native clipboard API

## [0.1.0] - 2025-12-10 (Phase 3)

### Added
- Initial release with Linux Wayland support
- Voice recording with PipeWire audio capture
- Whisper model integration for speech recognition
- Daemon mode for fast response times
- Model download and management
- wtype/xdotool integration for text injection
- Configuration system
- Hyprland integration examples

### Features
- Record voice input with configurable duration
- Transcribe speech to text using Whisper models
- Automatic text injection at cursor position
- Clipboard preservation during paste operations
- Support for GPU acceleration (CUDA, ROCm, Vulkan)
- Daemon mode for sub-second response times

---

## Version History

- **v0.2.0 (Phase 4):** Cross-platform support, simplified architecture
- **v0.1.0 (Phase 3):** Initial Linux Wayland release
