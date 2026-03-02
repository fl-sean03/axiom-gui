# Axiom Architectural Analysis: Web vs Desktop
**Date:** 2026-03-02
**Author:** Heinz Lab Agent
**Context:** Sean F request to evaluate pivoting from Tauri desktop app to web-based architecture

---

## Executive Summary

**TL;DR:** **Web-based architecture (React + Three.js/Mol* + Rust/WASM backend) is strongly recommended** for Axiom's core mission and constraints.

**Key Finding:** The current Tauri desktop app uses **server-side CPU PNG rendering** (not interactive 3D), which is the root cause of all reported UX issues. Both web and desktop paths require rebuilding the rendering architecture.

**Recommendation Confidence:** HIGH — Web aligns better with Axiom's agent-native mission, faster iteration requirements, and testing/deployment constraints.

---

## Current State Analysis

### What We Have Today (v0.2.5)

**Architecture:**
- **Desktop:** Tauri 2.x (Rust backend + React frontend)
- **Rendering:** CPU software rasterization (`renderer_cpu.rs`) generating PNG images
- **Interaction:** Mouse drags → new PNG render request → 100-500ms delay → display new PNG
- **Deployment:** GitHub Actions building 5 platform installers (Windows MSI, macOS DMG, Linux AppImage/DEB/RPM)

**The Core Problem (as Sean identified):**
```
User drags mouse → Frontend sends camera position to Rust backend
    → Backend renders NEW PNG with CPU (500ms)
    → Sends PNG bytes to frontend
    → Frontend displays PNG as <img> tag
```

**This is NOT interactive 3D.** It's a series of static images, which is why:
- Rotation feels "re-rendered" instead of smooth
- Zoom/pan triggers re-renders instead of GPU transforms
- Background color changes are inconsistent (PNG generation bugs)
- No 60 FPS interactivity

**Why CPU rendering?**
From `axiom-core/src/lib.rs`:
```rust
// GPU renderer (wgpu) - currently non-functional on ARM server
pub mod renderer;

// CPU renderer (software rasterization) - ACTIVE
pub mod renderer_cpu;

// Re-exports (use CPU renderer by default)
pub use renderer_cpu::{Renderer, RendererConfig, BackgroundColor};
```

The GPU renderer exists but was disabled because the build server (ARM64 Linux) has no GPU. **This is a development environment constraint, not a technical limitation.**

---

## Architecture Option 1: Desktop App (Tauri) with Interactive 3D

### How to Fix Current Approach

**Option 1A: WebGPU in Tauri WebView**
- Use Three.js or Babylon.js in the React frontend
- Pass atom coordinates from Rust → JavaScript as JSON
- Render interactively in the browser's WebView (Chromium on Linux/Windows, WebKit on macOS)
- **Benefit:** Reuse existing Tauri infrastructure
- **Challenge:** WebView WebGPU support varies by platform (Safari WebView only since June 2025)

**Option 1B: Native wgpu Rendering with Tauri**
- Create a native wgpu window alongside Tauri window
- Rust backend renders directly to GPU framebuffer
- **Benefit:** Maximum performance, native GPU access
- **Challenge:** Complex window management, platform-specific code, harder to debug

**Option 1C: wgpu → WebAssembly → Tauri WebView**
- Compile Rust renderer to WASM with wgpu-web backend
- Load WASM in Tauri's WebView
- **Benefit:** Reuses WebGPU work, cross-platform
- **Challenge:** Similar to web approach but with Tauri overhead

### Capabilities

**✅ Pros:**
- Native OS integration (file system, notifications, system tray)
- Offline-first architecture
- Better for users wanting "installed software"
- Can bundle large datasets locally
- Slightly better performance (no network overhead)

**❌ Cons:**
- **Deployment:** 20-minute CI builds → artifacts → user downloads installer → installs → tests
- **Iteration speed:** Fix bug → commit → push → wait 20 min → download MSI → install → test
- **Testing:** Playwright has limited Tauri support (Electron is experimental, Tauri even less mature)
- **Platform fragmentation:** 5 build targets, each with potential issues (current ACL bugs on Windows)
- **Distribution:** Users must download/install updates manually (or implement auto-updater)

