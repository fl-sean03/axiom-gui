# Axiom GUI - Test Suite Summary

## Quick Reference

### Running Tests

```bash
cd /home/agent/projects/axiom/axiom-gui/webdriver

# Run all tests
DISPLAY=:99 npm test

# Run individual suites
DISPLAY=:99 npm run test:window      # 6 tests
DISPLAY=:99 npm run test:layout      # 7 tests
DISPLAY=:99 npm run test:rendering   # 8 tests
DISPLAY=:99 npm run test:camera      # 10 tests
DISPLAY=:99 npm run test:mouse       # 8 tests
```

## Test Suites

### 1. Window Tests (`window.e2e.js`) - 6 tests
- ✅ Launch without errors
- ❓ Window title contains "Axiom"
- ✅ Main container displays
- ❓ Functional layout (#root element)
- ✅ Responsive to resize
- ✅ Minimize/maximize

### 2. Layout Tests (`layout.e2e.js`) - 7 tests
- Sidebar visibility
- Main canvas area
- Status bar at bottom
- No overflow at 1280x720
- Usable at 800x600
- Scales properly at 2560x1440
- Visible text and buttons

### 3. Rendering Tests (`rendering.e2e.js`) - 8 tests
- SSAA control buttons
- Ambient occlusion controls
- Background color selector
- Render button
- Toggle SSAA levels
- Toggle AO on/off
- Switch backgrounds
- Rendering state changes

### 4. Camera Tests (`camera.e2e.js`) - 10 tests
- Zoom buttons present
- Camera preset buttons
- Click Zoom In
- Click Zoom Out
- Click Top preset
- Click Side preset
- Click Front preset
- Click Isometric preset
- Click Reset preset
- Cycle through all presets

### 5. Mouse Tests (`mouse.e2e.js`) - 8 tests
- Find canvas element
- Mouse click
- Mouse drag (orbit)
- Vertical drag
- Right-click drag (pan)
- Mouse wheel scroll (zoom)
- Rapid clicking
- Combined actions

## Current Status

**Infrastructure:** ✅ Complete
**Test Suites:** ✅ Complete (39 tests written)
**Execution:** ❌ Blocked by Tauri config (devUrl vs frontendDist)

**Tests Run:** 4/39 passing (window-level tests)
**Tests Blocked:** 35/39 (require HTML content)

## How to Fix Blocker

```bash
# Install Tauri CLI
cd /home/agent/projects/axiom/axiom-gui
npm install --save-dev @tauri-apps/cli

# Build with proper frontend bundling
npm run tauri build -- --debug

# Run tests
cd webdriver
DISPLAY=:99 npm test
```

## Documentation

- `COMPREHENSIVE_TEST_REPORT.md` - Full report with infrastructure details
- `COMPREHENSIVE_VALIDATION_PLAN.md` - Complete validation matrix (100+ checkpoints)
- `TEST_SUITE_SUMMARY.md` - This file (quick reference)

## Test Infrastructure

- **WebKit WebDriver:** `/usr/bin/WebKitWebDriver`
- **Tauri Driver:** `~/.cargo/bin/tauri-driver`
- **Test Framework:** WebdriverIO 9.19.0 + Mocha
- **Display:** Xvfb :99 (1920x1080x24)
- **Location:** `/home/agent/projects/axiom/axiom-gui/webdriver/`
