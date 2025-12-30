# Phase 4 CI/CD: macOS Support Handoff

**Date:** 2025-12-17 02:02
**Context:** Post Phase 4 implementation, preparing for v0.2.0 release

---

## Goal

Add macOS runner support to CI/CD pipeline and verify cross-platform builds work before merging Phase 4 PR and releasing v0.2.0.

## Current State

**What's Working:**
- ✅ Phase 4 code complete (cross-platform text injection with enigo/arboard)
- ✅ Linux builds working (ubuntu-latest with CUDA support)
- ✅ All tests pass on Linux (32 tests, zero warnings)
- ✅ PR #1 created to main branch
- ✅ CUDA daemon working on RTX 4060 Ti (1.6GB VRAM)
- ✅ Code review completed (10/10 score)

**What's Missing:**
- ❌ CI only runs on Linux (ubuntu-latest)
- ❌ No macOS runners configured
- ❌ Zero macOS testing (audio capture, text injection, permissions)
- ❌ No GPU feature testing (cuda, metal, vulkan)
- ❌ No cross-platform build matrix

**File:** `.github/workflows/ci.yml`
- 5 jobs: check, test, clippy, fmt, coverage
- All hardcoded to `ubuntu-latest`
- No platform-specific logic

## Approach

### Multi-Platform Build Matrix

Add macOS runners alongside Linux:
- Linux: Keep `ubuntu-latest` (primary platform)
- macOS ARM: Add `macos-15`, `macos-14` (Apple Silicon)
- macOS Intel: Add `macos-15-intel` (support until Fall 2027)

### GPU Feature Matrix

Test compilation with different GPU backends:
- `cuda` - Linux/NVIDIA (already working)
- `metal` - macOS ARM only
- `vulkan` - Linux (future)

### Platform-Specific Dependencies

- Linux: Install `libclang-dev` (whisper-rs build dependency)
- macOS: Native tools (clang usually pre-installed)

## Key Decisions

### 1. macOS Runner Versions (CRITICAL)

**Decision:** Use `macos-15` and `macos-14` as primary targets

**Rationale:**
- macOS 26 (Tahoe) is released by Apple BUT GitHub Actions `macos-26` runner is still **beta** (no SLA)
- Building on `macos-15` provides **forward compatibility** → binaries work on macOS 26 users
- `macos-13` deprecated (retired Dec 4, 2025)
- `macos-15` = Sequoia (current GA), `macos-14` = Sonoma (still widely used)

**Runner → User Coverage:**
- `macos-15` builds support: macOS 13-26 users (with deployment target set)
- `macos-26` runner only needed for testing, not building

### 2. Intel vs ARM (IMPORTANT)

**Decision:** Support both, prioritize ARM

**Rationale:**
- Most Mac users on Apple Silicon (M1/M2/M3/M4)
- Intel support ending Fall 2027 (Apple announced)
- Metal GPU acceleration only works on ARM (not Intel)
- Include `macos-15-intel` for x86_64 users during transition period

### 3. Testing Strategy

**Decision:** CI builds first, manual testing second

**Rationale:**
- Verify builds succeed on macOS before manual testing
- Catch compilation issues early (dependency problems, platform-specific code)
- Manual testing expensive (requires Mac access)

## Blockers/Unknowns

### 1. macOS Permissions (HIGH RISK)

**Unknown:** How CPAL audio capture handles macOS microphone permissions
- Does it fail gracefully?
- What error messages appear?
- Does CI build succeed even if runtime permissions fail?

**Unknown:** How enigo handles macOS Accessibility permissions
- Required for keyboard injection?
- Error handling when denied?

**Action Needed:** Test on actual Mac hardware

### 2. Metal GPU Acceleration (MEDIUM RISK)

**Unknown:** Does `--features metal` compile on `macos-15` runner?
- whisper-rs Metal backend tested?
- Performance vs CPU on M1/M2/M3?

**Action Needed:** Add to build matrix, verify compilation

### 3. Cross-Platform Dependencies (LOW RISK)

**Unknown:** Do we need platform-specific dependency installation steps?
- macOS usually has clang pre-installed
- But might need explicit LLVM for whisper-rs?

**Action Needed:** Test CI build, add deps if needed

### 4. Release Artifact Strategy (DEFERRED)

**Question:** Should we build release binaries for all platforms in CI?
- Separate artifacts: `hyprvoice-linux-x64`, `hyprvoice-macos-arm64`, etc.?
- Or just verify builds work, manual release later?

**Decision:** Defer to v0.2.0 release preparation

## Next Steps

### Immediate (Next Session)

1. **Update CI workflow** (`.github/workflows/ci.yml`)
   - Add build matrix with `macos-15`, `macos-14`, `macos-15-intel`
   - Keep existing Linux jobs
   - Add platform-specific dependency installation
   - Test builds succeed (don't worry about functional tests yet)

2. **Verify PR #1 CI passes**
   - Push updated workflow
   - Monitor GitHub Actions for macOS build status
   - Fix any compilation errors

### Short-Term (Before v0.2.0 Release)

3. **Manual macOS testing**
   - Test CPAL audio capture on Mac
   - Test enigo text injection on Mac
   - Document permission setup (microphone, accessibility)
   - Verify daemon mode works

4. **Add GPU feature testing**
   - Test `--features metal` on `macos-15`
   - Test `--features cuda` on `ubuntu-latest`
   - Document GPU requirements

5. **Update documentation**
   - Add macOS installation guide to README
   - Document permission requirements
   - Update PR #1 description with CI changes

### Long-Term (Post v0.2.0)

6. **Release automation**
   - Build artifacts for all platforms
   - Upload to GitHub releases
   - Consider Homebrew formula (macOS distribution)

7. **Windows support** (Phase 5+)
   - Add `windows-latest` runner
   - Test on Windows 11

## Context Links

- **Implementation Summary:** `docs/project/phase4-implementation-summary.md`
- **Code Review:** `.claude/plans/luminous-dreaming-hamster.md`
- **PR:** https://github.com/itsdevcoffee/hyprvoice/pull/1
- **Current CI:** `.github/workflows/ci.yml`
- **Hyprland Config:** `~/.config/hypr/UserConfigs/UserKeybinds.conf` (lines 43-46)

## Quick Wins

If time is limited, focus on:
1. Add `macos-15` to build matrix → Proves macOS compilation works
2. Push and verify CI passes → Unblocks PR merge
3. Manual smoke test on Mac → Audio + text injection basics

Everything else can iterate post-release.

---

**Status:** Ready for macOS CI implementation
**Branch:** `phase4-cpal-migration`
**Next Milestone:** Merge PR #1, release v0.2.0
