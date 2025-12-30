# hyprvoice Open Source Preparation Roadmap

**Goal:** Prepare hyprvoice for open source release with cross-platform support (Linux distros + macOS)

**Status:** Planning
**Target:** After completing Weeks 1-3 of code quality improvements
**Created:** 2025-12-16
**Last Updated:** 2025-12-16

---

## Prerequisites (Current Work - In Progress)

Before starting open source prep, complete foundation work:

### ✅ Week 1: Critical Fixes (COMPLETED)
- [x] Code quality standards (clippy, rustfmt)
- [x] CI/CD pipeline (GitHub Actions)
- [x] Eliminate production panics (7 → 0)
- [x] Fix race conditions
- [x] Add IPC timeouts

### ✅ Week 2: Testing Infrastructure (COMPLETED 71%)
- [x] Test dependencies (mockall, serial_test)
- [x] Integration test framework
- [x] Protocol tests (13 tests)
- [x] Lifecycle tests (3 tests)
- [x] Code coverage tooling (14.08% baseline)
- [ ] Audio resampling tests (optional)
- [ ] PipeWire mocking (optional)

### 🔄 Week 3: Code Refactoring (IN PROGRESS)
- [ ] Extract 200-line audio duplication
- [ ] Move CLI handlers to commands/ module
- [ ] Improve code organization

**Target State:**
- Code quality: 9.0+/10
- Test coverage: 50%+
- Zero technical debt from audit
- Clean, maintainable architecture

---

## Phase 4: Platform Abstraction (2-3 weeks)

**Goal:** Make codebase platform-agnostic

### Task 1: Audio Capture Abstraction (Week 4)

**Current blocker:** Hardcoded PipeWire (Linux-only)

**Implementation:**
```rust
// src/audio/mod.rs
pub trait AudioCapture {
    fn capture(&self, duration: Option<u32>) -> Result<Vec<f32>>;
    fn sample_rate(&self) -> u32;
    fn stop(&self);
}

// Platform implementations
// src/audio/pipewire.rs    - Linux (current)
// src/audio/coreaudio.rs   - macOS (new)

// Platform detection
pub fn default_audio_source() -> Box<dyn AudioCapture> {
    #[cfg(target_os = "linux")]
    Box::new(PipeWireCapture::new())

    #[cfg(target_os = "macos")]
    Box::new(CoreAudioCapture::new())
}
```

**Subtasks:**
- [ ] Design AudioCapture trait API
- [ ] Refactor PipeWire code to implement trait
- [ ] Implement CoreAudio backend (macOS)
- [ ] Add platform feature flags to Cargo.toml
- [ ] Test on both platforms

**Files to modify:**
- `src/audio/mod.rs` - Add trait definition
- `src/audio/pipewire.rs` - Rename from capture.rs, implement trait
- `src/audio/coreaudio.rs` - New macOS implementation
- `Cargo.toml` - Add platform features

---

### Task 2: Text Injection Abstraction (Week 4)

**Current blocker:** Wayland/X11 only (no macOS support)

**Implementation:**
```rust
// src/output/mod.rs
pub trait TextInjector {
    fn inject(&self, text: &str) -> Result<()>;
    fn supports_clipboard(&self) -> bool;
}

// Platform implementations
// src/output/wayland.rs  - wtype (current)
// src/output/x11.rs      - xdotool (current)
// src/output/macos.rs    - NSPasteboard + AppleScript (new)
```

**Subtasks:**
- [ ] Design TextInjector trait
- [ ] Split inject.rs into wayland.rs + x11.rs
- [ ] Implement macOS text injection
- [ ] Update terminal detection to be platform-aware
- [ ] Test clipboard vs typing on all platforms

**Files to modify:**
- `src/output/mod.rs` - Add trait + platform selection
- `src/output/wayland.rs` - Extract from inject.rs
- `src/output/x11.rs` - Extract from inject.rs
- `src/output/macos.rs` - New macOS implementation

---

### Task 3: Compositor Detection → Generic (Week 5)

**Current blocker:** Hyprland-specific terminal detection

**Implementation:**
```rust
// Make compositor-agnostic
// Support: Hyprland, Sway, KDE, GNOME, macOS
pub enum Platform {
    Wayland { compositor: Compositor },
    X11,
    MacOS,
}

pub enum Compositor {
    Hyprland,
    Sway,
    GNOME,
    KDE,
    Other(String),
}
```

**Subtasks:**
- [ ] Generic Wayland window detection (not just Hyprland)
- [ ] Support for Sway compositor
- [ ] Support for KDE/GNOME
- [ ] macOS window management
- [ ] Configuration for terminal apps per platform

**Files to modify:**
- `src/output/inject.rs` - Make detection generic
- `config.example.toml` - Add terminal configuration

---

### Task 4: Feature Flags & Conditional Compilation (Week 5)

**Add platform features to Cargo.toml:**

