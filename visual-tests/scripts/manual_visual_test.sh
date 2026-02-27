#!/bin/bash
# Manual Visual Test Runner for Axiom GUI
# This script starts the GUI and captures screenshots at intervals

set -e

DISPLAY=:99
export DISPLAY

BASE_DIR="/home/agent/projects/axiom/axiom-gui/visual-tests"
SCREENSHOTS_DIR="$BASE_DIR/screenshots"
GUI_DIR="/home/agent/projects/axiom/axiom-gui"

mkdir -p "$SCREENSHOTS_DIR"

echo "=== Axiom GUI Manual Visual Test ==="
echo "Display: $DISPLAY"
echo "Screenshots: $SCREENSHOTS_DIR"
echo ""

# Check if scrot is available, if not use import from ImageMagick
SCREENSHOT_CMD=""
if command -v scrot &> /dev/null; then
    SCREENSHOT_CMD="scrot"
    echo "Using scrot for screenshots"
elif command -v import &> /dev/null; then
    SCREENSHOT_CMD="import"
    echo "Using ImageMagick import for screenshots"
else
    echo "ERROR: Neither scrot nor ImageMagick import found"
    echo "Install with: sudo apt-get install scrot imagemagick"
    exit 1
fi

# Function to take screenshot
take_screenshot() {
    local name="$1"
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local filename="${timestamp}_${name}.png"
    local filepath="$SCREENSHOTS_DIR/$filename"

    if [ "$SCREENSHOT_CMD" = "scrot" ]; then
        scrot "$filepath" 2>/dev/null || echo "  Screenshot failed"
    else
        import -window root "$filepath" 2>/dev/null || echo "  Screenshot failed"
    fi

    if [ -f "$filepath" ]; then
        echo "  Screenshot: $filename"
    fi
}

# Start the GUI in background
echo ""
echo "Starting Axiom GUI..."
cd "$GUI_DIR"

# Kill any existing instances
pkill -f "axiom-gui" || true
pkill -f "tauri dev" || true
sleep 2

# Start in background
npm run tauri dev &> /tmp/axiom-gui-visual-test.log &
GUI_PID=$!

echo "  GUI started (PID: $GUI_PID)"
echo "  Waiting for GUI to initialize (30 seconds)..."
sleep 30

echo ""
echo "=== Taking Screenshots ==="

# Take initial screenshot
take_screenshot "00_initial_state"
sleep 2

# Take screenshots every 5 seconds for a minute
for i in {1..12}; do
    echo "Capture $i/12..."
    take_screenshot "$(printf "%02d_state" $i)"
    sleep 5
done

echo ""
echo "=== Stopping GUI ==="
kill $GUI_PID 2>/dev/null || true
sleep 2

echo ""
echo "=== Test Complete ==="
echo "Screenshots saved to: $SCREENSHOTS_DIR"
ls -lh "$SCREENSHOTS_DIR" | tail -n 15

echo ""
echo "Next steps:"
echo "  1. Review screenshots to verify GUI rendered correctly"
echo "  2. Check for visual artifacts or errors"
echo "  3. Run interactive tests manually"
