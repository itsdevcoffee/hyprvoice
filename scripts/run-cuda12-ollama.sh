#!/bin/bash
# Wrapper to run dev-voice CUDA binary with Ollama's CUDA 12 libraries
#
# Usage:
#   ./scripts/run-cuda12-ollama.sh daemon
#   ./scripts/run-cuda12-ollama.sh start --duration 10
#
# Environment variables:
#   DEVVOICE_BINARY - Override binary location
#   DEVVOICE_DEBUG  - Print library loading info
#   DEVVOICE_STRICT - Use ONLY Ollama libs (default: 1, set to 0 to append system paths)
#   DEVVOICE_LIBDIR - Override CUDA library directory (for bundled libs)

# Set library path (strict by default to prevent Python CUDA pollution)
if [ -n "$DEVVOICE_LIBDIR" ]; then
    # Custom library directory (for future bundled libs)
    export LD_LIBRARY_PATH="$DEVVOICE_LIBDIR"
elif [ "${DEVVOICE_STRICT:-1}" = "0" ]; then
    # Non-strict mode: Append existing paths (opt-in only)
    export LD_LIBRARY_PATH=/usr/local/lib/ollama${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
else
    # Strict mode (DEFAULT): Only Ollama libs
    export LD_LIBRARY_PATH=/usr/local/lib/ollama
fi

# Find dev-voice binary
BINARY="${DEVVOICE_BINARY:-./dev-voice}"

if [ ! -x "$BINARY" ]; then
    # Try common locations (prefer artifacts/ over docs/tmp/)
    if [ -x "$HOME/.local/bin/dev-voice-cuda" ]; then
        BINARY="$HOME/.local/bin/dev-voice-cuda"
    elif [ -x "./target/release/dev-voice" ]; then
        BINARY="./target/release/dev-voice"
    elif [ -x "./artifacts/dev-voice-linux-x64-cuda/dev-voice" ]; then
        BINARY="./artifacts/dev-voice-linux-x64-cuda/dev-voice"
    elif [ -x "./docs/tmp/dev-voice-linux-x64-cuda/dev-voice" ]; then
        BINARY="./docs/tmp/dev-voice-linux-x64-cuda/dev-voice"
    else
        echo "Error: Cannot find dev-voice binary" >&2
        echo "Searched:" >&2
        echo "  - ~/.local/bin/dev-voice-cuda" >&2
        echo "  - ./target/release/dev-voice" >&2
        echo "  - ./artifacts/dev-voice-linux-x64-cuda/dev-voice" >&2
        echo "  - ./docs/tmp/dev-voice-linux-x64-cuda/dev-voice" >&2
        echo "" >&2
        echo "Set DEVVOICE_BINARY environment variable or install binary first" >&2
        exit 1
    fi
fi

# Debug mode: Show what's being loaded
if [ "$DEVVOICE_DEBUG" = "1" ]; then
    echo "=== dev-voice CUDA Debug Info ===" >&2
    echo "Binary: $BINARY" >&2
    echo "LD_LIBRARY_PATH: $LD_LIBRARY_PATH" >&2
    echo "" >&2
    echo "Libraries that will be loaded:" >&2
    ldd "$BINARY" | grep -E 'cudart|cublas|cudnn|cuda' || true
    echo "" >&2

    # Warn about Python CUDA pollution
    if ldd "$BINARY" | grep -q "site-packages/nvidia"; then
        echo "⚠️  WARNING: Loading CUDA libraries from Python site-packages!" >&2
        echo "This can cause version mismatches and subtle bugs." >&2
        echo "Consider running with DEVVOICE_STRICT=1 to use only Ollama libs." >&2
        echo "" >&2
    fi

    echo "Starting dev-voice..." >&2
    echo "==============================" >&2
    echo "" >&2
fi

# Run dev-voice with all arguments passed through
exec "$BINARY" "$@"
