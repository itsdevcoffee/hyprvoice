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

### Phase 1: Infrastructure & Dependencies
- [ ] Remove `whisper-rs` and `ggml` dependencies from `Cargo.toml`.
- [ ] Add `candle-core`, `candle-transformers`, `candle-nn` (v0.9.2-alpha.2).
- [ ] Add `hf-hub` and `tokenizers` for model management.
- [ ] Implement `ModelManager` to automate Safetensors downloads.

### Phase 2: Core Engine Migration
- [ ] Create `src/transcribe/candle_engine.rs` to house the new `CandleEngine`.
- [ ] Port Mel-filterbank calculation from C++ to native Rust (using `candle-transformers` utils).
- [ ] Implement the Whisper Encoder/Decoder forward passes.
- [ ] Set up the token streaming and decoding logic (Greedy/Beam Search).

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
    - [ ] `dev-voice` transcribes audio using the `Large-v3-Turbo` model via Candle.
    - [ ] Speculative Decoding toggle is functional and provides measurable speedup.
    - [ ] End-to-end latency is consistently below 150ms for short sentences.
    - [ ] VRAM usage is optimized (< 2GB for the Speculative pair).

## Future Roadmap
- [ ] Support for distilled English-only models for even lower latency.
- [ ] Integration with Ollama for unified model orchestration.
- [ ] Real-time "Lookahead" decoding (streaming while speaking).
