#!/bin/bash
# Install hyprvoice from GitHub releases or artifacts
#
# Usage:
#   ./scripts/install.sh                    # Auto-detect platform, install CPU version
#   ./scripts/install.sh --gpu              # Install GPU version (CUDA/Metal)
#   ./scripts/install.sh --run-id 12345     # Install from specific GitHub Actions run

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
REPO="itsdevcoffee/hyprvoice"
INSTALL_DIR="$HOME/.local/bin"
ARTIFACT_DIR="./artifacts"

# Parse arguments
GPU_VERSION=false
RUN_ID=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --gpu)
            GPU_VERSION=true
            shift
            ;;
        --run-id)
            RUN_ID="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--gpu] [--run-id RUN_ID]"
            exit 1
            ;;
    esac
done

# Detect platform
detect_platform() {
    local os=$(uname -s)
    local arch=$(uname -m)

    case "$os" in
        Linux)
            if [ "$arch" = "x86_64" ]; then
                if [ "$GPU_VERSION" = true ]; then
                    if command -v nvidia-smi &> /dev/null; then
                        echo "hyprvoice-linux-x64-cuda"
                    else
                        echo -e "${YELLOW}Warning: --gpu specified but no NVIDIA GPU detected${NC}" >&2
                        echo -e "${YELLOW}Installing CPU version instead${NC}" >&2
                        echo "hyprvoice-linux-x64"
                    fi
                else
                    echo "hyprvoice-linux-x64"
                fi
            else
                echo -e "${RED}Error: Linux ARM64 not yet supported${NC}" >&2
                exit 1
            fi
            ;;
        Darwin)
            if [ "$(sysctl -n machdep.cpu.brand_string | grep -c Apple)" -gt 0 ]; then
                # Apple Silicon
                if [ "$GPU_VERSION" = true ]; then
                    # Check macOS version
                    local macos_version=$(sw_vers -productVersion | cut -d. -f1)
                    if [ "$macos_version" -ge 15 ]; then
                        echo "hyprvoice-macos-15-arm64-metal"
                    else
                        echo "hyprvoice-macos-14-arm64-metal"
                    fi
                else
                    echo "hyprvoice-macos-arm64"
                fi
            else
                # Intel Mac
                if [ "$GPU_VERSION" = true ]; then
                    echo -e "${YELLOW}Warning: GPU not available on Intel Macs${NC}" >&2
                    echo -e "${YELLOW}Installing CPU version instead${NC}" >&2
                fi
                echo "hyprvoice-macos-intel"
            fi
            ;;
        *)
            echo -e "${RED}Error: Unsupported OS: $os${NC}" >&2
            exit 1
            ;;
    esac
}

# Download artifact
download_artifact() {
    local artifact_name=$1

    echo -e "${GREEN}Downloading $artifact_name...${NC}"

    # Create artifact directory
    mkdir -p "$ARTIFACT_DIR"

    if [ -n "$RUN_ID" ]; then
        # Download from specific run
        gh run download "$RUN_ID" -n "$artifact_name" -D "$ARTIFACT_DIR"
    else
        # Download from latest successful run
        local latest_run=$(gh run list --workflow=ci.yml --status=success --limit 1 --json databaseId --jq '.[0].databaseId')
        if [ -z "$latest_run" ]; then
            echo -e "${RED}Error: No successful CI runs found${NC}" >&2
            exit 1
        fi
        echo -e "${YELLOW}Using latest successful run: $latest_run${NC}"
        gh run download "$latest_run" -n "$artifact_name" -D "$ARTIFACT_DIR"
    fi

    echo -e "${GREEN}✓ Downloaded to $ARTIFACT_DIR/$artifact_name/${NC}"
}

# Install binary
install_binary() {
    local artifact_name=$1
    local source="$ARTIFACT_DIR/$artifact_name/hyprvoice"

    if [ ! -f "$source" ]; then
        echo -e "${RED}Error: Binary not found at $source${NC}" >&2
        exit 1
    fi

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Determine install name
    local install_name
    if [[ "$artifact_name" =~ cuda ]]; then
        install_name="hyprvoice-cuda"
    elif [[ "$artifact_name" =~ metal ]]; then
        install_name="hyprvoice"  # Metal is default on macOS
    else
        install_name="hyprvoice"
    fi

    # Install binary
    install -m 755 "$source" "$INSTALL_DIR/$install_name"
    echo -e "${GREEN}✓ Installed to $INSTALL_DIR/$install_name${NC}"

    # Install wrapper for CUDA version
    if [[ "$artifact_name" =~ cuda ]]; then
        if [ -f "./scripts/run-cuda12-ollama.sh" ]; then
            install -m 755 ./scripts/run-cuda12-ollama.sh "$INSTALL_DIR/hyprvoice-gpu"
            echo -e "${GREEN}✓ Installed wrapper to $INSTALL_DIR/hyprvoice-gpu${NC}"
        fi
    fi
}

# Main
main() {
    echo "=== hyprvoice Installation ==="
    echo ""

    # Check for gh CLI
    if ! command -v gh &> /dev/null; then
        echo -e "${RED}Error: GitHub CLI (gh) not found${NC}" >&2
        echo "Install from: https://cli.github.com/" >&2
        exit 1
    fi

    # Detect platform
    ARTIFACT_NAME=$(detect_platform)
    echo -e "Detected platform: ${GREEN}$ARTIFACT_NAME${NC}"
    echo ""

    # Download
    download_artifact "$ARTIFACT_NAME"
    echo ""

    # Install
    install_binary "$ARTIFACT_NAME"
    echo ""

    # Next steps
    echo -e "${GREEN}✓ Installation complete!${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Download a model:"
    if [[ "$ARTIFACT_NAME" =~ cuda ]]; then
        echo "     hyprvoice-cuda download base.en"
        echo ""
        echo "  2. Start daemon (with CUDA wrapper):"
        echo "     hyprvoice-gpu daemon"
        echo ""
        echo "  Or test library loading first:"
        echo "     DEVVOICE_DEBUG=1 hyprvoice-gpu --version"
    else
        echo "     hyprvoice download base.en"
        echo ""
        echo "  2. Start daemon:"
        echo "     hyprvoice daemon"
    fi
    echo ""
    echo "  3. In another terminal:"
    echo "     hyprvoice start"
    echo "     hyprvoice stop"
}

main "$@"
