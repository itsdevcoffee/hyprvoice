# CI/CD Task List

**Status as of 2025-12-18**: ✅ **100% CI SUCCESS** - 17/17 jobs passing + 6/6 artifacts building

---

## 🎉 **COMPLETED - Full Platform Support Achieved!**

### **CI/CD Infrastructure** ✅
- [x] Multi-platform build matrix (Linux, macOS ARM, macOS Intel)
- [x] Fix macOS 15 ARM build failure (whisper-rs git version)
- [x] Fix macOS 26 audio configuration (device default config)
- [x] Add Metal GPU feature support
- [x] Add CUDA GPU support with NVIDIA container
- [x] Fix all linting issues (Rustfmt, Clippy)
- [x] Fix all Linux dependencies (ALSA, xkbcommon, pkg-config)
- [x] Fix C++17 standard for whisper.cpp git version
- [x] Verify billing for Intel runners (macos-15-large)
- [x] Add automated artifact uploads (6 variants)

### **Artifacts Successfully Building** ✅
1. [x] hyprvoice-linux-x64 (CPU-only, universal Linux)
2. [x] hyprvoice-linux-x64-cuda (NVIDIA GPUs with CUDA)
3. [x] hyprvoice-macos-arm64 (M1/M2/M3/M4 CPU-only)
4. [x] hyprvoice-macos-intel (Intel Macs CPU-only)
5. [x] hyprvoice-macos-14-arm64-metal (M1+ with Metal GPU, macOS 14)
6. [x] hyprvoice-macos-15-arm64-metal (M1+ with Metal GPU, macOS 15/26)

### **Testing** ✅
- [x] Tested on macOS 26 (Tahoe) - Audio capture working
- [x] Verified daemon mode works
- [x] Verified Metal GPU acceleration loads model to VRAM

---

## 🔴 **Previously Failing - Now RESOLVED**

### ~~1. Fix Clippy Linting Errors~~ ✅ FIXED
**Resolution**: Added missing Linux dependencies to clippy job

### ~~2. Fix Ubuntu Test Failures~~ ✅ FIXED
**Resolution**: Added libxkbcommon-dev for enigo Wayland support

### ~~3. Fix CUDA Build on Linux~~ ✅ FIXED
**Resolution**: Used NVIDIA CUDA container (nvidia/cuda:12.6.2-devel-ubuntu24.04)
**Impact**: Linux CUDA GPU support not building

**Tasks**:
- [ ] Investigate CUDA build failure logs
- [ ] Determine if CUDA toolkit needs to be installed in CI
- [ ] Fix build configuration or dependencies
- [ ] Verify CUDA feature builds successfully

**Options**:
- Install CUDA toolkit in CI runner
- Mock CUDA build for compilation check only
- Make CUDA optional and document requirements

---

## 🟡 **Medium Priority - Release Artifacts**

### 4. Add Release Binary Artifacts for macOS
**Status**: ⏳ Not Started
**Impact**: Users can't download pre-built macOS binaries

**Tasks**:
- [ ] Add release build step to CI workflow
- [ ] Build for macOS ARM64 (macos-15)
- [ ] Build for macOS Intel (macos-15-large)
- [ ] Build with Metal feature enabled
- [ ] Upload artifacts with platform-specific names:
  - `hyprvoice-macos-arm64`
  - `hyprvoice-macos-arm64-metal`
  - `hyprvoice-macos-intel`
- [ ] Test artifact downloads work
- [ ] Add artifact retention policy (90 days default)

**Example Workflow Addition**:
```yaml
- name: Build Release Binary
  run: cargo build --release --features metal

- name: Upload Artifact
  uses: actions/upload-artifact@v4
  with:
    name: hyprvoice-macos-arm64-metal
    path: target/release/hyprvoice
```

---

### 5. Add Release Binary Artifacts for Linux
**Status**: ⏳ Not Started
**Impact**: Users can't download pre-built Linux binaries