```toml
[features]
default = ["detect-platform"]

# Audio backends
pipewire = ["dep:pipewire", "dep:libspa"]
coreaudio = ["dep:coreaudio-sys"]

# Output backends
wayland = ["dep:wayland-client"]
x11 = ["dep:x11"]
macos-output = []

# Platform detection (auto-selects appropriate features)
detect-platform = []

# GPU backends (existing)
cuda = ["whisper-rs/cuda"]
rocm = ["whisper-rs/hipblas"]
vulkan = ["whisper-rs/vulkan"]
metal = ["whisper-rs/metal"]  # macOS
```

**Subtasks:**
- [ ] Add platform feature flags
- [ ] Update build scripts for conditional compilation
- [ ] Test builds on each platform
- [ ] Document feature combinations

---

## Phase 5: Documentation & Community (1-2 weeks)

**Goal:** Make project accessible and contributor-friendly

### Task 1: Installation Documentation (Week 6)

**Create:**
- [ ] `docs/installation/linux-arch.md`
- [ ] `docs/installation/linux-ubuntu.md`
- [ ] `docs/installation/linux-fedora.md`
- [ ] `docs/installation/macos.md`
- [ ] `docs/installation/build-from-source.md`

**Content:**
- System requirements per platform
- Dependency installation
- Binary installation
- Build from source
- Troubleshooting common issues

---

### Task 2: Development Documentation (Week 6)

**Create:**
- [ ] `ARCHITECTURE.md` - System design overview
  - Current: `docs/context/hyprvoice-complete.md` (good start!)
  - Expand with: Platform abstraction, trait design, module structure

- [ ] `CONTRIBUTING.md` - How to contribute
  - Code style guide (rustfmt + clippy)
  - PR process
  - Testing requirements
  - Commit message format

- [ ] `CODE_OF_CONDUCT.md` - Community standards
  - Use Contributor Covenant

- [ ] API Documentation (rustdoc)
  - Add `///` doc comments to public APIs
  - Generate with `cargo doc`
  - Host on docs.rs

**Subtasks:**
- [ ] Expand ARCHITECTURE.md from existing docs
- [ ] Write CONTRIBUTING.md
- [ ] Adopt CODE_OF_CONDUCT.md
- [ ] Add rustdoc comments to public APIs
- [ ] Set up docs.rs integration

---

### Task 3: User Documentation (Week 6)

**Create:**
- [ ] `docs/user-guide/configuration.md` - Config file guide
- [ ] `docs/user-guide/keybindings.md` - Setup per compositor
- [ ] `docs/user-guide/models.md` - Model selection guide
- [ ] `docs/user-guide/troubleshooting.md` - Common issues

**Expand README.md:**
- [ ] Add platform support matrix
- [ ] Add installation badges
- [ ] Add usage examples with GIFs/screenshots
- [ ] Link to documentation
- [ ] Add "Why hyprvoice?" section

---

## Phase 6: Release Infrastructure (1 week)

**Goal:** Automate builds and distribution

### Task 1: Cross-Platform CI (Week 7)

**Expand `.github/workflows/ci.yml`:**

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest]
    include:
      - os: ubuntu-latest
        target: x86_64-unknown-linux-gnu
        features: pipewire,wayland
      - os: macos-latest
        target: x86_64-apple-darwin
        features: coreaudio,macos-output,metal
```

**Subtasks:**
- [ ] Add macOS to CI matrix
- [ ] Test builds on both platforms
- [ ] Add feature flag validation
- [ ] Test coverage on both platforms

---

### Task 2: Release Automation (Week 7)

**Create `.github/workflows/release.yml`:**

```yaml
# Triggered on git tags (v*)
# Builds binaries for:
#   - Linux (x86_64, arm64)
#   - macOS (x86_64, arm64)
# Creates GitHub Release
# Uploads pre-built binaries
```

**Subtasks:**
- [ ] Create release workflow
- [ ] Set up cross-compilation
- [ ] Add version bumping automation
- [ ] Generate CHANGELOG.md from commits
- [ ] Test release process

---

### Task 3: Package Manager Setup (Week 7-8)

**Distribution channels:**

#### Cargo (Highest priority)
- [ ] Publish to crates.io
- [ ] Set up automatic publishing from tags

#### AUR (Arch Linux)
- [ ] Create PKGBUILD
- [ ] Submit to AUR
- [ ] Set up AUR auto-update bot

#### Homebrew (macOS)
- [ ] Create Homebrew formula
- [ ] Submit to homebrew-core (or create tap)
- [ ] Test installation

#### GitHub Releases
- [ ] Pre-built binaries
- [ ] Checksums
- [ ] Installation script

**Deferred (post-launch):**
- [ ] Debian/Ubuntu packages (apt)
- [ ] RPM packages (dnf/yum)
- [ ] Snap/Flatpak

---

## Phase 7: Community Setup (1 week)

**Goal:** Prepare for community contributions

### Task 1: Repository Structure (Week 8)

**Create:**
```
.github/
├── ISSUE_TEMPLATE/
│   ├── bug_report.md
│   ├── feature_request.md
│   └── platform_support.md
├── PULL_REQUEST_TEMPLATE.md
└── workflows/
    ├── ci.yml
    ├── release.yml
    └── coverage.yml (exists)