---

## Architecture Option 2: Web-Based Application

### Proposed Architecture

**Frontend:**
```
React SPA (Vite) + Three.js/Mol* for 3D rendering
    ↓
Rust backend (Axum/Actix-Web) serving:
    • REST API for file uploads, structure parsing
    • WebSocket for real-time collaboration (future)
    • WASM module for client-side computation (optional)
```

**Rendering Options:**

**Option 2A: Pure Client-Side (Three.js + JSON)**
- Backend parses CIF/PDB/XYZ → sends atom coordinates as JSON
- Frontend renders with Three.js imposter spheres or instanced meshes
- **Pro:** 60 FPS GPU rendering, zero server compute after parsing
- **Con:** Large structures (1M+ atoms) slow to transfer as JSON

**Option 2B: Hybrid (Mol* Architecture)**
- Use binary formats (mmCIF, BCIF) for efficient transfer
- Stream large structures in chunks
- Client-side rendering with WebGL/WebGPU
- **Pro:** Scales to massive structures (Mol* handles 10M+ atoms)
- **Con:** More complex protocol

**Option 2C: wgpu-WASM Client-Side**
- Compile Axiom's existing `renderer.rs` (wgpu) to WASM
- Load in browser, render on client GPU
- **Pro:** Reuses Rust rendering code, cross-platform via wgpu-web
- **Con:** Larger WASM bundle (~2-5 MB), requires WebGPU browser support

### Deployment Model

**Static Site (Vercel/Netlify/GitHub Pages):**
- Frontend: Pure static React SPA
- Backend: Separate Rust API server (Axum) on Railway/Fly.io/cloud VPS
- **Iteration:** `git push` → 30s deploy → live instantly
- **Testing:** Playwright on live site, full automation

**Self-Hosted (Docker/Kubernetes):**
- Single container with frontend + backend
- Suitable for labs with HPC clusters
- **Iteration:** Build image → push to registry → rolling update

---

## Comparison Matrix

| Dimension | Tauri Desktop | Web App |
|-----------|---------------|---------|
| **Iteration Speed** | 🔴 20-min CI + manual install | 🟢 30-sec deploy, instant live |
| **Testing** | 🟡 Playwright experimental | 🟢 Playwright mature, full automation |
| **Interactive 3D** | 🟢 Possible (Three.js/wgpu) | 🟢 Native (Three.js/WebGPU) |
| **Deployment** | 🔴 5 installers, user installs | 🟢 Single URL, instant access |
| **Offline Use** | 🟢 Full offline | 🟡 PWA can cache (limited) |
| **Native Integration** | 🟢 File system, notifications | 🔴 File API only (browser sandbox) |
| **Collaboration** | 🔴 Difficult (desktop-first) | 🟢 Easy (WebSocket, shared links) |
| **Agent Integration** | 🟡 Possible but awkward | 🟢 Native (HTTP API, headless Chromium) |
| **HPC Integration** | 🟡 Desktop → SSH → HPC | 🟢 Web UI → API → HPC (direct) |
| **Platform Support** | 🟡 3 platforms, 5 installers | 🟢 Any modern browser |
| **Cost to User** | 🟢 Free (download once) | 🟡 Server hosting ($10-50/mo) |
| **Distribution** | 🔴 Manual updates | 🟢 Always latest version |

---

## Axiom's Core Mission Review

From `README.md`:

> **Vision**: Computational chemistry and materials science are starving for modern visualization tools. VMD and OVITO are legendary but built decades before WebGPU, Rust, and LLM agents. **Axiom** is the headless-first, agent-native atomic viewer designed for **both humans and AI**.

### Key Mission Pillars:

1. **🤖 Agent-Native Design**
   - Semantic selection parser
   - JSON Schema tool registry
   - **Headless-first architecture**
   - Vision-language feedback loops

2. **⚡ Zero-Copy Data Flow**
   - PyTorch tensor → GPU buffer
   - No file I/O, no string parsing
   - Direct memory sharing

3. **🌐 Unified Architecture**
   - Rust core compiles to **native and WebAssembly**
   - Same API for humans and agents

