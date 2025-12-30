# Task: Migrate Whisper Engine to Candle with Speculative Decoding

**Status:** Planning
**Priority:** High
**Owner:** Claude Agent

## Overview
Migrate the core transcription engine from the archived `whisper-rs` (whisper.cpp) to a native Rust implementation using `candle-core` (v0.9.2-alpha.2). This migration enables **Speculative Decoding** and support for **Whisper Large-v3-Turbo**, targeting sub-150ms end-to-end latency for a seamless "vibe coding" experience.

## Goals
- **Performance:** Achieve < 150ms transcription latency on modern hardware (CUDA/Metal).
- **Modernity:** Support `Large-v3-Turbo` (4 decoder layers) and `Distil-Large-v3`.
- **Reliability:** Escape the archived `whisper-rs` dependency.
- **Speculative Decoding:** Implement a Draft/Target model loop (e.g., Tiny.en -> Large-v3-Turbo).

## Technical Requirements
- **Framework:** Candle `0.9.2-alpha.2`
- **Models:**
    - **Target:** `openai/whisper-large-v3-turbo` (Safetensors)
    - **Draft:** `openai/whisper-tiny.en` or `distil-whisper/distil-small.en`
- **Acceleration:** Flash Attention v2.5, CUDA 12.x / Metal support
- **Quantization:** Q4_K / Q8_0 via Safetensors

## Tasks

### Phase 1: Infrastructure & Dependencies ✅ COMPLETED
- [x] Remove `whisper-rs` and `ggml` dependencies from `Cargo.toml`.
- [x] Add `candle-core`, `candle-transformers`, `candle-nn` (v0.9.2-alpha.2).
- [x] Add `hf-hub` and `tokenizers` for model management.
- [x] HuggingFace Hub integration handles model downloads automatically (no separate ModelManager needed).

### Phase 2: Core Engine Migration ✅ COMPLETED
- [x] Create `src/transcribe/candle_engine.rs` to house the new `CandleEngine`.
- [x] Port Mel-filterbank calculation from C++ to native Rust (using pre-computed filters from Candle).
- [x] Implement the Whisper Encoder/Decoder forward passes.
- [x] Set up the token streaming and decoding logic (Greedy decoding).
- [x] Add language configuration support (English by default, extensible).
- [x] Add technical vocabulary prompt biasing (initial_prompt support).
- [x] Proper special token handling (SOT, EOT, language, transcribe, notimestamps).
- [x] Fix mutable borrowing for encoder/decoder forward passes.
- [x] Download and integrate mel filter banks (80-bin and 128-bin).
- [x] Update Transcriber trait to use `&mut self` for Candle compatibility.
- [x] Integration with daemon server and main CLI.
- [x] All tests passing (14/14).

### Phase 3: Speculative Decoding Implementation
- [ ] Implement the `SpeculativeGenerator` loop.
- [ ] Logic for generating $K$ tokens via Draft and verifying in batch via Target.
- [ ] Handle token rejection and re-generation logic.

### Phase 4: Optimization & Vibe Mode
- [ ] Integrate Flash Attention kernels for CUDA.
- [ ] Create "Vibe Mode" configuration preset in `src/config/mod.rs`.
- [ ] Implement weight quantization (Q4) for reduced VRAM footprint.

## Expected Outcomes
- **Implementation Complete** when:
    - [ ] `hyprvoice` transcribes audio using the `Large-v3-Turbo` model via Candle.
    - [ ] Speculative Decoding toggle is functional and provides measurable speedup.
    - [ ] End-to-end latency is consistently below 150ms for short sentences.
    - [ ] VRAM usage is optimized (< 2GB for the Speculative pair).

## Future Roadmap
- [ ] Support for distilled English-only models for even lower latency.
- [ ] Integration with Ollama for unified model orchestration.
- [ ] Real-time "Lookahead" decoding (streaming while speaking).

## Implementation Notes (Phase 2 - Completed 2025-12-21)

### What Was Accomplished
1. **Full CandleEngine Implementation**: src/transcribe/candle_engine.rs (248 lines)
   - Proper HuggingFace model loading via `hf-hub` API
   - GPU acceleration (CUDA/Metal) with CPU fallback
   - Mel spectrogram preprocessing using pre-computed filter banks

2. **Whisper Decoding**:
   - Correct special token sequence: `<|startoftranscript|><|en|><|transcribe|><|notimestamps|>[prompt]`
   - Greedy decoder with EOT detection
   - Language configuration support (English default, extensible to other languages)
   - Initial prompt support for technical vocabulary biasing

3. **Feature Parity with whisper-rs**:
   - ✅ Language configuration (`language: "en"`)
   - ✅ Initial prompt/technical vocabulary (`initial_prompt: Option<String>`)
   - ✅ GPU acceleration (CUDA/Metal)
   - ✅ CPU fallback
   - ⚠️ Speculative decoding (Draft model) - deferred to Phase 3

4. **Integration**:
   - Updated `Transcriber` trait to use `&mut self` (required by Candle's forward passes)
   - Integrated with daemon server (src/daemon/server.rs)
   - Integrated with main CLI (src/main.rs)
   - All 14 tests passing

### Key Technical Decisions
1. **Mel Filter Banks**: Using pre-computed filter bytes from Candle repo (assets/melfilters80.bytes, assets/melfilters128.bytes)
   - Eliminates need for FFT computation
   - Matches reference Whisper implementation

2. **Model Downloads**: Using `hf-hub` API for automatic Safetensors downloads
   - No manual model management required
   - Models cached in `~/.cache/huggingface/`

3. **Default Model**: `openai/whisper-large-v3-turbo`
   - 4 decoder layers (vs 32 in Large-v3)
   - Optimized for speed while maintaining accuracy

### Known Limitations
1. **No Beam Search**: Currently only greedy decoding implemented
   - Sufficient for most use cases
   - Beam search can be added later if needed

2. **Fixed Timestamp Mode**: Always using `<|notimestamps|>` token
   - Dictation use case doesn't need timestamps
   - Can be made configurable if needed

3. **Single Language per Instance**: Language is set at initialization
   - Matches old whisper-rs behavior
   - Auto-detection can be added later

### Next Steps (Phase 3)
- Implement speculative decoding with draft model
- Add support for `distil-whisper/distil-large-v3` as draft model
- Benchmark latency with and without speculative decoding
- Target: < 150ms end-to-end latency

### References
- [Candle Whisper Example](https://github.com/huggingface/candle/blob/main/candle-examples/examples/whisper/main.rs)
- [Candle Transformers Docs](https://docs.rs/candle-transformers/latest/candle_transformers/models/whisper/)
- [OpenAI Whisper Tokenization](https://github.com/openai/whisper/blob/main/whisper/tokenizer.py)
