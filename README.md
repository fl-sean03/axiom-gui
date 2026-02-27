# Axiom GUI - Desktop Application

**Tauri-based desktop application for Axiom molecular visualization**

## Overview

This is the Phase 4 GUI component of Axiom - a desktop application built with:
- **Backend:** Rust + Tauri 2.x
- **Frontend:** React 18 + TypeScript + Vite
- **UI:** Radix UI + Tailwind CSS
- **State:** Zustand

## Prerequisites

### System Requirements
- **Rust:** 1.70+ (install via [rustup](https://rustup.rs/))
- **Node.js:** 18+ (LTS recommended)
- **npm:** 9+

### Platform-Specific Dependencies

**Linux (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

**macOS:**
```bash
# Xcode Command Line Tools (if not installed)
xcode-select --install
```

**Windows:**
- Install [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- Install [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

## Installation

```bash
# Clone Axiom repo (if you haven't)
git clone <axiom-repo-url>
cd axiom/axiom-gui

# Install Tauri CLI
cargo install tauri-cli --version "^2.0.0"

# Install Node dependencies
npm install

# Build axiom-core (Rust backend)
cd ../axiom-core
cargo build --release
cd ../axiom-gui
```

## Development

### Run in Development Mode
```bash
npm run tauri dev
```

This will:
1. Start Vite dev server (React HMR)
2. Build Rust backend
3. Launch Tauri window with live reload

### Build for Production
```bash
npm run tauri build
```

Outputs:
- **Linux:** `src-tauri/target/release/bundle/appimage/axiom_*.AppImage`
- **macOS:** `src-tauri/target/release/bundle/dmg/Axiom_*.dmg`
- **Windows:** `src-tauri/target/release/bundle/msi/Axiom_*.msi`

## Project Structure

```
axiom-gui/
├── src/                      # React frontend
│   ├── components/           # UI components
│   ├── hooks/                # Custom hooks
│   ├── store/                # Zustand state
│   ├── types/                # TypeScript types
│   ├── utils/                # Utilities
│   ├── App.tsx               # Root component
│   └── main.tsx              # Entry point
├── src-tauri/                # Rust backend
│   ├── src/
│   │   └── main.rs           # Tauri commands
│   ├── Cargo.toml            # Rust dependencies
│   └── tauri.conf.json       # Tauri configuration
├── public/                   # Static assets
├── index.html                # HTML entry
├── package.json              # Node dependencies
├── tsconfig.json             # TypeScript config
├── vite.config.ts            # Vite config
└── tailwind.config.js        # Tailwind config
```

## Features

### Phase 4A (MVP) - Current
- [x] Desktop app launch
- [x] File loading (PDB, XYZ, LAMMPS, GRO)
- [x] 3D molecular viewer
- [x] Mouse controls (orbit, zoom, pan)
- [x] Rendering controls (SSAA, AO, backgrounds)
- [x] Semantic selection interface
- [x] Image export (PNG)

### Phase 4B (Future)
- [ ] GPU-accelerated rendering (wgpu compute shaders)
- [ ] Bond rendering (cylinders)
- [ ] Multiple representation modes (cartoon, sticks, ribbon)
- [ ] Trajectory support (XTC, DCD)
- [ ] Measurement tools (distance, angle)
- [ ] Custom color schemes

## Usage

### Basic Workflow
1. **Launch app:** `npm run tauri dev`
2. **Load file:** File → Open → select `1CRN.pdb`
3. **Interact:** Left-drag to orbit, scroll to zoom, right-drag to pan
4. **Adjust rendering:** Toggle SSAA, AO in sidebar
5. **Select atoms:** Enter query like `"element O"` in selection panel
6. **Export:** File → Save Image

### Semantic Selection Examples
```
element O                          # All oxygen atoms
within 5 of element N              # Atoms within 5Å of nitrogen
element C and within 10 of resname LIG    # Carbon near ligand
(element O or element N) and not resname WAT   # O or N, excluding water
```

## Troubleshooting

### Build Errors

**Error:** `webkit2gtk-4.1 not found`
```bash
# Linux: Install webkit2gtk
sudo apt install libwebkit2gtk-4.1-dev
```

**Error:** `cargo not found`
```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**Error:** `node-gyp rebuild failed`
```bash
# Rebuild native modules
npm rebuild
```

### Runtime Errors

**Error:** `Failed to load file`
- Check file format (PDB, XYZ, GRO, LAMMPS)
- Verify file is not corrupted
- Check console for detailed error

**Error:** `Render failed`
- Check atom count (>100K may be slow on CPU renderer)
- Try reducing SSAA setting
- Disable AO for faster rendering

### Performance Issues

**Slow rendering:**
- Reduce SSAA to 1x (off)
- Disable ambient occlusion
- Reduce atom count (select subset)

**Choppy mouse controls:**
- Close other applications
- Reduce window size
- Wait for initial render to complete

## Development Notes

### Adding New Features
1. Add Rust command in `src-tauri/src/main.rs`
2. Add TypeScript type in `src/types/axiom.ts`
3. Add React component in `src/components/`
4. Wire to state in `src/store/axiomStore.ts`
5. Test in dev mode: `npm run tauri dev`

### Hot Reload
- **Frontend changes:** Auto-reload (Vite HMR)
- **Backend changes:** Requires restart (Ctrl+C → `npm run tauri dev`)

### Debugging
- **Frontend:** Chrome DevTools (Ctrl+Shift+I in dev mode)
- **Backend:** `RUST_LOG=debug npm run tauri dev`
- **Tauri:** `TAURI_DEBUG=1 npm run tauri dev`

## Testing

```bash
# Unit tests (Rust)
cd src-tauri && cargo test

# Unit tests (TypeScript)
npm test

# Integration tests (Playwright)
npm run test:e2e

# Build test
npm run tauri build --debug
```

## Architecture

See [PHASE4_GUI_PLAN.md](../PHASE4_GUI_PLAN.md) for detailed architecture documentation.

## Contributing

This project is part of the Axiom molecular visualization platform. For contributing guidelines, see the main Axiom README.

## License

MIT OR Apache-2.0 (same as Axiom core)

## Credits

Built by the Heinz Interfaces Laboratory Agent
Computational Materials Science • CU Boulder
