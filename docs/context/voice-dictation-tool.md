# Linux Voice Dictation Tool for Developers

**Status:** Planning
**Goal:** Build an open-source voice-to-text tool optimized for coding on Linux

---

## Problem Statement

- Popular voice dictation tools (Wispr Flow, Aqua Voice, SuperWhisper) are Mac/Windows exclusive
- Linux developers using coding agents (Claude Code, Cursor, Windsurf) lack quality voice input
- Existing Linux solutions are fragmented, poorly maintained, or require significant setup

---

## Market Research Summary

### Commercial Tools (Mac/Windows Only)

| Tool         | Accuracy | Price     | Key Feature                    |
| ------------ | -------- | --------- | ------------------------------ |
| Wispr Flow   | 97.2%    | ~$12/mo   | Cursor/Windsurf IDE extensions |
| Aqua Voice   | 98.5%    | ~$8-10/mo | Screen context awareness       |
| SuperWhisper | 95-98%   | ~$5/mo    | Fully offline/local            |

### Existing Linux Options

| Tool           | Status     | Notes                         |
| -------------- | ---------- | ----------------------------- |
| OpenWhispr     | Active     | Electron, basic hotkey+paste  |
| nerd-dictation | Active     | Python, Vosk-based, CLI       |
| Speech Note    | Active     | Qt app, multiple backends     |
| voice_typing   | Maintained | Bash script, terminal-focused |

---

## Proposed Architecture

```
Global Hotkey (push-to-talk)
         │
         ▼
Audio Capture (PipeWire/PulseAudio)
         │
         ▼
Whisper.cpp (local transcription)
         │
         ▼
LLM Post-Processing (optional)
  - Fix technical terminology
  - Context from active window
  - Handle voice commands
         │
         ▼
Output (wtype for Wayland / xdotool for X11)
```

---

## Technical Decisions

### Language Options

| Choice   | Recommendation  | Reasoning                                                     |
| -------- | --------------- | ------------------------------------------------------------- |
| **Rust** | Preferred       | Performance, `whisper-rs`, `cpal` audio, system-level control |
| Go       | Alternative     | Simpler, good whisper bindings, fast dev                      |
| Bun/TS   | Not recommended | Would require FFI/subprocess for Whisper                      |

### Core Dependencies

**Rust crates:**

- `whisper-rs` - Whisper.cpp bindings
- `cpal` - Cross-platform audio capture
- `global-hotkey` - System-wide keyboard shortcuts
- `wayland-client` or subprocess to `wtype` - Text output

**System dependencies (Fedora):**

```bash
sudo dnf install wtype  # Wayland text input
# or xdotool for X11
```

### Speech Recognition

- **Primary:** whisper.cpp (via whisper-rs)
  - Models: tiny, base, small, medium, large
  - Tradeoff: larger = more accurate but slower
- **Alternative:** Vosk (lighter weight, less accurate)

### LLM Integration (Differentiator)

**Purpose:** Context-aware post-processing

```
Raw: "create a function called get user by eye dee"
     ↓ LLM with context: "in TypeScript file"
Out: "getUserById"
```

**Options:**

- Local: Ollama (Llama 3.2, Qwen2.5) - no latency penalty
- Cloud: Claude API / OpenAI - higher accuracy, requires network

---

## MVP Feature Set

1. **Global hotkey** - Toggle or push-to-talk recording
2. **Audio capture** - PipeWire/PulseAudio support
3. **Local transcription** - whisper.cpp with configurable model size
4. **Text output** - Wayland (wtype) and X11 (xdotool) support
5. **Basic config** - Hotkey, model selection, output method

### Post-MVP

- LLM post-processing toggle
- Active window context detection
- Custom vocabulary/corrections
- Voice commands ("new line", "select all", "undo")
- Tray icon / status indicator

---

## Reference Projects

| Project        | URL                                          | What to Learn          |
| -------------- | -------------------------------------------- | ---------------------- |
| whisper.cpp    | https://github.com/ggerganov/whisper.cpp     | Core engine            |
| whisper-rs     | https://github.com/tazz4843/whisper-rs       | Rust bindings          |
| OpenWhispr     | https://github.com/HeroTools/open-whispr     | Hotkey + paste flow    |
| nerd-dictation | https://github.com/ideasman42/nerd-dictation | VAD, Vosk integration  |
| cpal examples  | https://github.com/RustAudio/cpal            | Audio capture patterns |

---

## Open Questions

- [ ] Push-to-talk vs toggle vs voice activity detection (VAD)?
- [ ] Ship whisper model bundled or download on first run?
- [ ] Config file format? (TOML seems fitting for Rust)
- [ ] Tray icon framework? (egui, gtk-rs, or headless daemon?)
- [ ] Name for the project?

---

## Next Steps

1. Set up Rust project with basic structure
2. Implement audio capture with `cpal`
3. Integrate `whisper-rs` for transcription
4. Add global hotkey listener
5. Implement wtype/xdotool output
6. Test end-to-end flow
7. Add configuration system
8. (Later) LLM post-processing layer

---

## Environment Context

- **Target OS:** Linux (Fedora 42, Wayland/Hyprland)
- **Display Server:** Wayland (Hyprland compositor)
- **Audio:** PipeWire
- **Primary use case:** Voice input for Claude Code and terminal workflows
