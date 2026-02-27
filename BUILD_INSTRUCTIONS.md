# Axiom GUI - Build Instructions

**Complete step-by-step guide to building and running the Axiom desktop GUI**

---

## Prerequisites

### 1. Install Rust

**All Platforms:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 2. Install Node.js

**Recommended:** Node.js 18 LTS or higher

**macOS (Homebrew):**
```bash
brew install node
```

**Linux (Ubuntu/Debian):**
```bash
curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
sudo apt-get install -y nodejs
```

**Windows:**
Download from https://nodejs.org/

**Verify:**
```bash
node --version  # Should be v18+ or v20+
npm --version   # Should be 9+ or 10+
```

### 3. Install Tauri CLI

```bash
cargo install tauri-cli --version "^2.0.0"

# Verify
cargo tauri --version
```

### 4. Install Platform-Specific Dependencies

#### Linux (Ubuntu/Debian)
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

#### macOS
```bash
# Install Xcode Command Line Tools (if not already installed)
xcode-select --install
```

#### Windows
1. Install [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
2. Install [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually pre-installed on Windows 11)

---

## Building the GUI

### Step 1: Navigate to GUI Directory

```bash
cd /path/to/axiom/axiom-gui
```

If you're copying from the lab agent server:
```bash
# On your local machine
scp -r agent@91.98.162.122:/home/agent/projects/axiom/axiom-gui ./
cd axiom-gui
```

### Step 2: Install Node Dependencies

```bash
npm install
```

This will install:
- React, TypeScript, Vite
- Tauri plugins (dialog, fs)
- UI libraries (Radix, Tailwind)
- State management (Zustand)
- ~200 packages total, ~150MB

**Expected time:** 2-5 minutes

### Step 3: Generate Icon Assets

**Option A: Use placeholder (quick):**
```bash
# Create a simple SVG icon
cat > icon.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256">
  <rect width="256" height="256" fill="#3B82F6"/>
  <circle cx="128" cy="128" r="60" fill="#FFFFFF"/>
  <circle cx="100" cy="100" r="20" fill="#3B82F6"/>
  <circle cx="156" cy="100" r="20" fill="#3B82F6"/>
  <circle cx="128" cy="156" r="20" fill="#3B82F6"/>
</svg>
EOF

# Generate all icon formats
npx @tauri-apps/cli icon icon.svg
```

**Option B: Use custom icon:**
```bash
# If you have a high-res PNG or SVG
npx @tauri-apps/cli icon /path/to/your/icon.png
```

This generates:
- `src-tauri/icons/32x32.png`
- `src-tauri/icons/128x128.png`
- `src-tauri/icons/128x128@2x.png`
- `src-tauri/icons/icon.icns` (macOS)
- `src-tauri/icons/icon.ico` (Windows)

### Step 4: Build Axiom Core (Rust Backend)

The GUI depends on `axiom-core` crate. Build it first:

```bash
cd ../axiom-core
cargo build --release
cd ../axiom-gui
```

**Expected time:** 2-5 minutes (first build), 10-30 seconds (subsequent)

### Step 5: Run in Development Mode

```bash
npm run tauri:dev
```

This will:
1. Start Vite dev server (React HMR on port 1420)
2. Compile Rust backend (Tauri + axiom-core)
3. Launch desktop window with GUI

**Expected time:**
- First run: 3-8 minutes (compiles all Rust dependencies)
- Subsequent runs: 10-30 seconds

**What you should see:**
- Terminal shows Vite dev server + Tauri build logs
- Desktop window opens with Axiom GUI
- Empty canvas with "No Structure Loaded" message
- Toolbar with File/View/Help menus
- Sidebar with controls (collapsed or open)

### Step 6: Test Basic Functionality

**Load a structure:**
1. Click "File → Open"
2. Select a PDB file (e.g., `../1CRN.pdb`)
3. Should render molecular structure in canvas

**Test mouse controls:**
- Left drag → orbit view
- Right drag → pan view
- Scroll → zoom in/out

**Test rendering controls:**
1. Toggle SSAA (antialiasing)
2. Toggle Ambient Occlusion
3. Change background (black/white/transparent)

**Test selection:**
1. Enter query: `element O`
2. Click "Apply"
3. Should select oxygen atoms (count appears in status bar)

**Save image:**
1. Click "File → Save Image"
2. Choose save location
3. Verify PNG file created

---

## Building for Production

### Step 1: Create Release Build

```bash
npm run tauri:build
```

This will:
1. Build optimized React bundle (Vite production build)
2. Compile Rust with `--release` optimizations
3. Create platform-specific installer/bundle

**Expected time:** 5-15 minutes

**Output locations:**

**Linux:**
- `src-tauri/target/release/bundle/deb/axiom-gui_0.1.0_amd64.deb`
- `src-tauri/target/release/bundle/appimage/axiom-gui_0.1.0_amd64.AppImage`

**macOS:**
- `src-tauri/target/release/bundle/dmg/Axiom_0.1.0_x64.dmg`
- `src-tauri/target/release/bundle/macos/Axiom.app`

**Windows:**
- `src-tauri/target/release/bundle/msi/Axiom_0.1.0_x64_en-US.msi`
- `src-tauri/target/release/bundle/nsis/Axiom_0.1.0_x64-setup.exe`

### Step 2: Install/Distribute

**Linux (AppImage - recommended):**
```bash
chmod +x src-tauri/target/release/bundle/appimage/axiom-gui_*.AppImage
./src-tauri/target/release/bundle/appimage/axiom-gui_*.AppImage
```

**Linux (Debian package):**
```bash
sudo dpkg -i src-tauri/target/release/bundle/deb/axiom-gui_*.deb
axiom-gui
```

**macOS:**
- Double-click `.dmg` file
- Drag Axiom.app to Applications folder
- Open from Applications

**Windows:**
- Run `.msi` installer
- Launch from Start Menu

---

## Troubleshooting

### Build Errors

**Error: `webkit2gtk-4.1 not found`** (Linux)
```bash
sudo apt install libwebkit2gtk-4.1-dev
```

**Error: `cargo: command not found`**
```bash
# Restart shell after installing Rust, or:
source $HOME/.cargo/env
```

**Error: `EACCES: permission denied`** (npm install)
```bash
# Don't use sudo with npm. Fix permissions:
sudo chown -R $(whoami) ~/.npm
npm install
```

**Error: `failed to load axiom-core`**
```bash
# Build axiom-core first
cd ../axiom-core && cargo build --release && cd ../axiom-gui
```

**Error: `Vite port 1420 already in use`**
```bash
# Kill existing Vite server
lsof -ti:1420 | xargs kill -9
# Or change port in vite.config.ts
```

### Runtime Errors

**Error: `Failed to load file` (when opening PDB)**
- Check file format (must be valid PDB/XYZ/GRO/LAMMPS)
- Check file permissions (must be readable)
- Try with a known-good file (e.g., download from RCSB PDB)

**Error: `Render failed`**
- Check atom count (>100K may be slow on CPU renderer)
- Try reducing SSAA to 1x
- Disable Ambient Occlusion
- Check console logs for details

**Blank window or crash on launch**
- Check GPU drivers (macOS/Linux)
- Try updating WebView2 (Windows)
- Check console logs: `RUST_LOG=debug npm run tauri:dev`

### Performance Issues

**Slow rendering:**
- Current: CPU renderer (35s for 327 atoms @ 1920×1080 SSAA 2x)
- Reduce SSAA to Off (0x)
- Disable Ambient Occlusion
- Reduce window size
- Future: GPU renderer will be 100-1000x faster

**Choppy mouse controls:**
- Normal during active rendering
- Wait for render to complete before interacting
- Close resource-intensive applications

**Large file size:**
- Expected: ~10-20MB (Linux AppImage), ~30-50MB (macOS/Windows)
- Includes Chromium webview and Rust runtime
- Tauri is lighter than Electron (would be 100-200MB)

---

## Development Workflow

### Hot Reload (Dev Mode)

**Frontend changes** (React/TypeScript):
- Edit files in `src/`
- Vite auto-reloads (< 1 second)
- No restart needed

**Backend changes** (Rust/Tauri):
- Edit files in `src-tauri/src/`
- Stop dev server (Ctrl+C)
- Restart: `npm run tauri:dev`
- Full recompile (~10-30 seconds)

### Debugging

**Frontend (React):**
- Dev mode: Press `Ctrl+Shift+I` (Linux/Windows) or `Cmd+Option+I` (macOS)
- Opens Chrome DevTools
- Inspect elements, view console, debug React components

**Backend (Rust):**
```bash
# Enable Rust logging
RUST_LOG=debug npm run tauri:dev

# Or trace level (very verbose)
RUST_LOG=trace npm run tauri:dev
```

**Tauri internals:**
```bash
TAURI_DEBUG=1 npm run tauri:dev
```

### Testing

**Manual testing:**
1. Run dev mode: `npm run tauri:dev`
2. Test each feature (load, render, select, save)
3. Test error cases (invalid files, bad queries)

**Automated testing (future):**
```bash
# Unit tests (Rust)
cd src-tauri && cargo test

# Unit tests (TypeScript)
npm test

# E2E tests (Playwright)
npm run test:e2e
```

---

## Code Structure

```
axiom-gui/
├── src/                      # React frontend
│   ├── components/           # UI components
│   │   ├── App.tsx           # Root layout
│   │   ├── Toolbar.tsx       # Top menu
│   │   ├── Sidebar.tsx       # Left panel
│   │   ├── Canvas.tsx        # 3D viewer
│   │   ├── StatusBar.tsx     # Bottom status
│   │   ├── RenderControls.tsx
│   │   ├── CameraControls.tsx
│   │   ├── SelectionPanel.tsx
│   │   ├── StatsPanel.tsx
│   │   └── ui/               # Base UI components
│   ├── hooks/                # React hooks
│   │   ├── useAxiom.ts       # Axiom operations
│   │   └── useMouseControls.ts
│   ├── store/                # State management
│   │   └── axiomStore.ts     # Zustand store
│   ├── types/                # TypeScript types
│   │   └── axiom.ts
│   ├── utils/                # Utilities
│   │   ├── tauri.ts          # Tauri API wrappers
│   │   ├── camera.ts         # Camera math
│   │   └── cn.ts             # Class name merger
│   ├── main.tsx              # React entry
│   └── index.css             # Global styles
├── src-tauri/                # Rust backend
│   ├── src/
│   │   └── main.rs           # Tauri commands
│   ├── Cargo.toml            # Rust dependencies
│   ├── tauri.conf.json       # Tauri config
│   └── icons/                # App icons
├── package.json              # Node dependencies
├── tsconfig.json             # TypeScript config
├── vite.config.ts            # Vite config
├── tailwind.config.js        # Tailwind CSS
└── README.md                 # User guide
```

---

## Next Steps

After successful build:

1. **Test thoroughly**
   - Load different file formats (PDB, XYZ, GRO)
   - Test all rendering options
   - Test semantic selection
   - Test image export

2. **Performance tuning**
   - Profile rendering times
   - Optimize camera interaction
   - Consider GPU renderer upgrade (Phase 4B)

3. **Feature additions** (optional, Phase 4B)
   - Bond rendering
   - Multiple representation modes
   - Trajectory support
   - Measurement tools

4. **Distribution**
   - Create release on GitHub
   - Write user documentation
   - Package for distribution

---

## Support

**Issues during build?**
1. Check this troubleshooting section
2. Check Tauri docs: https://tauri.app/
3. Check axiom-gui/IMPLEMENTATION_STATUS.md for known issues
4. Report to Slack #lab-agent

**Build successful?**
You should now have a working Axiom GUI desktop application!

**Next:** Test with real molecular structures and report any bugs or feature requests.
