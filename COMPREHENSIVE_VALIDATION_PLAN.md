# Axiom Phase 4 GUI - Comprehensive Validation Plan

## Overview
This document defines ALL validation gates for the Axiom GUI. Each gate must be tested, validated, and pass before the GUI is considered complete.

## Validation Categories

### 1. Window & Desktop App Fundamentals
- [ ] **Window launch**: App launches without errors
- [ ] **Window resize**: Resize window (drag corners/edges), UI adapts responsively
- [ ] **Window minimize**: Minimize and restore window
- [ ] **Window maximize**: Maximize and restore window
- [ ] **Window close**: Close button works, app exits cleanly
- [ ] **Window title**: Shows "Axiom" or proper title
- [ ] **Native menu bar**: File/View/About menus present and functional
- [ ] **App icon**: Proper icon in title bar and taskbar

### 2. Layout & Responsive Design
- [ ] **Sidebar visibility**: Sidebar is visible on launch
- [ ] **Sidebar collapse**: Collapse button works, sidebar hides
- [ ] **Sidebar expand**: Expand button works, sidebar shows
- [ ] **Sidebar width**: Sidebar has appropriate width (not too wide/narrow)
- [ ] **Main canvas area**: Canvas fills remaining space when sidebar is collapsed/expanded
- [ ] **Status bar**: Status bar is visible at bottom
- [ ] **No layout overflow**: No scrollbars or overflow at default window size
- [ ] **Small window**: UI remains usable at minimum window size (800x600)
- [ ] **Large window**: UI scales properly on large displays (2560x1440+)

### 3. File Loading
- [ ] **File → Open menu**: Opens file dialog
- [ ] **File dialog**: Shows proper file filters (PDB, XYZ, GRO, LAMMPS)
- [ ] **Load PDB**: Successfully loads a PDB file (e.g., 1crn.pdb)
- [ ] **Load XYZ**: Successfully loads an XYZ file (e.g., water.xyz)
- [ ] **Load GRO**: Successfully loads a GRO file (if available)
- [ ] **Load LAMMPS**: Successfully loads a LAMMPS dump file (if available)
- [ ] **Invalid file**: Shows error message on invalid file
- [ ] **Large file**: Handles large structures (>1000 atoms) without crashing
- [ ] **File info display**: Status bar shows correct filename and atom count
- [ ] **No file loaded state**: Shows "No structure loaded" message initially

### 4. Molecule Rendering
- [ ] **Initial render**: Structure renders on canvas after loading
- [ ] **Render quality**: Atoms are spheres with proper shading (not pixelated)
- [ ] **Element colors**: Atoms colored by element (O=red, C=gray, H=white, etc.)
- [ ] **Atom sizes**: Atoms appear reasonable size (not too large/small)
- [ ] **Default zoom**: Default camera distance shows whole molecule with context
- [ ] **Canvas fills space**: Rendered image fills entire canvas component
- [ ] **No distortion**: Aspect ratio is correct (circles are circles, not ovals)
- [ ] **Background color**: Background applies to entire canvas (not just behind atoms)

### 5. Rendering Controls
- [ ] **SSAA selector**: Four buttons visible (Off, 1x, 2x, 4x)
- [ ] **SSAA Off**: Renders without antialiasing (faster, more aliased)
- [ ] **SSAA 1x**: Renders with 1x supersampling
- [ ] **SSAA 2x**: Renders with 2x supersampling (smoother edges)
- [ ] **SSAA 4x**: Renders with 4x supersampling (highest quality)
- [ ] **SSAA visual difference**: Higher SSAA values produce visibly smoother edges
- [ ] **AO toggle**: Ambient occlusion checkbox works
- [ ] **AO enabled**: Enabling AO adds depth/shadows to rendering
- [ ] **AO disabled**: Disabling AO removes shadows (flat lighting)
- [ ] **AO samples selector**: Four buttons visible (4, 8, 16, 32)
- [ ] **AO samples change**: Changing samples affects AO quality
- [ ] **Background Black**: Black background works
- [ ] **Background White**: White background works
- [ ] **Background Transparent**: Transparent background works (checkerboard or alpha)
- [ ] **Render button**: "Render" button is visible and clickable
- [ ] **Render progress**: Shows "Rendering..." state while rendering
- [ ] **Render completion**: Image updates after render completes

### 6. Camera Controls
- [ ] **Zoom In button**: Clicking "Zoom In" makes molecule larger
- [ ] **Zoom Out button**: Clicking "Zoom Out" makes molecule smaller
- [ ] **Zoom Max Out button**: Clicking "Max Out" zooms out significantly
- [ ] **Camera presets section**: Presets buttons visible (Top, Side, Front, Iso, Reset)
- [ ] **Top preset**: Camera moves to top view (looking down Z-axis)
- [ ] **Side preset**: Camera moves to side view
- [ ] **Front preset**: Camera moves to front view
- [ ] **Isometric preset**: Camera moves to isometric view (45° angles)
- [ ] **Reset preset**: Camera returns to default auto-framed position
- [ ] **Visual difference**: Each preset shows molecule from different angle