docs/
├── installation/
├── user-guide/
├── development/
└── architecture/

CONTRIBUTING.md
CODE_OF_CONDUCT.md
CHANGELOG.md
LICENSE (MIT - already exists)
```

**Subtasks:**
- [ ] Create issue templates
- [ ] Create PR template
- [ ] Organize documentation
- [ ] Create CHANGELOG.md

---

### Task 2: Labels & Project Management (Week 8)

**GitHub Labels:**
```
Platform:
- platform:linux
- platform:macos
- platform:windows (future)

Type:
- type:bug
- type:feature
- type:docs
- type:refactor

Difficulty:
- good-first-issue
- help-wanted
- advanced

Status:
- needs-triage
- needs-reproduction
- blocked
```

**Subtasks:**
- [ ] Set up label system
- [ ] Create project board
- [ ] Set up milestones for releases

---

### Task 3: Community Guidelines (Week 8)

**Create:**
- [ ] SUPPORT.md - How to get help
- [ ] SECURITY.md - Security policy & reporting
- [ ] GOVERNANCE.md - Project governance (if needed)

**Set up:**
- [ ] Discussions (GitHub Discussions)
- [ ] Discord/Matrix server? (optional)

---

## Timeline Overview

| Week | Phase | Focus | Deliverable |
|------|-------|-------|-------------|
| 1-3 | Prerequisite | Code quality & testing | ✅ Weeks 1-2 done, Week 3 in progress |
| 4 | Phase 4.1 | Audio & text abstraction | Platform traits implemented |
| 5 | Phase 4.2 | Compositor & feature flags | Cross-platform builds working |
| 6 | Phase 5 | Documentation | Docs site ready |
| 7 | Phase 6 | Release infrastructure | CI/CD + packages |
| 8 | Phase 7 | Community setup | Ready for contributors |
| 9+ | Launch | Public release | 🚀 Open source! |

---

## Platform Support Roadmap

### Tier 1 (Launch - Week 9)
- ✅ Linux - Arch, Ubuntu, Fedora
- ✅ macOS - Intel & Apple Silicon
- ✅ Audio: PipeWire (Linux), CoreAudio (macOS)
- ✅ Output: Wayland, X11, macOS

### Tier 2 (Post-Launch)
- More Linux distros
- NixOS support
- BSD variants?

### Tier 3 (Future)
- Windows? (if Whisper supports well)
- Mobile? (Android/iOS - big lift)

---

## Key Design Principles

### 1. Graceful Degradation
```rust
// Example: Feature detection
if !audio_backend_available() {
    eprintln!("PipeWire not found.");
    eprintln!("Install: sudo apt install pipewire");
    std::process::exit(1);
}
```

### 2. Platform Parity
- Core features work on all platforms
- Platform-specific features clearly documented
- No "second-class citizen" platforms

### 3. Low Barrier to Entry
- One-command install: `cargo install hyprvoice`
- Pre-built binaries for releases
- Clear error messages with solutions

### 4. Contributor Friendly
- Comprehensive documentation
- Good test coverage
- Welcoming community
- Clear contribution guidelines

---

## Success Metrics

**Pre-launch (Week 9):**
- ✅ Builds on Linux + macOS
- ✅ 50%+ code coverage
- ✅ All docs complete
- ✅ 3+ package managers

**3 months post-launch:**
- 100+ GitHub stars
- 10+ contributors
- 5+ distro packages
- Active issue/PR engagement

**6 months post-launch:**
- 500+ GitHub stars
- 50+ contributors
- Production usage examples
- Community-driven features

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| **macOS testing limited** | Set up macOS CI early, recruit macOS beta testers |
| **Platform bugs hard to reproduce** | Good logging, clear issue templates, platform tags |
| **Dependency conflicts** | Feature flags, optional dependencies, clear docs |
| **Community management overhead** | Clear guidelines, good templates, automation |
| **Maintenance burden** | Good tests, CI/CD, helpful contributor docs |

---

## Next Steps (After Week 3)

1. **Review this roadmap** - Adjust based on lessons learned
2. **Start Phase 4** - Begin audio abstraction work
3. **Set up macOS testing** - Get access to macOS machine or CI
4. **Draft ARCHITECTURE.md** - Expand from existing docs
5. **Create project board** - Track open source prep tasks

---

## Notes

- This roadmap assumes completion of Weeks 1-3 first
- Timeline is aggressive but achievable
- Can adjust priorities based on community interest
- Platform abstraction is the hardest part
- Documentation is just as important as code
- Community building takes time - be patient

**Let's make hyprvoice an amazing open source project!** 🚀
