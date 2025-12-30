# hyprvoice Improvements Task List

**Status:** Complete
**Started:** 2025-12-09
**Last Updated:** 2025-12-09

---

## Completed

### Phase 1 - Core Improvements (2025-12-09)

- [x] **Native HTTP Downloads** - Replaced external `curl` with `ureq` crate
  - Location: `src/model/download.rs`

- [x] **Checksum Verification** - SHA256 verification for downloaded models
  - Location: `src/model/verify.rs`

- [x] **Pre-allocate Audio Buffer** - Reduce reallocations during recording
  - Location: `src/audio/capture.rs:37-43`

- [x] **Improve Resampling Quality** - Replaced linear interpolation with rubato
  - Location: `src/audio/capture.rs:431-483`

- [x] **Better Display Server Detection** - Check XDG_SESSION_TYPE first
  - Location: `src/output/inject.rs:11-30`

- [x] **Clipboard Mode** - Added `--clipboard` flag
  - Location: `src/output/inject.rs:33-88`

- [x] **Extract Model Management Module** - Created `src/model/`
  - `download.rs`, `verify.rs`, `registry.rs`

- [x] **Custom Error Types** - Created `src/error.rs`

### Phase 2 - Toggle Mode & Logging (2025-12-09)

- [x] **File Logging** - Rolling daily logs with tracing_appender
  - Location: `src/main.rs:101-126`
  - Logs to: `~/.local/state/hyprvoice/logs/hyprvoice.log`

- [x] **Toggle Mode** - Start/stop recording with same command
  - Location: `src/main.rs:138-209`, `src/state/toggle.rs`
  - First `hyprvoice start` → begins recording
  - Second `hyprvoice start` or `hyprvoice stop` → stops and transcribes
  - 5 minute timeout if never stopped

- [x] **State Management Module** - Created `src/state/`
  - `paths.rs` - XDG directory management
  - `toggle.rs` - PID file and signal handling

- [x] **Stop Command** - Explicit stop command added
  - `hyprvoice stop` sends SIGUSR1 to recording process

---

## Summary of All Changes

### New Files
```
src/model/mod.rs
src/model/download.rs
src/model/verify.rs
src/model/registry.rs
src/error.rs
src/state/mod.rs
src/state/paths.rs
src/state/toggle.rs
```

### Modified Files
```
Cargo.toml - Added: ureq, sha2, hex, rubato, tracing-appender, nix, scopeguard
src/main.rs - Toggle mode, file logging, stop command
src/lib.rs - Added model and state module exports
src/audio/capture.rs - Pre-allocation, rubato resampling, capture_toggle()
src/output/inject.rs - OutputMode enum, clipboard support, better detection
```

### New Dependencies
| Crate | Version | Purpose |
|-------|---------|---------|
| ureq | 2.9 | HTTP downloads |
| sha2 | 0.10 | Checksum verification |
| hex | 0.4 | Hex encoding |
| rubato | 0.15 | High-quality resampling |
| tracing-appender | 0.2 | File logging |
| nix | 0.30 | Signal handling |
| scopeguard | 1.2 | Cleanup guards |

### CLI Changes

```bash
# Toggle mode (default)
hyprvoice start           # Start recording
hyprvoice start           # Stop and transcribe (same command)
hyprvoice stop            # Explicit stop

# Fixed duration (unchanged)
hyprvoice start -d 5      # Record for 5 seconds

# Clipboard mode
hyprvoice start -c        # Output to clipboard

# Check logs location
hyprvoice doctor          # Shows log directory
```

### Test Results
- 14 tests passing
- All modules have unit tests

---

## Deferred / Future

- [ ] **Voice Activity Detection (VAD)** - Auto-stop on silence
- [ ] **Push-to-Talk Mode** - Global hotkey integration
- [ ] **Better Doctor Command** - Version checks, benchmarks
- [ ] **Async Architecture** - Major rewrite for concurrency
