# Production Polish Tasks

**Status**: Post-v0.2.0 Launch Preparation
**Created**: 2025-12-18
**Priority**: Optional improvements before public release

---

## 🎯 **Current State**

✅ **Core Platform Support: 100% Complete**
- All platforms building (Linux x64, macOS ARM, macOS Intel)
- All GPU variants working (CUDA, Metal)
- 17/17 CI jobs passing
- 6/6 artifacts building automatically
- Tested on real hardware (macOS 26, RTX 4060 Ti)

**Ready to ship!** Items below are polish, not blockers.

---

## 🟡 **Nice-to-Have: Documentation**

### 1. Contributing Guidelines
**File**: `CONTRIBUTING.md`
**Priority**: Medium
**Time**: 1-2 hours

**Content:**
- [ ] Code style requirements (rustfmt + clippy)
- [ ] How to run tests locally
- [ ] PR submission process
- [ ] Commit message format (conventional commits?)
- [ ] How to test on different platforms
- [ ] Where to ask questions

**Why:**
- Makes it easy for contributors to get started
- Sets expectations for code quality
- Reduces back-and-forth on PRs

---

### 2. Code of Conduct
**File**: `CODE_OF_CONDUCT.md`
**Priority**: Medium
**Time**: 15 minutes

**Tasks:**
- [ ] Adopt Contributor Covenant 2.1
- [ ] Add contact email for violations
- [ ] Link from README

**Template:** https://www.contributor-covenant.org/version/2/1/code_of_conduct/

**Why:**
- Standard for open source projects
- Creates welcoming environment
- Required for some package managers

---

### 3. Architecture Documentation
**File**: `ARCHITECTURE.md`
**Priority**: Low
**Time**: 2-3 hours

**Content:**
- [ ] System overview diagram
- [ ] Module structure explanation
- [ ] Audio pipeline (CPAL → resampling → Whisper)
- [ ] IPC protocol (client ↔ daemon)
- [ ] Text injection flow (enigo/clipboard)
- [ ] GPU acceleration architecture
- [ ] Configuration system
- [ ] Testing strategy

**Why:**
- Helps contributors understand codebase
- Useful for complex features
- Good for blog posts/talks

**Note:** Can extract from existing docs:
- `docs/project/phase4-implementation-summary.md`
- Current README Technical Details section

---

### 4. Platform-Specific Install Guides
**Files**: `docs/installation/` directory
**Priority**: Low
**Time**: 3-4 hours

**Create guides for:**
- [ ] `linux-fedora.md` - Fedora/RHEL specific
- [ ] `linux-ubuntu.md` - Ubuntu/Debian specific
- [ ] `linux-arch.md` - Arch Linux (AUR package info)
- [ ] `macos.md` - macOS permissions, troubleshooting
- [ ] `build-from-source.md` - Detailed build guide
- [ ] `gpu-setup.md` - CUDA/Metal setup details

**Why:**
- Better SEO (search for "hyprvoice fedora install")
- Platform-specific troubleshooting
- Easier to maintain than one huge README

**Can defer:** README already has essential info. Only create if users request.

---

## 🟡 **Nice-to-Have: CI/CD Optimizations**

### 5. Faster CI Caching
**Priority**: Low
**Time**: 30 minutes
**Savings**: ~20-30s per CI run

**Tasks:**
- [ ] Replace manual cargo cache with `Swatinem/rust-cache@v2`
- [ ] Benchmark CI times before/after
- [ ] Update workflow

**Example:**
```yaml
- uses: Swatinem/rust-cache@v2
  with:
    shared-key: "hyprvoice"
```

**Why:**
- Smarter caching (auto-detects what to cache)
- Faster CI runs
- Less maintenance

**Trade-off:** Another dependency. Current caching works fine.

---

### 6. CI Performance Benchmarks
**Priority**: Low
**Time**: 1 hour

**Tasks:**
- [ ] Document current CI times per job
- [ ] Identify slowest jobs
- [ ] Optimize if needed (e.g., parallel builds)
- [ ] Set up CI time monitoring

**Current times (from last run):**
- Fastest: Rustfmt (6s)
- Slowest: Build CUDA (18m) - expected due to whisper.cpp compilation

**Why:**
- Track performance regressions
- Identify optimization opportunities

**Can defer:** CI is already fast enough.

---

## 🟡 **Nice-to-Have: Code Coverage**

### 7. Coverage Badge
**Priority**: Low
**Time**: 15 minutes

**Tasks:**
- [ ] Get Codecov token from https://codecov.io
- [ ] Add token to GitHub secrets
- [ ] Verify uploads work
- [ ] Add badge to README: `[![codecov](https://codecov.io/gh/...)](https://codecov.io/gh/...)`

**Current coverage:** Generated but not tracked (tarpaulin runs in CI)

**Why:**
- Shows project quality
- Motivates better testing
- Identifies untested code

---

### 8. Coverage Dashboard
**Priority**: Low
**Time**: 30 minutes

