#!/usr/bin/env bash
# hyprvoice-status.sh - Waybar custom module for voice dictation status

STATE_DIR="${XDG_STATE_HOME:-$HOME/.local/state}/hyprvoice"
PID_FILE="${STATE_DIR}/recording.pid"
PROCESSING_FILE="${STATE_DIR}/processing"

# Icons (Nerd Fonts required)
ICON_IDLE="󰔊"
ICON_RECORDING="󰑋"
ICON_PROCESSING="󱐋"

# Check if recording
if [[ -f "$PID_FILE" ]]; then
    PID=$(head -n 1 "$PID_FILE" 2>/dev/null || echo "")

    if [[ "$PID" =~ ^[0-9]+$ ]] && kill -0 "$PID" 2>/dev/null; then
        STARTED_AT=$(sed -n '2p' "$PID_FILE" 2>/dev/null || echo "0")
        NOW=$(date +%s)
        ELAPSED=$((NOW - STARTED_AT))
        MINS=$((ELAPSED / 60))
        SECS=$((ELAPSED % 60))
        TIME_STR=$(printf "%d:%02d" "$MINS" "$SECS")
        echo "{\"text\": \"${ICON_RECORDING} ${TIME_STR}\", \"tooltip\": \"Recording... Click to stop\", \"class\": \"recording\"}"
        exit 0
    else
        # Stale PID file
        rm -f "$PID_FILE"
    fi
fi

# Check if processing (transcribing)
if [[ -f "$PROCESSING_FILE" ]]; then
    echo "{\"text\": \"${ICON_PROCESSING}\", \"tooltip\": \"Thinking...\", \"class\": \"processing\"}"
    exit 0
fi

# Idle state
echo "{\"text\": \"${ICON_IDLE}\", \"tooltip\": \"Click to start recording\", \"class\": \"idle\"}"
