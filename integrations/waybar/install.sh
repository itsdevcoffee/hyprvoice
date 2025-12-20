#!/bin/bash
# Waybar Integration Installer for dev-voice

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Paths
WAYBAR_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/waybar"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo -e "${GREEN}=== dev-voice Waybar Integration Installer ===${NC}"
echo ""

# Check if Waybar config exists
if [ ! -d "$WAYBAR_DIR" ]; then
    echo -e "${RED}Error: Waybar config directory not found at $WAYBAR_DIR${NC}"
    echo "Please install and configure Waybar first."
    exit 1
fi

# Create scripts directory if needed
mkdir -p "$WAYBAR_DIR/scripts"

# Copy script to standard location
echo -e "${YELLOW}Installing status script...${NC}"
cp "$SCRIPT_DIR/dev-voice-status.sh" "$WAYBAR_DIR/scripts/"
chmod +x "$WAYBAR_DIR/scripts/dev-voice-status.sh"

echo -e "${GREEN}✓ Script installed to $WAYBAR_DIR/scripts/dev-voice-status.sh${NC}"
echo ""

# Show config snippet
echo -e "${GREEN}=== Installation Complete ===${NC}"
echo ""
echo -e "${YELLOW}Next Steps:${NC}"
echo ""
echo "1. Add this module to your Waybar config ($WAYBAR_DIR/config.jsonc or modules file):"
echo ""
cat "$SCRIPT_DIR/config-snippet.jsonc"
echo ""
echo "2. Add 'custom/dev-voice' to one of your module lists:"
echo -e "   ${GREEN}\"modules-left\": [..., \"custom/dev-voice\"]${NC}"
echo ""
echo "3. (Optional) Add these styles to your style.css:"
echo ""
cat "$SCRIPT_DIR/style-snippet.css"
echo ""
echo "4. Reload Waybar:"
echo -e "   ${GREEN}pkill -SIGUSR2 waybar${NC}"
echo ""
echo "5. Configure dev-voice to refresh Waybar:"
echo "   Edit ~/.config/dev-voice/config.toml and add to [output] section:"
echo -e "   ${GREEN}refresh_command = \"pkill -RTMIN+8 waybar\"${NC}"
echo ""