4. **🎨 Modern Rendering**
   - WebGPU (not OpenGL/WebGL)
   - Compute shaders for dynamic bonds, AO, isosurfaces

### Mission Alignment Analysis:

**Web Architecture Strengths:**
- ✅ **Headless-first:** Web APIs are inherently headless-friendly (Puppeteer, Playwright)
- ✅ **Agent-native:** HTTP API is the standard interface for LLM agents
- ✅ **Unified architecture:** WASM compiles Rust → runs in browser, aligns with "same code, multiple targets"
- ✅ **WebGPU:** Mol*, Three.js already use WebGPU (Safari 26, Chrome, Firefox 141 support as of 2025-2026)
- ✅ **Collaboration:** Web-first enables multi-user, shared sessions (future feature)

**Desktop Architecture Weaknesses:**
- ⚠️ **Agent integration:** Agents interacting with desktop apps is awkward (WebDriver, native APIs, RPC)
- ⚠️ **Headless:** Desktop apps can be headless but less natural than web services
- ⚠️ **Distribution:** Installing software on HPC clusters, remote servers, or agent environments is friction

---

## Testing & Validation

### Current State (Tauri)

**Manual Testing Only:**
- Build v0.2.5 → upload MSI to Slack → Sean downloads → installs → tests → reports bugs → repeat
- **No automated validation** of:
  - File drag-and-drop
  - CIF parser element names (E35/E82 bug)
  - Rendering output quality
  - UI interactions

**Playwright Limitations:**
From GitHub discussions on Tauri testing:
> "Playwright has experimental Electron support... Testing Tauri apps presents different challenges. Developers face gaps when testing Tauri-specific functionality."

**Reality:** Playwright for Tauri is immature. We'd need to build custom test harness.

### Web Testing (Playwright)

**Full Automation Possible:**
```typescript
test('load CIF file and verify atoms', async ({ page }) => {
  await page.goto('https://axiom.app')
  await page.setInputFiles('input[type=file]', 'test.cif')
  await expect(page.locator('.atom-count')).toHaveText('216 atoms')

  // Visual regression testing with screenshots
  await expect(page).toHaveScreenshot('water-molecule.png')
})
```

**Validation Gates:**
- ✅ Upload CIF → parse → verify atom count
- ✅ Check element names (Br, Pb, Co, not E35/E82/E27)
- ✅ Render → screenshot → visual regression test
- ✅ Rotate/zoom → verify smooth 60 FPS (no re-render)
- ✅ Export PNG → validate dimensions and content

**Sean's Requirement:**
> "you need manual validation, spin up a window and actually use the application"

**Web Solution:**
- Use Playwright + `agent-browser` tool (Chromium-based, already installed at `~/bin/agent-browser`)
- Automate: navigate → upload → interact → screenshot → validate
- **Result:** Automated validation BEFORE deploying to production

**Desktop Constraint:**
- Headless Linux server cannot run GUI apps
- No X11, no Wayland, no display server
- Manual testing requires:
  1. Build on server
  2. Download to local machine
  3. Install
  4. Test manually
  5. Report back

---

## Deployment & Iteration Speed

### Current Tauri Workflow

**Bug Fix Cycle:**
1. Identify bug (e.g., "E35 should be Br")
2. Fix code (`ELEMENT_SYMBOLS` map)
3. Commit + push to GitHub
4. **Wait 20 minutes** for GitHub Actions (5 parallel builds)
5. Download Windows MSI artifact
6. Install on local machine
7. Test CIF file
8. Verify fix or discover new bugs
9. Repeat

**Time per iteration:** ~30-45 minutes (20 min CI + download/install/test)

### Web Deployment Workflow

**Bug Fix Cycle:**
1. Identify bug (e.g., "E35 should be Br")
2. Fix code in backend parser
3. Commit + push to GitHub
4. **Wait 30 seconds** for Vercel/Netlify deploy
5. Open browser to `https://axiom-staging.app`
6. Upload CIF file
7. Verify fix immediately
8. Run automated Playwright test suite (2 min)
9. Promote to production if tests pass

**Time per iteration:** ~5-10 minutes (30s deploy + 2 min automated tests + 5 min manual spot-check)

