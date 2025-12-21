# Performance Benchmarks Implementation

**Status:** Planning
**Estimated Time:** 4-6 hours
**Priority:** Medium
**Owner:** TBD

## Overview

Implement comprehensive performance benchmarks using Criterion.rs to track performance of critical paths (audio processing, resampling, daemon protocol) and detect performance regressions in CI.

## Goals

- Establish performance baselines for critical operations
- Detect performance regressions automatically in CI
- Provide visibility into optimization opportunities
- Enable data-driven performance decisions

## Non-Goals

- Full Whisper model benchmarks (requires large model files, too slow for CI)
- Memory profiling (Phase 7, optional)
- Profiler integration (Phase 7, optional)

---

## Implementation Checklist

### Phase 1: Setup & Infrastructure (~30 min)

- [ ] Add `criterion` to `[dev-dependencies]` in Cargo.toml

  ```toml
  [dev-dependencies]
  criterion = { version = "0.5", features = ["html_reports"] }
  ```

- [ ] Add benchmark configurations to Cargo.toml

  ```toml
  [[bench]]
  name = "audio_benchmarks"
  harness = false

  [[bench]]
  name = "daemon_benchmarks"
  harness = false
  ```

- [ ] Create benchmark directory structure

  ```
  benches/
  ├── audio_benchmarks.rs
  └── daemon_benchmarks.rs
  ```

- [ ] Add `.criterion/` to `.gitignore`

- [ ] Test that `cargo bench` works with placeholder benchmark

### Phase 2: Audio Processing Benchmarks (~1-2 hours)

#### 2.1 Resampling Benchmarks

- [ ] Expose `resample()` function for benchmarking
  - Option A: Add `pub(crate)` visibility
  - Option B: Add `bench` feature flag (recommended)
  - Option C: Re-implement test version in benches/

- [ ] Create helper function to generate test audio signals

  ```rust
  fn generate_test_audio(duration: f32, sample_rate: u32) -> Vec<f32>
  ```

- [ ] Benchmark: 44.1kHz → 16kHz resampling
  - [ ] 1 second duration
  - [ ] 5 second duration
  - [ ] 10 second duration
  - [ ] 30 second duration

- [ ] Benchmark: 48kHz → 16kHz resampling
  - [ ] 1 second duration
  - [ ] 5 second duration
  - [ ] 10 second duration
  - [ ] 30 second duration

- [ ] Benchmark: 16kHz → 48kHz upsampling (5s duration)

- [ ] Benchmark: FFT vs Linear resampling comparison
  - [ ] Expose both `resample()` (FFT) and `resample_linear()` functions
  - [ ] Create comparison benchmark group
  - [ ] Measure quality vs performance trade-off

#### 2.2 Audio Format Conversion Benchmarks

- [ ] Benchmark: Stereo to mono conversion
  - [ ] 1 second @ 48kHz
  - [ ] 5 seconds @ 48kHz
  - [ ] 10 seconds @ 48kHz

- [ ] Benchmark: i16 to f32 conversion
  - [ ] Small buffer (1024 samples)
  - [ ] Medium buffer (48000 samples - 1s @ 48kHz)
  - [ ] Large buffer (480000 samples - 10s @ 48kHz)

#### 2.3 Memory Allocation Benchmarks

- [ ] Benchmark: Pre-allocated vs dynamic Vec growth
- [ ] Benchmark: Different buffer sizes for chunked processing

### Phase 3: Daemon/Protocol Benchmarks (~1 hour)

#### 3.1 Protocol Serialization

- [ ] Benchmark: Serialize `DaemonRequest::Ping`
- [ ] Benchmark: Serialize `DaemonRequest::StartRecording`
- [ ] Benchmark: Serialize `DaemonRequest::StopRecording`
- [ ] Benchmark: Serialize `DaemonRequest::Shutdown`

#### 3.2 Protocol Deserialization

- [ ] Benchmark: Deserialize `DaemonResponse::Ok`
- [ ] Benchmark: Deserialize `DaemonResponse::Recording`
- [ ] Benchmark: Deserialize `DaemonResponse::Success` (with text)
- [ ] Benchmark: Deserialize `DaemonResponse::Error`