**Tasks**:
- [ ] Build for Linux x86_64 (ubuntu-latest)
- [ ] Build with CUDA feature (if fixed)
- [ ] Upload artifacts with platform-specific names:
  - `hyprvoice-linux-x64`
  - `hyprvoice-linux-x64-cuda` (if CUDA fixed)
- [ ] Test artifact downloads work
- [ ] Document runtime dependencies (ALSA, CUDA)

---

### 6. Add Artifact Download Instructions
**Status**: ⏳ Not Started
**Impact**: Users don't know how to download/use artifacts

**Tasks**:
- [ ] Create `INSTALLATION.md` or update `README.md`
- [ ] Document how to download artifacts from GitHub Actions
- [ ] Document platform-specific instructions
- [ ] Document GPU feature requirements
- [ ] Add troubleshooting section for common issues

---

## 🟢 **Low Priority - Improvements**

### 7. Optimize CI Caching
**Status**: ⏳ Not Started
**Impact**: CI runs slower than necessary

**Tasks**:
- [ ] Review cargo cache usage across jobs
- [ ] Add Rust toolchain caching
- [ ] Consider using rust-cache action for better caching
- [ ] Benchmark CI run times before/after

---

### 8. Add Code Coverage Reporting
**Status**: ⚠️ Pending in CI
**Job**: `Code Coverage`
**Impact**: No visibility into test coverage

**Tasks**:
- [ ] Wait for Code Coverage job to complete
- [ ] Fix any failures in coverage generation
- [ ] Integrate with Codecov or similar service
- [ ] Add coverage badge to README
- [ ] Set coverage thresholds

---

### 9. Consider Windows Support
**Status**: ⏳ Deferred
**Impact**: Windows users can't use hyprvoice

**Discussion Points**:
- Windows dependencies already in Cargo.toml (line 74-75)
- No CI runner configured
- No testing on Windows
- Significant effort required

**Tasks** (if pursued):
- [ ] Add `windows-latest` runner to matrix
- [ ] Test builds on Windows
- [ ] Fix any Windows-specific issues
- [ ] Add Windows artifact uploads
- [ ] Document Windows installation

---

### 10. Consider Linux ARM64 Support
**Status**: ⏳ Deferred
**Impact**: ARM Linux users (Raspberry Pi, etc.) can't use hyprvoice

**Discussion Points**:
- No ARM64 Linux runner available in GitHub Actions standard tier
- Would require self-hosted runner or cross-compilation
- Niche use case

**Tasks** (if pursued):
- [ ] Set up cross-compilation for aarch64-unknown-linux-gnu
- [ ] Test on ARM Linux hardware
- [ ] Add ARM64 Linux artifacts

---

## 📊 **Current CI Status Summary**

| Category | Passing | Total | Percentage |
|----------|---------|-------|------------|
| Platform Checks | 4/4 | 4 | 100% |
| Platform Tests (macOS) | 3/3 | 3 | 100% |
| Platform Tests (Linux) | 0/1 | 1 | 0% |
| GPU Builds (Metal) | 2/2 | 2 | 100% |
| GPU Builds (CUDA) | 0/1 | 1 | 0% |
| Code Quality | 1/2 | 2 | 50% |
| **Total** | **12/14** | **14** | **86%** |

---

## 🎯 **Next Session Goals**

1. **Immediate**: Fix Clippy and Ubuntu test failures → 100% CI
2. **Short-term**: Add macOS/Linux artifact downloads
3. **Discussion**: Windows, Linux ARM64, FreeBSD support decisions

---

## 📝 **Notes**

- macOS 15 fix required git version of whisper-rs (commit d38738df)
- Intel runner requires billing (now resolved)
- Metal GPU only works on Apple Silicon, not Intel Macs
- CUDA requires NVIDIA GPU and drivers at runtime
- All core platform compilation working - issues are test/quality related