**Tasks:**
- [ ] Set up Codecov project
- [ ] Configure coverage thresholds (e.g., 50% minimum)
- [ ] Add coverage trends to CI
- [ ] Block PRs if coverage drops significantly

**Why:**
- Prevents regressions in test coverage
- Encourages contributors to add tests

**Trade-off:** Adds another service dependency. May annoy contributors.

---

## 🔵 **Future: Out-of-Scope Platforms**

### 9. Windows Support
**Status**: Code exists, no CI/testing
**Priority**: Deferred
**Time**: 4-8 hours

**Current state:**
- ✅ Windows dependencies in Cargo.toml (line 74-75)
- ✅ enigo supports Windows (SendInput API)
- ❌ No CI runner
- ❌ No testing
- ❌ No artifacts

**If pursued:**
- [ ] Add `windows-latest` to CI matrix
- [ ] Install Windows dependencies (Visual Studio Build Tools)
- [ ] Fix any Windows-specific build issues
- [ ] Test on real Windows machine
- [ ] Add Windows artifact upload
- [ ] Document Windows installation in README

**Challenges:**
- Whisper.cpp CUDA on Windows is complex
- Need Windows machine for testing
- Different permissions model than Unix
- Text injection might need admin rights

**Recommendation:** Wait for user demand. Ship Linux + macOS first.

---

### 10. Linux ARM64 Support
**Status**: No CI runner
**Priority**: Deferred
**Time**: 2-4 hours

**Use cases:**
- Raspberry Pi
- ARM servers (AWS Graviton)
- Pinebook laptops

**Challenges:**
- GitHub Actions doesn't offer ARM64 Linux runners (standard tier)
- Would need self-hosted runner OR cross-compilation
- Testing requires ARM hardware

**If pursued:**
- [ ] Set up cross-compilation for `aarch64-unknown-linux-gnu`
- [ ] Add to CI workflow (if self-hosted runner)
- [ ] Test on ARM hardware (Raspberry Pi 4+)
- [ ] Add ARM64 Linux artifact
- [ ] Document ARM-specific requirements

**Recommendation:** Deferred until requested. Niche use case.

---

### 11. FreeBSD / Other Unix
**Status**: Not in scope
**Priority**: Very Low
**Time**: Unknown

**Challenges:**
- No dependencies in Cargo.toml
- CPAL FreeBSD support unknown
- enigo FreeBSD support unknown
- No CI runners available
- Very niche audience

**Recommendation:** Only pursue if significant user demand.

---

## 📋 **Recommended Priorities**

### **Before Public Launch** (Do These)
1. ✅ All platform support working
2. ✅ Automated artifacts
3. ✅ Professional README
4. 🟡 **CONTRIBUTING.md** ← Do this (30 min)
5. 🟡 **CODE_OF_CONDUCT.md** ← Do this (15 min)

**Total time: ~45 minutes to be fully launch-ready**

### **After Launch** (Based on Feedback)
6. Platform install guides (if users struggle)
7. ARCHITECTURE.md (if contributors need it)
8. CI optimizations (if CI gets slow)
9. Coverage badge (if quality questions arise)

### **Only If Requested**
10. Windows support
11. Linux ARM64 support
12. FreeBSD support

---

## 🎯 **Decision Points**

### **Question 1: Pre-Launch Documentation**

**Do before launch?**
- ✅ **YES**: CONTRIBUTING.md, CODE_OF_CONDUCT.md (45 min, standard for OSS)
- ❌ **NO**: ARCHITECTURE.md, install guides (can add based on demand)

### **Question 2: Out-of-Scope Platforms**

**Windows, Linux ARM64, FreeBSD:**
- ✅ **Document as "not yet supported"** in README
- ✅ **Accept PRs if contributors want to add them**
- ❌ **Don't spend time implementing now** (wait for demand)

---

## 📝 **Next Steps**

**To launch today:**
1. Create CONTRIBUTING.md (30 min)
2. Add CODE_OF_CONDUCT.md (15 min)
3. Push to GitHub
4. Make repo public
5. Post on Reddit/HN/Twitter

**That's it!** Everything else can be done post-launch.

---

## 📊 **Launch Checklist**

- [x] Core functionality working
- [x] Multi-platform support
- [x] CI/CD pipeline (100%)
- [x] Automated artifacts (6 variants)
- [x] Professional README
- [x] GPU acceleration (CUDA + Metal)
- [x] Tested on real hardware
- [ ] CONTRIBUTING.md (30 min)
- [ ] CODE_OF_CONDUCT.md (15 min)
- [ ] GitHub release with all artifacts
- [ ] Make repository public
- [ ] Announce on social media

**ETA to public launch: 45 minutes of work + announcement**

---

## 💭 **Philosophy**

**Ship fast, iterate based on feedback.**

You have a production-ready, multi-platform voice dictation tool that:
- Works better than most closed-source alternatives
- Fully local and private
- GPU accelerated
- Professional quality

The community will tell you what documentation they need. Launch first, polish later.

**Don't let perfect be the enemy of good.** 🚀