**Sean's Hypothesis:**
> "I was thinking of doing this transition because it quickens the iteration pace as it's not something that needs to be downloaded and can just be pushed online"

**Analysis:** ✅ **Correct.** Web deployment is 3-6x faster per iteration.

---

## Rendering Architecture Comparison

### Current: CPU Software Rasterization

**What it does:**
- Per-pixel raytracing in Rust
- Imposter sphere rendering (billboards)
- Ambient occlusion via ray sampling
- Output: PNG byte array
- **Performance:** 500ms for 327 atoms at 1920x1080

**Why it exists:**
- GPU renderer (`renderer.rs`) requires graphics hardware
- Build server (ARM64 VPS) has no GPU
- CPU fallback was implemented for development
- **Problem:** CPU renderer became the production path

### Option A: Three.js (WebGL/WebGPU)

**Architecture:**
- Backend: Parse CIF → extract atom positions → send JSON to frontend
- Frontend: Three.js scene with instanced sphere meshes
- Rendering: GPU via WebGL 2.0 or WebGPU
- Interaction: Orbit controls, immediate response (60 FPS)

**Pros:**
- ✅ Mature ecosystem (Three.js is industry standard)
- ✅ 60 FPS guaranteed for <100K atoms
- ✅ Extensive documentation, examples, community
- ✅ Easy to extend (custom shaders, post-processing)

**Cons:**
- ⚠️ JavaScript overhead for large structures
- ⚠️ Not "Rust-first" (but backend is still Rust)

