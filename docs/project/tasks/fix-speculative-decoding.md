# Task: Fix Speculative Decoding Implementation

**Assigned To:** Sub-Agent | **Priority:** Critical | **Status:** Not Started

## IMPORTANT: Research Requirements

**BEFORE starting implementation, you MUST:**
1. Use `context7` MCP to fetch the latest `whisper-rs` documentation
2. Use `WebFetch` to read the whisper-rs GitHub repository examples
3. Search for "speculative decoding" or "draft model" in the whisper.cpp documentation

**Key URLs to research:**
- `https://github.com/tazz4843/whisper-rs` (the Rust binding we use)
- `https://github.com/ggerganov/whisper.cpp` (underlying C++ library)
- `https://docs.rs/whisper-rs/latest/whisper_rs/` (API documentation)

## Objective

Implement working speculative decoding in the `Transcriber` to use a small "draft" model (e.g., `tiny.en`) to accelerate inference with a larger "target" model (e.g., `large-v3-turbo`).

**Expected Performance Gain:** 30-50% reduction in transcription latency.

## Current State

### What's Already Done:
- ✅ `ModelConfig` supports `draft_model_path: Option<PathBuf>`
- ✅ `Transcriber` struct holds `draft_ctx: Option<WhisperContext>`
- ✅ Draft model is loaded in `Transcriber::with_language()` (src/transcribe/whisper.rs:41-56)
- ✅ Daemon passes draft model from config (src/daemon/server.rs:85-88)

### What's Broken:
The code attempts to use `set_encoder_begin_callback` which doesn't exist in whisper-rs v0.15.1:

**Location:** `src/transcribe/whisper.rs:95-103`

```rust
// Enable speculative decoding if draft model is available
// Note: As of whisper-rs 0.15, speculative decoding is enabled by passing the draft context
// to the state or through specialized parameter setters.
// We will fallback to standard if the exact callback name is deprecated.
/*
if let Some(ref d_ctx) = self.draft_ctx {
    params.set_encoder_begin_callback(d_ctx);
}
*/
```

**Compilation Error (when uncommented):**
```
error[E0599]: no method named `set_encoder_begin_callback` found for struct `FullParams<'a, 'b>`
```

## Your Task

### 1. Research Phase
- Find the correct whisper-rs API for enabling speculative decoding
- Check if it requires:
  - A method on `FullParams`
  - A method on `WhisperState`
  - A different initialization pattern for `WhisperContext`
  - Specialized `WhisperContextParameters`

### 2. Implementation Phase
- Uncomment and fix the speculative decoding logic in `src/transcribe/whisper.rs`
- Ensure the draft model is used during inference
- Handle the case where draft model fails to load (graceful fallback)

### 3. Validation Phase
- Verify the code compiles: `cargo check --features cuda`
- Verify logs show draft model being used
- Test that transcription still works correctly
- Measure performance improvement (if possible)

## Technical Context

### File Structure
```
src/
├── config/settings.rs        # ModelConfig with draft_model_path
├── transcribe/whisper.rs     # Transcriber with draft_ctx (LINE 95-103 needs fix)
└── daemon/server.rs          # Loads draft model from config
```

### Key Code Locations

**Draft Model Loading** (src/transcribe/whisper.rs:41-56):
```rust
let draft_ctx = if let Some(path) = draft_model_path {
    if path.exists() {
        info!("Loading draft model for speculative decoding: {}", path.display());
        Some(WhisperContext::new_with_params(
            path.to_str()?,
            WhisperContextParameters::default(),
        )?)
    } else {
        None
    }
} else {
    None
};
```

**Inference Call** (src/transcribe/whisper.rs:105-116):
```rust
let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

// THIS IS WHERE THE FIX GOES (line 95-103)

// Apply technical vocabulary prompt if available
if let Some(ref prompt) = self.prompt {
    params.set_initial_prompt(prompt);
}

// Configure for dictation use case
params.set_language(Some(&self.language));
params.set_print_special(false);
// ... more params

// Run inference
state.full(params, audio)?;
```

## Success Criteria

- [ ] Code compiles without errors
- [ ] Draft model is loaded (visible in daemon logs)
- [ ] Transcription produces correct results
- [ ] Performance improvement is measurable (optional but ideal)
- [ ] Graceful fallback if draft model unavailable

## Possible Solutions (Research These)

Based on whisper.cpp architecture, speculative decoding might involve:

1. **Grammar/Constraints API**: Some implementations use "grammar" or "constraints" to guide the model
2. **State-based approach**: Pass draft context to `WhisperState` instead of `FullParams`
3. **Callback API**: Different callback name (e.g., `set_token_callback`, `set_logit_filter`)
4. **Direct whisper.cpp binding**: Call underlying C functions if Rust wrapper doesn't expose it

**You must research the actual whisper-rs v0.15.1 API to determine the correct approach.**

## Testing

After implementation, test with:
```bash
# Build
cargo build --release --features cuda

# Run daemon (should show draft model loading in logs)
pkill -9 dev-voice
./target/release/dev-voice daemon &

# Check logs
tail -f ~/.local/state/dev-voice/logs/dev-voice.log.2025-12-20
# Look for: "Loading draft model for speculative decoding"

# Test transcription
./target/release/dev-voice start --duration 5
# Speak a test phrase
# Verify output appears correctly
```

## References

- Current implementation: `src/transcribe/whisper.rs`
- Model registry: `src/model/registry.rs`
- Daemon integration: `src/daemon/server.rs`
- Phase 2 design: `docs/project/model-revamp-tasklist.md`

## Notes

- The draft model (`tiny.en`, 78MB) must be loaded alongside the target model
- Total VRAM usage: ~1.7GB (turbo) + ~80MB (tiny) = ~1.78GB
- If speculative decoding isn't directly supported in whisper-rs, document this and propose alternatives

🤖 Generated with [Claude Code](https://claude.com/claude-code)