### 7. Mouse Interaction (CRITICAL - needs manual or Playwright testing)
- [ ] **Mouse orbit**: Click and drag to rotate molecule
- [ ] **Orbit smoothness**: Rotation is smooth (not jumpy)
- [ ] **Orbit axis**: Rotation feels natural (orbits around molecule center)
- [ ] **Mouse pan**: Right-click (or modifier+drag) to pan camera
- [ ] **Pan smoothness**: Panning is smooth
- [ ] **Mouse zoom**: Scroll wheel zooms in/out
- [ ] **Scroll zoom smoothness**: Zoom is smooth and responsive
- [ ] **Scroll zoom direction**: Scroll up = zoom in, scroll down = zoom out
- [ ] **Zoom limits**: Zoom stops at reasonable min/max distances (doesn't clip or go infinite)
- [ ] **Combined interactions**: Can orbit, pan, and zoom in same session without conflicts

### 8. Selection Interface
- [ ] **Selection panel visible**: Selection section in sidebar
- [ ] **Selection input field**: Text input for selection queries
- [ ] **Example queries visible**: Shows example queries ("element O", etc.)
- [ ] **Select button**: "Select" button is clickable
- [ ] **Element selection**: "element O" selects all oxygen atoms
- [ ] **Range selection**: "within 5 of element O" selects atoms near oxygen
- [ ] **Selection visual**: Selected atoms highlighted or shown differently
- [ ] **Clear selection**: Can clear selection
- [ ] **Invalid query**: Shows error message on invalid selection syntax
- [ ] **Selection count**: Shows how many atoms are selected

### 9. Image Export
- [ ] **File → Export Image menu**: Menu item present
- [ ] **Export dialog**: Opens save dialog
- [ ] **Save PNG**: Successfully saves PNG file to disk
- [ ] **Exported image quality**: Exported PNG matches on-screen rendering
- [ ] **Export resolution**: Exported image has correct resolution (matches render settings)
- [ ] **Export transparency**: Transparent background exports properly (if selected)
- [ ] **Export filename**: Default filename is reasonable (e.g., "axiom_export.png")

### 10. Element Statistics
- [ ] **Element stats visible**: Element counts/distribution shown in sidebar
- [ ] **Correct counts**: Element counts match loaded structure
- [ ] **Updates on load**: Stats update when new file is loaded

### 11. Performance & Stability
- [ ] **No crashes**: App doesn't crash during normal usage
- [ ] **No memory leaks**: Memory usage stable over multiple renders
- [ ] **Responsive UI**: UI remains responsive during rendering (doesn't freeze)
- [ ] **Render timeout**: Large structures render in reasonable time (<60s)
- [ ] **Multiple files**: Can load multiple different files in sequence
- [ ] **Rapid clicking**: Doesn't break when clicking buttons rapidly
- [ ] **CPU usage**: CPU usage returns to idle after render completes

### 12. Error Handling
- [ ] **Missing file**: Shows error when file doesn't exist
- [ ] **Corrupted file**: Shows error on malformed structure file
- [ ] **Invalid format**: Shows error on unsupported file format
- [ ] **Render failure**: Shows error if rendering fails
- [ ] **Error messages**: Error messages are clear and helpful

### 13. Accessibility & UX
- [ ] **Keyboard navigation**: Can navigate UI with Tab key
- [ ] **Button hover states**: Buttons show hover state
- [ ] **Button click feedback**: Buttons show click feedback
- [ ] **Tooltips**: Tooltips present on buttons (if applicable)
- [ ] **Loading states**: UI shows loading/processing states clearly
- [ ] **Disabled states**: Disabled controls appear disabled (grayed out)

### 14. Cross-Platform (if testing on multiple platforms)
- [ ] **Linux**: All features work on Linux
- [ ] **macOS**: All features work on macOS
- [ ] **Windows**: All features work on Windows

## Test Strategy

### Phase 1: Automated Tests (Playwright)
- Set up Playwright for Tauri app testing
- Automate as many validation gates as possible
- Focus on: window behavior, UI layout, button clicks, file loading, render triggering

### Phase 2: Visual Validation
- Screenshot-based validation for rendering quality
- Compare rendering output across different settings
- Validate camera presets produce different views

### Phase 3: Manual Testing (if Playwright can't test)
- Mouse interaction (orbit, pan, zoom)
- Performance testing with large structures
- Cross-platform validation

### Phase 4: Edge Cases & Stress Testing
- Very large files (>10,000 atoms)
- Very small window sizes
- Rapid interactions
- Invalid inputs

## Success Criteria
- **ALL validation gates pass**
- **Zero crashes or hangs**
- **Visual quality meets expectations**
- **Performance is acceptable** (<5s for small structures, <60s for large)
- **User interactions feel natural and responsive**

## Tools
- **Playwright**: Automated UI testing
- **Xvfb + x11vnc**: Headless GUI testing on server
- **Screenshot comparison**: Validate visual output
- **Memory profiling**: Check for leaks
- **Manual testing**: Final validation on target platforms