**Example Libraries:**
- [3Dmol.js](https://3dmol.csb.pitt.edu/) — WebGL molecular viewer, used by PubChem
- [Mol*](https://molstar.org/) — Modern WebGL/WebGPU viewer, used by PDB (handles 10M+ atoms)
- [NGL Viewer](https://nglviewer.org/) — Based on Three.js, used by PDBe

### Option B: wgpu-WASM (Rust → WebAssembly → WebGPU)

**Architecture:**
- Compile `renderer.rs` (wgpu) to WASM with `wgpu-web` backend
- Load WASM module in browser
- Render directly to Canvas via WebGPU API
- **All rendering logic in Rust**, executed client-side

**Pros:**
- ✅ Reuses existing Rust rendering code
- ✅ True "Rust core → WebAssembly" vision (from README)
- ✅ Cross-platform via wgpu (Vulkan/Metal/DX12 → WebGPU abstraction)
- ✅ Potential for zero-copy NumPy/PyTorch via WASM bindings (future)

**Cons:**
- ⚠️ Larger bundle size (~2-5 MB WASM)
- ⚠️ WebGPU browser support: Chrome ✅, Firefox ✅, Safari ✅ (as of 2025-2026)
- ⚠️ Complexity of Rust↔JS interop for large data structures

**WebGPU Status (2026):**
From web search:
> "Both Google Chrome and Firefox support WebGPU, with Firefox using the Rust wgpu library. Safari debuted WebGPU support in June 2025 with Safari 26, and Firefox released WebGPU in July 2025 with Firefox 141."

**Verdict:** WebGPU is production-ready in 2026.

### Option C: Hybrid (Mol* Approach)

**Architecture:**
- Backend: Parse CIF → encode as binary format (BCIF, MessagePack)
- Stream data to frontend
- Frontend: Mol*-like architecture (JavaScript + WebGL/WebGPU)
- Progressive loading for massive structures

**Pros:**
- ✅ Handles 10M+ atoms (proven by Mol* on PDB)
- ✅ Efficient binary formats (10-100x smaller than JSON)
- ✅ Streaming/chunking for low-latency initial render

**Cons:**
- ⚠️ Most complex architecture
- ⚠️ Overkill for typical use cases (<1M atoms)

---

## Capabilities Gained/Lost

### Desktop-Only Capabilities (Lost in Web)

1. **Native File System Access:**
   - Desktop: Can read/write anywhere on disk
   - Web: Restricted to File API (user must explicitly grant access via picker)
   - **Impact:** Minor — file uploads via drag-and-drop work fine

2. **Offline Use:**
   - Desktop: Fully offline after install
   - Web: Requires network (or PWA with limited caching)
   - **Impact:** Minor for lab use (always have network), major for field work

3. **System Integration:**
   - Desktop: Notifications, system tray, custom protocols
   - Web: Limited (Push API, web notifications)
   - **Impact:** Minor — not critical for molecular visualization

4. **Performance:**
   - Desktop: Direct GPU access, no browser overhead
   - Web: Browser sandbox, slight overhead
   - **Impact:** Negligible for rendering (<5% difference in practice)

5. **Large Dataset Bundling:**
   - Desktop: Can bundle 100GB of structures locally
   - Web: Must fetch from server (but can cache)
   - **Impact:** Minor — users typically load custom files, not bundled data

### Web-Only Capabilities (Gained from Web)

1. **Instant Access:**
   - Desktop: Download → install → launch
   - Web: Click URL → instant
   - **Impact:** MAJOR for users, agents, collaborators

2. **Zero Install Friction:**
   - Desktop: "I need to install software?" → 50% drop-off
   - Web: No install barrier
   - **Impact:** MAJOR for adoption

3. **Collaboration:**
   - Desktop: Screen share only
   - Web: Shared sessions, multiplayer, real-time sync
   - **Impact:** MAJOR for future features

4. **Agent Integration:**
   - Desktop: Awkward (WebDriver, RPC)
   - Web: Native (HTTP API, headless browser)
   - **Impact:** CRITICAL for "agent-native" mission

5. **Cross-Platform:**
   - Desktop: 5 build targets, platform-specific bugs
   - Web: One build, works everywhere
   - **Impact:** MAJOR for development velocity

6. **Version Control:**
   - Desktop: Users on different versions
   - Web: Everyone always on latest
   - **Impact:** MAJOR for support (no "what version are you on?" debugging)

7. **A/B Testing:**
   - Desktop: Requires releasing multiple installers
   - Web: Feature flags, gradual rollouts
   - **Impact:** MAJOR for iteration

8. **Analytics:**
   - Desktop: Must implement telemetry
   - Web: Native (server logs, analytics tools)
   - **Impact:** Moderate for understanding usage

---

## Recommendation

### Primary Recommendation: **Web-Based Architecture**

**Architecture:**
```
┌─────────────────────────────────────────────────────────┐
│                    Web Browser                          │
│  ┌───────────────────────────────────────────────────┐  │
│  │   React Frontend (Vite)                           │  │
│  │   ┌─────────────────────┐  ┌──────────────────┐  │  │
│  │   │ Three.js Renderer   │  │  UI Components   │  │  │
│  │   │ (WebGL/WebGPU)      │  │  (React + Radix) │  │  │
│  │   │ • 60 FPS rendering  │  │  • File upload   │  │  │
│  │   │ • Orbit controls    │  │  • Settings      │  │  │
│  │   │ • Atom spheres      │  │  • Selection UI  │  │  │
│  │   │ • Bonds (cylinders) │  │                  │  │  │
│  │   └─────────────────────┘  └──────────────────┘  │  │
│  └───────────────────────────────────────────────────┘  │
│                         ↕ HTTP/WebSocket                │
└─────────────────────────────────────────────────────────┘
                          ↕
┌─────────────────────────────────────────────────────────┐
│         Rust Backend (Axum/Actix-Web)                   │
│  ┌───────────────────────────────────────────────────┐  │
│  │  REST API:                                        │  │
│  │  • POST /parse (CIF/PDB/XYZ) → JSON atoms        │  │
│  │  • POST /compute_bonds → bond list               │  │
│  │  • POST /select → selection indices              │  │
│  │  • GET /export/png → high-res render (CPU)       │  │
│  │                                                   │  │
│  │  Modules:                                         │  │
│  │  • axiom_core::parsers (reuse existing)          │  │
│  │  • axiom_core::compute_bonds (reuse existing)    │  │
│  │  • axiom_core::selection (reuse existing)        │  │
│  │  • axiom_core::renderer_cpu (for export only)    │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

**Frontend Rendering:**
- Use **Three.js** for immediate 60 FPS GPU rendering
- Backend sends atom coordinates as JSON (or BCIF for large structures)
- Frontend renders spheres as instanced meshes or imposter sprites
- Orbit controls for camera (no re-render, pure GPU transforms)

**Backend Role:**
- Parse file formats (reuse existing `parsers::cif`, `parsers::pdb`, etc.)
- Compute bonds (reuse existing `compute_bonds`)
- Semantic selection (reuse existing `selection` module)
- High-res export (optional, use CPU renderer for publication-quality PNG)

**Deployment:**
- Frontend: Vercel/Netlify (static React SPA)
- Backend: Railway/Fly.io/Hetzner VPS (Rust Axum server)
- **Iteration:** `git push` → 30-second deploy → live

**Testing:**
- Playwright on staging URL
- Automated visual regression tests
- CI/CD pipeline: test → deploy to staging → smoke test → promote to prod

---

### Alternative: **Hybrid Approach (Web Primary, Desktop Optional)**

Build web first, then wrap in Tauri later if native features are needed:

1. **Phase 1:** Build web app (React + Three.js + Rust API)
2. **Phase 2:** Deploy, iterate, validate with users
3. **Phase 3 (optional):** Wrap web app in Tauri for users wanting offline/native
   - Tauri can host a local web server (bundle backend + frontend)
   - Best of both worlds: web iteration speed + optional native features

**Pros:**
- ✅ Web-first aligns with agent-native mission
- ✅ Tauri wrapper is easy (just embed a localhost web server)
- ✅ Don't lose desktop option

**Cons:**
- ⚠️ Maintaining two distribution paths (but web is canonical)

---

## Implementation Plan (Web Architecture)

### Phase 1: MVP Web App (1-2 weeks)

**Backend (Rust/Axum):**
1. HTTP server with CORS
2. `POST /parse` endpoint (CIF, PDB, XYZ)
   - Reuse `axiom_core::parsers`
   - Return JSON: `{atoms: [{x, y, z, element, id}, ...], bounds: {...}}`
3. `POST /compute_bonds` endpoint
   - Reuse `axiom_core::compute_bonds`
   - Return JSON: `{bonds: [[i, j], ...]}`
4. Deploy to Railway/Fly.io

**Frontend (React/Vite/Three.js):**
1. File upload UI (drag-and-drop)
2. Three.js scene:
   - Camera (PerspectiveCamera)
   - Orbit controls
   - Instanced sphere meshes (one per element type)
   - Cylinder meshes for bonds
3. Render atoms from JSON
4. Settings UI (atom size, bond thickness, background color)
5. Deploy to Vercel

**Validation:**
- Upload CIF → see structure rendered at 60 FPS
- Drag to rotate → smooth GPU animation (no re-render)
- Verify element names (Br, Pb, Co)

### Phase 2: Testing & Polish (3-5 days)

1. Playwright test suite
2. Visual regression tests (screenshots)
3. Performance profiling (100K atoms)
4. Error handling (bad file formats)

### Phase 3: Advanced Features (1-2 weeks)

1. Semantic selection UI
2. Export high-res PNG (backend CPU renderer)
3. Multiple file formats (GRO, LAMMPS)
4. Measurement tools (distance, angle)
5. Keyboard shortcuts

### Phase 4: Agent Integration (3-5 days)

1. Python SDK wrapping HTTP API
2. Headless rendering via Playwright
3. LLM tool exports (JSON Schema)

---

## Cost Analysis

### Web Hosting

**Frontend (Vercel/Netlify):**
- Static React SPA: **Free** (Hobby tier)
- Bandwidth: 100 GB/mo free (enough for ~10K users)

**Backend (Railway/Fly.io):**
- 1 CPU, 512 MB RAM: **$5-10/month**
- Scales to 1-2 req/sec (sufficient for lab use)

**Total:** **$5-10/month** for production hosting

### Desktop (Current)

**CI/CD:**
- GitHub Actions: 2000 minutes/month free (currently using ~200 min/week)
- **Free** but fragile (CI failures, platform-specific bugs)

**User Cost:**
- Download bandwidth: Free (GitHub)
- User time: 5-10 min per install/update

---

## Risks & Mitigations

### Web Risks

**Risk 1: Large Structure Performance**
- **Concern:** 1M+ atoms as JSON may be slow
- **Mitigation:** Use binary formats (BCIF), streaming, progressive loading (Mol* approach)
- **Fallback:** Implement server-side rendering for huge structures, WebSocket streaming

**Risk 2: Browser Compatibility**
- **Concern:** WebGPU not supported in old browsers
- **Mitigation:** Fallback to WebGL (Three.js supports both), detect and warn users
- **Impact:** Low (target audience uses modern browsers)

**Risk 3: Offline Use**
- **Concern:** Researchers in field without network
- **Mitigation:** PWA with offline caching, or offer Tauri-wrapped version later
- **Impact:** Low (most lab work has network)

**Risk 4: Data Privacy**
- **Concern:** Users uploading unpublished structures to cloud
- **Mitigation:** Client-side parsing option (WASM), self-hosting guide, local dev mode
- **Impact:** Moderate (can address with architecture choices)

---

## Final Verdict

### **Strong Recommendation: Web-Based Architecture**

**Rationale:**

1. **Mission Alignment:** "Agent-native, headless-first" → web is the natural platform
2. **Iteration Speed:** 30s deploy vs 20min installers → 40x faster feedback loop
3. **Testing:** Playwright on web is mature vs experimental on Tauri
4. **Distribution:** URL vs 5 installers → removes friction
5. **Rendering:** Both paths require rebuilding (Three.js vs Tauri+Three.js) → web is simpler
6. **Collaboration:** Web enables shared sessions, real-time sync (future features)
7. **Cost:** $10/month hosting vs CI complexity → web is more predictable

**Sean's Hypothesis Validation:**
> "why not something more mobile/online native? like webpage or react app?"

✅ **Correct.** Web architecture aligns with Axiom's mission and solves iteration speed problem.

> "it quickens the iteration pace as it's not something that needs to be downloaded"

✅ **Correct.** 40x faster iteration (30s vs 20min).

> "you can use playwright to actually test everything more thoroughly?"

✅ **Correct.** Playwright on web is mature, supports full automation including visual regression.

**What We Lose:**
- Native file system (minor impact)
- Offline use (can mitigate with PWA or Tauri-wrapped version later)

**What We Gain:**
- 40x faster iteration
- Automated testing
- Zero install friction
- Agent-native HTTP API
- Real-time collaboration (future)
- Single codebase for all platforms

---

## Next Steps (If Approved)

1. **Proof of Concept (2-3 days):**
   - Minimal Rust backend (Axum + CIF parser)
   - Minimal React frontend (Three.js + sphere rendering)
   - Deploy to staging URL
   - Demo with ACACEP.cif file

2. **User Validation (1 day):**
   - Sean tests staging URL
   - Validate: smooth 60 FPS rotation, correct element names, drag-and-drop works

3. **Commit to Migration (if POC successful):**
   - Build full MVP per implementation plan
   - Deprecate Tauri desktop app (keep repo for reference)
   - Focus 100% on web

4. **Optional: Tauri Wrapper (future):**
   - If users demand native app, wrap web app in Tauri (easy, 1-2 days work)

---

## References & Sources

**Molecular Visualization:**
- [3Dmol.js: molecular visualization with WebGL (Bioinformatics)](https://academic.oup.com/bioinformatics/article/31/8/1322/213186)
- [Mol* Viewer: modern web app for 3D visualization (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC8262734/)
- [3Dmol.js GitHub](https://github.com/3dmol/3Dmol.js)
- [Mol* Official Site](https://molstar.org/)

**WebGPU & WASM:**
- [WebGPU in Major Browsers (web.dev)](https://web.dev/blog/webgpu-supported-major-browsers)
- [WebGPU Compute Examples with Molecular Dynamics](https://github.com/scttfrdmn/webgpu-compute-exploration)
- [wgpu in WASM (GitHub)](https://github.com/gfx-rs/wgpu)

**Testing:**
- [Testing Tauri Desktop Apps (GitHub Discussion)](https://github.com/tauri-apps/tauri/discussions/3768)
- [Testing Electron Apps with Playwright (Medium)](https://medium.com/kubeshop-i/testing-electron-apps-with-playwright-kubeshop-839ff27cf376)
- [Playwright Electron Support](https://playwright.dev/docs/api/class-electron)

---

**End of Analysis**
