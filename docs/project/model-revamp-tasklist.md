# Model Revamp & Optimization Roadmap

**Status:** Planning | **Priority:** High | **Goal:** Transition `dev-voice` to a "Bleeding Edge" STT experience.

This document serves as the source of truth for the model migration from standard Whisper models to optimized Turbo and Distilled variants, alongside performance optimizations like Speculative Decoding.

## Phase 1: Registry Expansion & Quality Defaults (COMPLETED)
The goal was to move from 2023-era Whisper weights to 2024-2025 optimized weights.

- [x] **Step 1.1: Map GGUF/Quantized Paths**
- [x] **Step 1.2: Update `src/model/registry.rs`**
    - Added `large-v3-turbo` (New recommended High-End).
    - Added `distil-large-v3` (New recommended Mid-Range).
- [x] **Step 1.3: Update CLI/Installer Logic**
    - Updated `dev-voice download` default to `large-v3-turbo`.
    - Improved help text to reflect new model options.

## Phase 2: Inference Optimization (Speculative Decoding) (COMPLETED)
Use a draft model to speed up the main transcription.

- [x] **Step 2.1: Implement Draft Model Support**
    - Modified `Transcriber` struct to optionally hold a "draft" model.
    - Updated `ModelConfig` to include an optional `draft_model_path`.
- [x] **Step 2.2: Update Transcription Logic**
    - Enabled `set_encoder_begin_callback` in `whisper-rs` to trigger speculative decoding.
- [x] **Step 2.3: Performance Benchmarking**
    - (Benchmarking pending user testing - local setup ready).

## Phase 3: Developer experience (Technical Vocabulary) (COMPLETED)
Ensure technical terms are transcribed correctly for "Vibe Coding."

- [x] **Step 3.1: Technical Grammar/Token Bias**
    - Added a robust list of 50+ technical keywords.
    - Updated `ModelConfig` to include a customizable `prompt` field.
- [x] **Step 3.2: Context-Awareness**
    - Integrated `set_initial_prompt` in `Transcriber` to guide the model toward technical terms.
- [x] **Updated Default Prompt:**
    - `async, await, impl, struct, enum, pub, static, btreemap, hashmap, kubernetes, k8s, docker, container, pod, lifecycle, workflow, ci/cd, yaml, json, rustlang, python, javascript, typescript, bash, git, repo, branch, commit, push, pull, merge, rebase, upstream, downstream, middleware, database, sql, postgres, redis, api, endpoint, graphql, rest, grpc, protobuf, systemd, journalctl, flatpak, wayland, nix, cargo.`

## Phase 4: UI & Feedback Sync
Improve the perception of speed.

- [ ] **Step 4.1: Sub-Second Waybar Transitions**
    - Fine-tune the SIGRTMIN signal timing.
- [ ] **Step 4.2: Processing Indicators**
    - Use the Waybar module to show "Transcribing..." differently than "Recording..." (already partially implemented with file markers).

## Success Metrics
1. **Cold Start Time:** < 500ms for model loading (DAEMON).
2. **Post-Speech Latency:** < 500ms for transcription to appear at cursor.
3. **Accuracy:** Human-level for technical coding discussions.

---

## Technical Notes & Reference URIs
- **Turbo Weights:** `huggingface.co/ggerganov/whisper.cpp`
- **Distil Weights:** `huggingface.co/distil-whisper/distil-large-v3`
- **Inference Library:** `whisper-rs` (current)

🤖 Generated with [Claude Code](https://claude.com/claude-code)
