# hyprvoice Project Roadmap

**Status:** Active Development | **Last Updated:** 2025-12-20

Post model-revamp prioritized task list for public release preparation.

## High Priority (Public Release Prep)

### 1. Fix Speculative Decoding API
**Status:** Blocked - API Research Needed
**Priority:** Critical
**Effort:** Medium (4-8 hours)

- [ ] Research correct whisper-rs API for draft model integration
- [ ] Replace commented `set_encoder_begin_callback` with working implementation
- [ ] Test performance improvements (target: 30-50% speedup)
- [ ] Document actual vs theoretical performance gains
- [ ] Update model-revamp-tasklist.md with final benchmarks

**Why:** Core performance feature that unlocks the full speed potential.

### 2. Documentation Overhaul
**Status:** Not Started
**Priority:** Critical
**Effort:** Large (8-12 hours)

- [ ] Update main README.md with new features (Turbo models, technical vocab)
- [ ] Create QUICKSTART.md for new users
- [ ] Document integrations/ pattern for contributors
- [ ] Add screenshots/GIFs of Waybar integration in action
- [ ] Create ARCHITECTURE.md explaining daemon/client model
- [ ] Document GPU vs CPU fallback behavior

**Why:** Essential for onboarding new users and contributors.

### 3. Polybar Integration
**Status:** Not Started
**Priority:** High
**Effort:** Small (2-4 hours)

- [ ] Create integrations/polybar/ directory structure
- [ ] Write polybar-compatible status script (IPC or tail -f approach)
- [ ] Create module.ini config snippet
- [ ] Write install.sh for Polybar users
- [ ] Create README.md with Polybar-specific instructions
- [ ] Test on i3/bspwm environment
- [ ] Update integrations/README.md with Polybar entry

**Why:** Expands reach to X11/i3 users (large community).

## Medium Priority (Polish)

### 4. Performance Benchmarking
**Status:** Not Started
**Priority:** Medium
**Effort:** Medium (4-6 hours)

- [ ] Create benchmarking test suite (cargo bench)
- [ ] Compare old models (medium.en) vs new (large-v3-turbo)
- [ ] Measure with/without draft models
- [ ] Test across different audio lengths (5s, 30s, 60s)
- [ ] Document results in docs/research/benchmarks.md
- [ ] Add performance regression tests to CI

**Why:** Validate the "bleeding edge" claims with data.

### 5. Better First-Run Experience
**Status:** Not Started
**Priority:** Medium
**Effort:** Medium (6-8 hours)

- [ ] Auto-download large-v3-turbo + tiny.en on first run
- [ ] Interactive setup wizard (hyprvoice setup)
- [ ] Auto-detect desktop environment (Waybar/Polybar/none)
- [ ] Offer integration installation during setup
- [ ] Show welcome message with keybind instructions
- [ ] Create getting-started tutorial

**Why:** Reduce friction for new users.

### 6. CI/CD for New Models
**Status:** Not Started
**Priority:** Medium
**Effort:** Medium (4-6 hours)

- [ ] Update .github/workflows/ci.yml for new model registry
- [ ] Add model checksum verification to CI
- [ ] Build artifacts with new default models
- [ ] Automated release workflow (tags → GitHub Releases)
- [ ] Generate checksums for all binaries
- [ ] AUR package automation

**Why:** Streamlines releases and builds trust.

## Nice-to-Have (Future Enhancements)

### 7. Advanced Features
**Status:** Ideas Stage
**Priority:** Low
**Effort:** Large (12+ hours each)

- [ ] Project-aware vocabulary (detect .rs/.py/.ts, bias accordingly)
- [ ] Custom wake word for hands-free mode
- [ ] Multi-language support testing (Spanish, French, etc.)
- [ ] Noise cancellation preprocessing
- [ ] Voice commands (e.g., "undo last", "clear line")
- [ ] Integration with IDE plugins (VSCode, Neovim)

**Why:** Differentiate from other voice tools.

### 8. Community Infrastructure
**Status:** Partially Done
**Priority:** Medium (for open source)
**Effort:** Small (2-3 hours)

- [x] LICENSE file (MIT)
- [ ] CODE_OF_CONDUCT.md
- [ ] CONTRIBUTING.md with development setup
- [ ] Issue templates (bug report, feature request)
- [ ] PR template with checklist
- [ ] Discord/Matrix community channel

**Why:** Prepare for community contributions.

### 9. Cross-Platform Testing
**Status:** Not Started
**Priority:** Low
**Effort:** Large (platform-dependent)

- [ ] Test macOS Metal backend with new models
- [ ] Verify Windows compatibility (if supporting)
- [ ] Test on various Linux distros (Arch, Ubuntu, Fedora)
- [ ] Validate all GPU backends (CUDA, ROCm, Vulkan)
- [ ] Document platform-specific quirks

**Why:** Ensure reliability across environments.

## Suggested Next Action

**Start with #1 (Fix Speculative Decoding)** - This unlocks the core performance promise and validates the entire revamp effort. Once that's proven, move to #3 (Polybar) for quick community wins.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
