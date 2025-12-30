#!/bin/bash
# Waybar Integration Installer for hyprvoice

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Paths
WAYBAR_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/waybar"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo -e "${GREEN}=== hyprvoice Waybar Integration Installer ===${NC}"
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
cp "$SCRIPT_DIR/hyprvoice-status.sh" "$WAYBAR_DIR/scripts/"
chmod +x "$WAYBAR_DIR/scripts/hyprvoice-status.sh"

echo -e "${GREEN}✓ Script installed to $WAYBAR_DIR/scripts/hyprvoice-status.sh${NC}"
echo ""

# Configure hyprvoice to refresh Waybar
echo -e "${YELLOW}Configuring hyprvoice refresh command...${NC}"
DEVVOICE_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}/hyprvoice/config.toml"

if [ -f "$DEVVOICE_CONFIG" ]; then
    # Check if refresh_command already exists
    if grep -q "refresh_command" "$DEVVOICE_CONFIG"; then
        echo -e "${YELLOW}⚠ refresh_command already configured, skipping${NC}"
    else
        # Check if [output] section exists
        if grep -q "^\[output\]" "$DEVVOICE_CONFIG"; then
            # Add refresh_command after [output] line
            sed -i '/^\[output\]/a refresh_command = "pkill -RTMIN+8 waybar"' "$DEVVOICE_CONFIG"
            echo -e "${GREEN}✓ Added refresh_command to hyprvoice config${NC}"
        else
            # Append [output] section with refresh_command
            echo "" >> "$DEVVOICE_CONFIG"
            echo "[output]" >> "$DEVVOICE_CONFIG"
            echo 'refresh_command = "pkill -RTMIN+8 waybar"' >> "$DEVVOICE_CONFIG"
            echo -e "${GREEN}✓ Created [output] section with refresh_command${NC}"
        fi
    fi
else
    echo -e "${YELLOW}⚠ hyprvoice config not found, skipping auto-configuration${NC}"
    echo "  Run 'hyprvoice config --reset' to create it"
fi
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
echo "2. Add 'custom/hyprvoice' to one of your module lists:"
echo -e "   ${GREEN}\"modules-left\": [..., \"custom/hyprvoice\"]${NC}"
echo ""
echo "3. (Optional) Add these styles to your style.css:"
echo ""
cat "$SCRIPT_DIR/style-snippet.css"
echo ""
echo "4. Reload Waybar:"
echo -e "   ${GREEN}pkill -SIGUSR2 waybar${NC}"
echo ""
echo "Done! Your Waybar module is ready to use."
echo ""
