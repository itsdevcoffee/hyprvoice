#!/bin/bash
# Diagnostic script for dev-voice troubleshooting and bug reports
#
# Usage: ./scripts/doctor.sh

echo "=== dev-voice Doctor Report ==="
echo ""

echo "## System Information"
uname -a
echo "Date: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo ""

echo "## Binary Resolution"
echo "Checking which binaries are in PATH and their details..."
command -v dev-voice 2>/dev/null || echo "dev-voice: not in PATH"
command -v dev-voice-cuda12 2>/dev/null || echo "dev-voice-cuda12: not in PATH"
command -v dev-voice-gpu 2>/dev/null || echo "dev-voice-gpu: not in PATH"
echo ""

echo "Binary files:"
ls -lh "$(command -v dev-voice 2>/dev/null)" 2>/dev/null || echo "dev-voice: not found"
ls -lh "$(command -v dev-voice-cuda12 2>/dev/null)" 2>/dev/null || echo "dev-voice-cuda12: not found"
ls -lh "$(command -v dev-voice-gpu 2>/dev/null)" 2>/dev/null || echo "dev-voice-gpu: not found (wrapper script)"
echo ""

echo "Binary checksums (CPU and CUDA should be DIFFERENT):"
sha256sum "$(command -v dev-voice 2>/dev/null)" "$(command -v dev-voice-cuda12 2>/dev/null)" 2>/dev/null || echo "One or both binaries not found"
echo ""

echo "## NVIDIA GPU"
if command -v nvidia-smi &> /dev/null; then
    nvidia-smi --query-gpu=name,driver_version --format=csv,noheader 2>/dev/null || nvidia-smi 2>/dev/null | head -5
else
    echo "nvidia-smi: not found (no NVIDIA GPU or drivers not installed)"
fi
echo ""

echo "## CUDA Libraries"
echo "LD_LIBRARY_PATH: ${LD_LIBRARY_PATH:-<not set>}"
echo ""

# Check Ollama CUDA libs
echo "Ollama CUDA 12 libraries:"
ls -lh /usr/local/lib/ollama/libcudart.so* /usr/local/lib/ollama/libcublas.so* 2>/dev/null || echo "  Not found at /usr/local/lib/ollama/"
echo ""

# Check system CUDA
echo "System CUDA libraries:"
ls -lh /usr/local/cuda*/lib64/libcudart.so* 2>/dev/null || echo "  Not found at /usr/local/cuda*/lib64/"
echo ""

echo "## CUDA Binary Dependencies"

# Check CPU binary (clear LD_LIBRARY_PATH to avoid confusion)
if command -v dev-voice &> /dev/null; then
    BIN_CPU="$(command -v dev-voice)"
    echo "dev-voice (CPU) link-time dependencies (LD_LIBRARY_PATH cleared):"
    ( unset LD_LIBRARY_PATH; ldd "$BIN_CPU" | grep -E 'cudart|cublas|cudnn|cuda' ) && echo "  ❌ ERROR: CPU binary has CUDA dependencies!" || echo "  ✅ OK: No CUDA dependencies (expected for CPU version)"
    echo ""

    echo "dev-voice (CPU) NEEDED libraries check:"
    readelf -d "$BIN_CPU" | grep -E 'NEEDED.*cudart|NEEDED.*cublas|NEEDED.*cudnn' && echo "  ❌ ERROR: CPU binary linked against CUDA!" || echo "  ✅ OK: No CUDA linkage"
    echo ""
fi

# Check CUDA binary
if command -v dev-voice-cuda12 &> /dev/null; then
    BIN_GPU="$(command -v dev-voice-cuda12)"

    echo "dev-voice-cuda12 (GPU) ambient environment (may be polluted):"
    echo "Current LD_LIBRARY_PATH shows Python CUDA paths - this is the problem we're fixing."
    ldd "$BIN_GPU" | grep -E 'cudart|cublas|cudnn|cuda' || echo "  ❌ ERROR: No CUDA libs found"

    # Warn about Python pollution
    if ldd "$BIN_GPU" | grep -q "site-packages/nvidia"; then
        echo ""
        echo "⚠️  Confirmed: CUDA libraries would load from Python site-packages in ambient env!"
    fi
    echo ""

    echo "dev-voice-cuda12 (GPU) NEEDED libraries:"
    readelf -d "$BIN_GPU" | grep NEEDED | grep -E 'cudart|cublas|cudnn' || echo "  (No explicit CUDA NEEDED entries - dependencies may be indirect)"
    echo ""

    echo "dev-voice-cuda12 (GPU) with strict CUDA 12 env (Ollama-only):"
    echo "This is what the binary sees when run with clean LD_LIBRARY_PATH:"
    LD_LIBRARY_PATH=/usr/local/lib/ollama ldd "$BIN_GPU" | grep -E 'cudart|cublas|cublasLt|cudnn|cuda' || echo "  ❌ ERROR: CUDA libs not found even with Ollama path"
    echo ""

    # Show wrapper runtime resolution (the real truth)
    if command -v dev-voice-gpu &> /dev/null; then
        echo "dev-voice-cuda12 (GPU) via dev-voice-gpu wrapper (strict mode - REAL RUNTIME):"
        echo "This is what actually loads when users run 'dev-voice-gpu':"
        DEVVOICE_DEBUG=1 dev-voice-gpu --version 2>&1 | grep -A 10 "Libraries that will be loaded:" | grep -E 'libcuda|cudart|cublas|cublasLt|cudnn' || echo "  (Wrapper ran successfully; enable DEVVOICE_DEBUG=1 to see full runtime resolution)"
        echo ""
    fi
fi

# Check downloaded artifact if not installed
if [ ! command -v dev-voice-cuda12 &> /dev/null ] && [ -x "./docs/tmp/dev-voice-linux-x64-cuda/dev-voice" ]; then
    BIN_ARTIFACT="./docs/tmp/dev-voice-linux-x64-cuda/dev-voice"
    echo "Downloaded CUDA artifact (not installed):"
    echo "Link-time dependencies:"
    ( unset LD_LIBRARY_PATH; ldd "$BIN_ARTIFACT" | grep -E 'cudart|cublas|cudnn|cuda' ) || echo "  No CUDA libs (ERROR - CUDA artifact should have CUDA deps)"
    echo ""
fi

echo "## Audio System"
if command -v arecord &> /dev/null; then
    echo "ALSA devices:"
    arecord -l 2>/dev/null | grep -E "card|device" | head -5 || echo "  No devices found"
else
    echo "arecord: not found"
fi
echo ""

echo "## Display Server"
echo "XDG_SESSION_TYPE: ${XDG_SESSION_TYPE:-<not set>}"
echo "WAYLAND_DISPLAY: ${WAYLAND_DISPLAY:-<not set>}"
echo "DISPLAY: ${DISPLAY:-<not set>}"
echo ""

echo "=== End of Report ==="
echo ""
echo "Copy this output when reporting bugs or asking for help!"
