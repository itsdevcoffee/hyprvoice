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

echo "## Installed Binaries"
for binary in dev-voice dev-voice-cuda12 dev-voice-gpu; do
    if command -v "$binary" &> /dev/null; then
        echo "✓ $binary: $(command -v $binary)"
        $binary --version 2>/dev/null || echo "  (version check failed)"
    else
        echo "✗ $binary: not found in PATH"
    fi
done
echo ""

echo "## NVIDIA GPU"
if command -v nvidia-smi &> /dev/null; then
    nvidia-smi --query-gpu=name,driver_version,cuda_version --format=csv,noheader 2>/dev/null || nvidia-smi 2>/dev/null | head -5
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

# Check CPU binary
if command -v dev-voice &> /dev/null; then
    BIN_CPU="$(command -v dev-voice)"
    echo "dev-voice (CPU) link-time dependencies:"
    ldd "$BIN_CPU" | grep -E 'cudart|cublas|cudnn|cuda' || echo "  No CUDA dependencies (expected for CPU version)"
    echo ""
fi

# Check CUDA binary
if command -v dev-voice-cuda12 &> /dev/null; then
    BIN_GPU="$(command -v dev-voice-cuda12)"

    echo "dev-voice-cuda12 (GPU) link-time dependencies (ldd):"
    ldd "$BIN_GPU" | grep -E 'cudart|cublas|cudnn|cuda' || echo "  No CUDA libs found (ERROR)"
    echo ""

    echo "dev-voice-cuda12 (GPU) runtime loader resolution (LD_DEBUG=libs):"
    echo "(Showing actual runtime loading with strict mode - filtered for CUDA libs)"
    DEVVOICE_STRICT=1 LD_DEBUG=libs "$BIN_GPU" --version 2>&1 | grep -E '(cudart|cublas|cudnn|libcuda\.so).*calling init|trying file.*(cudart|cublas|cudnn|libcuda\.so)' | sed 's/^[[:space:]]*/  /' | head -20 || echo "  Failed to get runtime info"
    echo ""

    # Warn about Python pollution in ldd output
    if ldd "$BIN_GPU" | grep -q "site-packages/nvidia"; then
        echo "⚠️  WARNING: ldd shows Python site-packages paths!"
        echo "   This means your system LD_LIBRARY_PATH may be polluted."
        echo "   The dev-voice-gpu wrapper (strict mode) prevents this at runtime."
        echo ""
    fi
elif [ -x "./docs/tmp/dev-voice-linux-x64-cuda/dev-voice" ]; then
    # Check downloaded artifact if not installed
    BIN_ARTIFACT="./docs/tmp/dev-voice-linux-x64-cuda/dev-voice"
    echo "Downloaded artifact (not installed) link-time dependencies:"
    ldd "$BIN_ARTIFACT" | grep -E 'cudart|cublas|cudnn|cuda' || echo "  No CUDA libs found"
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