#### 3.3 Round-Trip Performance

- [ ] Benchmark: Full JSON encode/decode cycle for typical request
- [ ] Benchmark: Large response payload (500+ chars transcribed text)

### Phase 4: Baseline Establishment (~30 min)

- [ ] Run initial benchmarks: `cargo bench`
- [ ] Save baseline: `cargo bench -- --save-baseline main`
- [ ] Document baseline results in this file (see Results section below)
- [ ] Create performance expectations table
- [ ] Verify benchmarks are stable (run 3x, check variance)

### Phase 5: CI Integration (~30 min)

- [ ] Create new CI job: `benchmark`
  - [ ] Add to `.github/workflows/ci.yml`
  - [ ] Configure to run on Linux only
  - [ ] Run benchmarks: `cargo bench --no-fail-fast`
  - [ ] Determine when to run (every push vs. nightly)

- [ ] Add benchmark result storage
  - [ ] Option A: Use `github-action-benchmark` action
  - [ ] Option B: Store results as artifacts
  - [ ] Option C: Store in gh-pages branch (recommended)

- [ ] Configure regression detection
  - [ ] Set threshold (recommend: 10% slowdown = warning)
  - [ ] Add PR comment with comparison results
  - [ ] Configure auto-push to results branch

- [ ] Test CI integration on a test branch

### Phase 6: Documentation (~30 min)

- [ ] Create benchmark usage documentation
  - [ ] How to run benchmarks locally
  - [ ] How to compare with baseline
  - [ ] How to interpret results
  - [ ] Performance expectations table

- [ ] Update main README with benchmark info
  - [ ] Add "Performance" section
  - [ ] Link to benchmark documentation
  - [ ] Add badges for benchmark results (if using github-action-benchmark)

- [ ] Document performance targets
  - [ ] Resampling: Target X ms for Y seconds of audio
  - [ ] Protocol: Target sub-microsecond serialization
  - [ ] Set acceptable regression threshold

- [ ] Add benchmark results to this document (see Results section)

### Phase 7: Advanced Features (Optional, ~1-2 hours)

- [ ] Add flamegraph support

  ```toml
  pprof = { version = "0.13", features = ["flamegraph", "criterion"] }
  ```

- [ ] Add memory profiling integration

- [ ] Create comparative analysis benchmarks
  - [ ] Algorithm A vs Algorithm B comparisons
  - [ ] Quality vs speed trade-off analysis

- [ ] Add custom plots and visualizations

---

## Detailed Implementation Guide

### Benchmark Structure Template

```rust
// benches/audio_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::f32::consts::PI;

// Helper function to generate test audio
fn generate_test_audio(duration: f32, sample_rate: u32) -> Vec<f32> {
    let samples = (duration * sample_rate as f32) as usize;
    (0..samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            (2.0 * PI * 440.0 * t).sin() // 440Hz sine wave
        })
        .collect()
}

fn bench_resampling(c: &mut Criterion) {
    let mut group = c.benchmark_group("resampling");

    // Configure group settings
    group.sample_size(10); // Fewer samples for long benchmarks

    // Test different durations
    for duration in [1.0, 5.0, 10.0, 30.0] {
        let input = generate_test_audio(duration, 44100);

        group.bench_with_input(
            BenchmarkId::new("44100_to_16000", format!("{}s", duration)),
            &input,
            |b, input| {
                b.iter(|| {
                    // Would call internal resample function
                    // Note: Need to expose this function
                    black_box(dev_voice::audio::resample(input, 44100, 16000))
                });
            }
        );
    }

    group.finish();
}

fn bench_stereo_to_mono(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_conversion");

    for duration in [1.0, 5.0, 10.0] {
        let stereo: Vec<f32> = generate_test_audio(duration, 48000)
            .iter()
            .flat_map(|&s| [s, s * 0.9]) // Simulate stereo
            .collect();

        group.bench_with_input(
            BenchmarkId::new("stereo_to_mono", format!("{}s", duration)),
            &stereo,
            |b, stereo| {
                b.iter(|| {
                    // Would call internal function
                    black_box(dev_voice::audio::stereo_to_mono(stereo))
                });
            }
        );
    }

    group.finish();
}

criterion_group!(benches, bench_resampling, bench_stereo_to_mono);
criterion_main!(benches);
```

