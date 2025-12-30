#!/bin/bash
# Quick dev install script - Build and update local binaries
#
# Usage:
#   ./scripts/dev-update.sh           # CPU-only build
#   ./scripts/dev-update.sh --cuda    # CUDA build
#   ./scripts/dev-update.sh --metal   # Metal build (macOS)
#   ./scripts/dev-update.sh --restart # Also restart daemon

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Parse args
FEATURES=""
RESTART=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --cuda)
            FEATURES="--features cuda"
            shift
            ;;
        --metal)
            FEATURES="--features metal"
            shift
            ;;
        --restart)
            RESTART=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--cuda|--metal] [--restart]"
            exit 1
            ;;
    esac
done

# Build
echo -e "${YELLOW}Building release binary${FEATURES:+ with $FEATURES}...${NC}"

# Set RUSTFLAGS for CUDA linking (points to lib64 + Ollama CUDA libs)
if [[ "$FEATURES" == *"cuda"* ]]; then
    export RUSTFLAGS="-L /usr/lib64 -L /usr/local/lib/ollama"
    echo -e "${YELLOW}Using RUSTFLAGS for CUDA: $RUSTFLAGS${NC}"
fi

cargo build --release $FEATURES

# Install
echo -e "${YELLOW}Installing binaries...${NC}"
cp target/release/hyprvoice ~/.local/bin/hyprvoice
cp target/release/hyprvoice ~/.local/bin/hyprvoice-cuda

echo -e "${GREEN}✓ Installed to ~/.local/bin/{hyprvoice,hyprvoice-cuda}${NC}"

# Restart daemon if requested
if [ "$RESTART" = true ]; then
    echo -e "${YELLOW}Restarting daemon...${NC}"
    pkill -9 hyprvoice || true
    sleep 0.5
    hyprvoice-gpu daemon &
    echo -e "${GREEN}✓ Daemon restarted${NC}"
fi

echo -e "\n${GREEN}Update complete!${NC}"
