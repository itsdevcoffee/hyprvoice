# Desktop Environment Integrations

Status bar and system tray integrations for hyprvoice across different desktop environments.

## Available Integrations

### Waybar (Wayland)
Real-time status module for Waybar with signal-based updates.
- **Path**: `integrations/waybar/`
- **Platforms**: Linux (Wayland compositors: Hyprland, Sway)
- **Status**: ✅ Complete

[Setup Guide →](waybar/README.md)

### Polybar (X11)
Coming soon - Status module for i3/bspwm users.
- **Path**: `integrations/polybar/`
- **Platforms**: Linux (X11 window managers)
- **Status**: 🚧 Planned

### i3status
Coming soon - Minimal integration for i3status users.
- **Path**: `integrations/i3status/`
- **Platforms**: Linux (i3wm)
- **Status**: 🚧 Planned

## Architecture

All integrations follow a common pattern:

1. **State Files** (Generic):
   - `~/.local/state/hyprvoice/recording.pid` - Recording state
   - `~/.local/state/hyprvoice/processing` - Transcription state

2. **Refresh Mechanism** (Configurable):
   - Set via `output.refresh_command` in hyprvoice config
   - Each integration documents its specific command

3. **Module Files** (Integration-specific):
   - Status script (reads state files, outputs formatted data)
   - Config snippet (integration-specific format)
   - Styling example (CSS/theme)
   - Install script (automated setup)

## Contributing

Want to add support for your favorite desktop environment?

1. Create `integrations/<name>/` directory
2. Follow the Waybar structure as a template
3. Document the refresh command for your bar
4. Submit a PR

See `integrations/waybar/` for a complete reference implementation.