### Daemon Benchmark Template

```rust
// benches/daemon_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dev_voice::daemon::protocol::{DaemonRequest, DaemonResponse};

fn bench_protocol_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_serialize");

    group.bench_function("ping", |b| {
        let request = DaemonRequest::Ping;
        b.iter(|| {
            black_box(serde_json::to_string(&request).unwrap())
        });
    });

    group.bench_function("start_recording", |b| {
        let request = DaemonRequest::StartRecording { max_duration: 300 };
        b.iter(|| {
            black_box(serde_json::to_string(&request).unwrap())
        });
    });

    group.bench_function("stop_recording", |b| {
        let request = DaemonRequest::StopRecording;
        b.iter(|| {
            black_box(serde_json::to_string(&request).unwrap())
        });
    });

    group.finish();
}

fn bench_protocol_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_deserialize");

    group.bench_function("success_response", |b| {
        let json = r#"{"type":"Success","text":"This is a test transcription result"}"#;
        b.iter(|| {
            black_box(serde_json::from_str::<DaemonResponse>(json).unwrap())
        });
    });

    group.bench_function("error_response", |b| {
        let json = r#"{"type":"Error","message":"Already recording"}"#;
        b.iter(|| {
            black_box(serde_json::from_str::<DaemonResponse>(json).unwrap())
        });
    });

    group.finish();
}

fn bench_round_trip(c: &mut Criterion) {
    c.bench_function("request_round_trip", |b| {
        let request = DaemonRequest::StartRecording { max_duration: 300 };
        b.iter(|| {
            let json = serde_json::to_string(&request).unwrap();
            black_box(serde_json::from_str::<DaemonRequest>(&json).unwrap())
        });
    });
}

criterion_group!(
    benches,
    bench_protocol_serialization,
    bench_protocol_deserialization,
    bench_round_trip
);
criterion_main!(benches);
```

### Exposing Internal Functions

**Option 1: Feature Flag (Recommended)**

```rust
// In src/audio/mod.rs

#[cfg(any(test, feature = "bench"))]
pub fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    // existing implementation
}

#[cfg(not(any(test, feature = "bench")))]
fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    // existing implementation
}
```

Then run benchmarks with:

```bash
cargo bench --features bench
```

**Option 2: pub(crate) Visibility**

```rust
// In src/audio/mod.rs
pub(crate) fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    // existing implementation
}
```

Simpler but exposes to entire crate.

### CI Configuration

```yaml
# Add to .github/workflows/ci.yml

benchmark:
  name: Performance Benchmarks
  runs-on: ubuntu-latest
  # Only run on main or when explicitly requested
  if: github.ref == 'refs/heads/main' || contains(github.event.head_commit.message, '[bench]')
  steps:
    - uses: actions/checkout@v4

    - name: Install Rust toolchain
      uses: dtolnay/rust-toolchain@stable

    - name: Install Linux dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y libclang-dev libasound2-dev pkg-config libxkbcommon-dev

    - name: Cache cargo registry
      uses: actions/cache@v4
      with:
        path: ~/.cargo/registry
        key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

    - name: Cache cargo index
      uses: actions/cache@v4
      with:
        path: ~/.cargo/git
        key: ${{ runner.os }}-cargo-git-${{ hashFiles('**/Cargo.lock') }}

    - name: Run benchmarks
      run: cargo bench --features bench -- --output-format bencher | tee output.txt

    - name: Store benchmark result
      uses: benchmark-action/github-action-benchmark@v1
      with:
        tool: "cargo"
        output-file-path: output.txt
        github-token: ${{ secrets.GITHUB_TOKEN }}
        auto-push: true
        # Alert if performance degrades by more than 10%
        alert-threshold: "110%"
        comment-on-alert: true
        fail-on-alert: false
```

---

## Performance Targets

### Audio Processing

