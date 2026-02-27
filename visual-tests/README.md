# Axiom GUI Visual Validation Tests

Fast visual validation for Axiom molecular visualization using small structures.

## Quick Start

```bash
# Run programmatic rendering tests (recommended)
cd /home/agent/projects/axiom
/home/agent/projects/axiom/axiom-py/venv/bin/python3 \
  /home/agent/projects/axiom/axiom-gui/visual-tests/scripts/direct_render_test.py
```

**Duration**: ~3 seconds
**Output**: 12 PNG screenshots in `screenshots/`
**Report**: `VISUAL_VALIDATION_REPORT.md`

## What's Tested

- ✓ Rotation (multiple camera angles)
- ✓ Zoom (multiple camera distances)
- ✓ Background colors (black, white, gray)
- ✓ SSAA anti-aliasing
- ✓ Small structures (3-9 atoms, <1s render time)

## Test Structures

| File | Atoms | Description |
|------|-------|-------------|
| `water.xyz` | 3 | H2O - minimal test |
| `methane.xyz` | 5 | CH4 - rotation test |
| `ethanol.xyz` | 9 | C2H6O - zoom test |
| `glycine.pdb` | 9 | Amino acid - PDB format |

## Directory Structure

```
visual-tests/
├── structures/          # Test molecules
├── screenshots/         # Generated images
├── scripts/            # Test scripts
├── VISUAL_VALIDATION_REPORT.md    # Detailed results
├── VALIDATION_SUMMARY.md          # Executive summary
└── README.md           # This file
```

## Available Scripts

### 1. direct_render_test.py (Recommended)

Programmatic rendering via axiom Python bindings.

**Usage**:
```bash
cd /home/agent/projects/axiom
/home/agent/projects/axiom/axiom-py/venv/bin/python3 \
  visual-tests/scripts/direct_render_test.py
```

**Tests**: Rotation, zoom, backgrounds
**Time**: ~3 seconds
**Output**: 12 screenshots + report

### 2. manual_visual_test.sh

Launches GUI and captures screenshots over time.

**Usage**:
```bash
# Requires Xvfb on :99
./scripts/manual_visual_test.sh
```

**Tests**: GUI startup, rendering over time
**Time**: ~2 minutes
**Output**: Timed screenshots

### 3. webdriver_visual_test.py

WebDriver-based GUI automation (requires ChromeDriver).

**Usage**:
```bash
python3 scripts/webdriver_visual_test.py
```

**Tests**: GUI elements, file loading, interaction
**Time**: Variable
**Output**: Screenshots + interaction report

## Test Results

**Status**: ✓ PASSED (4/4 tests)
**Images**: 12 screenshots
**Performance**: <1s per render
**Bugs**: None found

See `VALIDATION_SUMMARY.md` for full details.

## Performance

| Structure | Atoms | Render Time |
|-----------|-------|-------------|
| Water | 3 | <1s |
| Methane | 5 | <1s |
| Ethanol | 9 | <1s |

**60x faster** than previous 327-atom tests (3s vs 3+ minutes).

## Visual Examples

All screenshots show high-quality rendering with SSAA 2x anti-aliasing:

- **Water**: Red oxygen + white hydrogens
- **Methane**: Tetrahedral structure from 4 angles
- **Ethanol**: Progressive zoom levels
- **Backgrounds**: Black/white/gray variations

## Requirements

- Python 3.12+
- Axiom with Python bindings (in venv)
- PIL/Pillow (for image handling)
- Xvfb (for headless GUI testing)
- Optional: ChromeDriver (for WebDriver tests)

## Adding New Tests

Edit `scripts/direct_render_test.py`:

```python
def test_new_feature(self):
    """Test description"""
    atoms = axiom.Atoms()
    # Add atoms...

    renderer = axiom.Renderer(width=800, height=600)
    # Configure renderer...

    renderer.save_image(atoms, "output.png")
```

Keep structures small (3-20 atoms) for fast iteration.

## Troubleshooting

**"Cannot import axiom"**:
- Use the venv: `/home/agent/projects/axiom/axiom-py/venv/bin/python3`
- Run from `/home/agent/projects/axiom` directory

**No images generated**:
- Check `screenshots/` directory exists
- Verify axiom.so is built
- Check write permissions

**Slow rendering**:
- Use smaller structures (3-20 atoms)
- Reduce image size (default 800x600)
- Disable SSAA if needed

## Next Steps

1. Run programmatic tests for quick validation
2. Use small structures for fast iteration
3. Add GUI-specific tests as needed
4. Compare screenshots for regression testing

## License

Part of Axiom molecular visualization project.