| Operation           | Input       | Target  | Acceptable Range |
| ------------------- | ----------- | ------- | ---------------- |
| Resample 44.1→16kHz | 1s          | < 5ms   | 2-8ms            |
| Resample 44.1→16kHz | 30s         | < 100ms | 50-150ms         |
| Resample 48→16kHz   | 1s          | < 5ms   | 2-8ms            |
| Stereo→Mono         | 5s @ 48kHz  | < 500μs | 100μs-1ms        |
| i16→f32 conversion  | 48k samples | < 100μs | 50-200μs         |

**Throughput Target:** At least 100x realtime for resampling (i.e., process 1s of audio in <10ms)

### Protocol/IPC

| Operation                | Target  | Acceptable Range |
| ------------------------ | ------- | ---------------- |
| Serialize Ping           | < 100ns | 50-200ns         |
| Serialize StartRecording | < 200ns | 100-400ns        |
| Deserialize Response     | < 500ns | 200ns-1μs        |
| Round-trip encode/decode | < 1μs   | 500ns-2μs        |

**Throughput Target:** > 1M serializations/sec

---

## Baseline Results

**Status:** Not yet established

After running initial benchmarks, record results here:

### Audio Benchmarks

```
TBD - Run: cargo bench --features bench audio_benchmarks
```

### Daemon Benchmarks

```
TBD - Run: cargo bench daemon_benchmarks
```

### System Info

```
TBD - Record:
- CPU model
- RAM
- Rust version
- Criterion version
- Date of baseline
```

---

## Running Benchmarks

### Locally

```bash
# Run all benchmarks
cargo bench --features bench

# Run specific benchmark file
cargo bench --features bench audio_benchmarks

# Run specific benchmark function
cargo bench --features bench resampling

# Save baseline
cargo bench --features bench -- --save-baseline main

# Compare with baseline
cargo bench --features bench -- --baseline main

# Open HTML reports
open target/criterion/report/index.html
```

### In CI

Benchmarks run automatically on:

- Every push to `main` branch
- PRs with `[bench]` in commit message

Results are stored in the `gh-pages` branch and viewable at:

```
https://<username>.github.io/<repo>/dev/bench/
```

---

## Success Criteria

- [ ] All benchmarks run successfully without errors
- [ ] Baseline established and documented
- [ ] CI runs benchmarks and stores results
- [ ] Performance regression detection works (test with intentional slowdown)
- [ ] HTML reports are generated and accessible
- [ ] Documentation is clear and complete
- [ ] Benchmarks run in reasonable time (< 5 minutes total)

---

## Key Decisions

### 1. Benchmark Frequency

**Decision:** Run on every push to `main`, optional on PRs
**Rationale:** Balance between coverage and CI time/cost

### 2. Performance Targets

**Decision:** Establish baselines first, then set ±15% tolerance
**Rationale:** Need real-world data before setting targets

### 3. Whisper Model Benchmarks

**Decision:** Skip for now, add as separate manual benchmark suite
**Rationale:** Too slow for CI, requires large model files

### 4. Result Storage

**Decision:** Use gh-pages branch via github-action-benchmark
**Rationale:** Free, automatic, good visualization, historical tracking

### 5. Function Exposure

**Decision:** Use feature flag (`bench`) for internal functions
**Rationale:** Clean separation, doesn't pollute public API

---

## Future Enhancements

After initial implementation, consider:

- [ ] Flamegraph integration for profiling
- [ ] Memory profiling with valgrind/heaptrack
- [ ] Comparative benchmarks (algorithm A vs B)
- [ ] Benchmark dashboard with trends over time
- [ ] Per-PR benchmark comparison comments
- [ ] Benchmark against industry baselines (FFmpeg resampling, etc.)
- [ ] Add statistical analysis (outlier detection, variance)

---

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [github-action-benchmark](https://github.com/benchmark-action/github-action-benchmark)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [How to Write Fast Rust Code](https://likebike.com/posts/How_To_Write_Fast_Rust_Code.html)

---

**Last Updated:** 2025-12-19
**Next Review:** After Phase 4 completion (baseline establishment)
