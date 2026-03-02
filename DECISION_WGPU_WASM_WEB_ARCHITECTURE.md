# FINAL DECISION: Axiom Web Architecture — wgpu-WASM Client-Side Rendering

**Date:** March 2, 2026
**Decision Authority:** Sean F (Heinz Lab)
**Implementation Owner:** axiom-agent
**Status:** APPROVED — Implementation Starting Immediately

---

## Executive Summary

**DECISION: Pivot Axiom from Tauri desktop app to web-based architecture using wgpu-WASM client-side rendering.**

**Rationale:** This is the **highest quality, lowest tech debt** path forward. It reuses existing Rust renderer code, provides a unified codebase for web/CLI/headless use cases, eliminates future rewrites, and aligns with the "Rust core → WebAssembly" vision.

**Timeline:** 6-8 weeks for production-grade implementation (quality over speed, per Feb 28 directive)
**No POC phase** — we build the final architecture from day one.

---

## Why wgpu-WASM (Not Three.js)

### 1. Zero Tech Debt from Architecture Pivot
- Current codebase has `renderer.rs` (wgpu) — **already built and validated**
- Three.js approach = throw away Rust renderer, rebuild in JavaScript (duplicate effort, tech debt)
- wgpu-WASM approach = **reuse existing renderer code**, compile to WASM
- Leverage existing investment in wgpu instead of abandoning it

### 2. Future-Proof Technology Stack
- wgpu → WebGPU → cross-platform GPU rendering (native + web from ONE codebase)
- Three.js = JavaScript-only, can't share code with desktop/CLI/headless Rust tools
- wgpu-WASM = write once (Rust), run everywhere (native, web, WASM)
- Aligns with "Rust core" vision — not a JavaScript rewrite

### 3. Performance Ceiling = Highest
- wgpu provides direct GPU control: compute shaders, custom pipelines, zero-copy buffers
- Three.js = abstraction layer, harder to optimize for massive structures (1M+ atoms)
- When advanced features needed (ambient occlusion, custom shaders, massive structures), wgpu is ready
- No performance compromises

### 4. Headless-First, Agent-Native Architecture
Same `renderer.rs` code powers:
- **Web UI:** wgpu-WASM in browser (interactive visualization)
- **CLI:** wgpu-native headless (screenshot generation)
- **HPC:** Batch rendering on Alpine cluster (same code, no duplication)

Three.js = web-only, can't unify headless + interactive paths

### 5. Minimizes Future Rewrites
- **If we start with Three.js today:** Will rewrite to Rust later when performance/features demand it
- **If we start with wgpu-WASM today:** This IS the final architecture — no future rewrites
- Pay the WASM complexity cost ONCE, never rewrite the renderer again

---

## What We're Trading (The Hard Parts)

### Complexity We Accept:
1. **WASM Build Setup** (one-time cost)
   - Configure `wasm-pack`, `wasm-bindgen`, WebGPU bindings
   - Larger bundle (~2-5 MB vs ~500 KB for Three.js)
   - Slightly longer initial load time
   - **Payoff:** Never rewrite the renderer, unified codebase

2. **Browser Compatibility Requirements**
   - WebGPU support required: Chrome ✓, Firefox ✓, Safari ✓ (2025-2026)
   - Lab users expected to have modern browsers
   - **Payoff:** Cutting-edge rendering capabilities

3. **WASM Debugging**
   - Less mature than JavaScript debugging
   - Need source maps, wasm-debugging extensions
   - **Payoff:** Production-grade reusable code when it works

### Speed We Trade for Quality:
- **Three.js POC:** 2-3 days to validate web approach
- **wgpu-WASM production:** 6-8 weeks for final architecture
- **Accepted trade:** 6-8 weeks for zero-tech-debt solution vs fast throwaway POC

---

## Architecture Overview

### High-Level Flow:

```
User Browser
    ↓
React Frontend (Vite/TypeScript)
    ↓
axiom-renderer.wasm (Rust → WASM)
    ↓
WebGPU (browser GPU)
    ↓
60 FPS interactive 3D rendering
```

### Backend (Optional, for CIF parsing):

```
Rust Backend (Axum HTTP API)
    ↓
CIF Parser (axiom-core)
    ↓
JSON atom coordinates → Frontend
```

**OR** parse CIF entirely in WASM (no backend needed)

---

## Unified Codebase Architecture

### Rust Crates:

```
axiom-core/          # CIF parser, data structures (existing)
axiom-renderer/      # NEW: GPU rendering (extracted from renderer.rs)
    ├── src/
    │   ├── lib.rs           # Core rendering logic
    │   ├── wasm.rs          # WASM bindings (target = wasm32)
    │   ├── native.rs        # Native bindings (CLI, headless)
    │   └── shaders/         # WGSL shaders
    └── Cargo.toml           # wgpu dependency (supports native + web)

axiom-web/           # NEW: React frontend
    ├── src/
    │   ├── components/      # React UI components
    │   ├── wasm/            # WASM loader, axiom-renderer bindings
    │   └── App.tsx          # Main app
    └── package.json         # Vite, React, TypeScript

axiom-cli/           # EXISTING: CLI tool (uses axiom-renderer native)
axiom-backend/       # OPTIONAL: Axum HTTP API (if needed)
```

### Build Targets:

| Target                | Output                        | Use Case                          |
|-----------------------|-------------------------------|-----------------------------------|
| `wasm32-unknown-unknown` | `axiom-renderer.wasm`         | Web UI (browser)                  |
| `x86_64-unknown-linux-gnu` | `axiom-cli` binary           | CLI screenshot generation         |
| `x86_64-unknown-linux-gnu` | `axiom-backend` binary (opt) | HTTP API for CIF parsing          |

---

## Quality Gates (Must Pass Before Moving to Next Phase)

### Phase 1 Quality Gates: ✅ wgpu-WASM Foundation Complete
- [ ] `axiom-renderer` crate compiles for `wasm32-unknown-unknown` target
- [ ] WASM bundle loads in browser without errors
- [ ] WebGPU context initializes successfully (Chrome, Firefox, Safari tested)
- [ ] Minimal test: Render single sphere on screen at 60 FPS
- [ ] Bundle size ≤ 5 MB (optimized build)
- [ ] Source maps generated for debugging

### Phase 2 Quality Gates: ✅ CIF Parsing & Rendering Complete
- [ ] CIF parser works in WASM (or backend JSON API works)
- [ ] Load water molecule (H2O.cif) → correct atom positions rendered
- [ ] Load methane (CH4.cif) → correct atom positions, bonds rendered
- [ ] Load ACACEP.cif → correct structure (no crashes, correct element names)
- [ ] 60 FPS rotation with mouse drag (tested in Chrome, Firefox, Safari)
- [ ] Zoom in/out with mouse wheel (smooth, no jitter)
- [ ] Camera controls match desktop version behavior

### Phase 3 Quality Gates: ✅ Feature Parity with Tauri Desktop
- [ ] Drag-and-drop CIF file upload works
- [ ] File picker dialog works (browser File API)
- [ ] Recent files list works (localStorage or backend)
- [ ] Export PNG screenshot works (canvas.toDataURL)
- [ ] Selection tools work (click atom → highlight, show info panel)
- [ ] Measurements work (distance, angle tools)
- [ ] Rendering settings UI works (sphere/stick, colors, quality)
- [ ] Visual quality matches or exceeds desktop version

### Phase 4 Quality Gates: ✅ Playwright Testing Infrastructure
- [ ] Playwright test suite runs in CI (GitHub Actions)
- [ ] Visual regression tests: Compare screenshots vs baseline PNGs
- [ ] Functional tests: Load CIF, rotate, zoom, select, export
- [ ] Cross-browser tests: Chrome, Firefox, Safari (via Playwright)
- [ ] Performance tests: 60 FPS maintained for typical structures (<100K atoms)
- [ ] Accessibility tests: Keyboard navigation, ARIA labels, screen reader support

### Phase 5 Quality Gates: ✅ Production Deployment Ready
- [ ] Deployed to staging URL (Vercel/Cloudflare Pages)
- [ ] Tested with 10+ real lab CIF files (water, methane, perovskites, MXenes)
- [ ] No console errors in browser DevTools
- [ ] Bundle size optimized (<5 MB WASM, <500 KB JS)
- [ ] Initial load time <3 seconds on typical connection
- [ ] Mobile browser tested (iOS Safari, Android Chrome) — basic functionality works
- [ ] Documentation complete (README, user guide, developer setup guide)
- [ ] Tauri desktop version deprecated (marked as legacy, link to web version)

### Phase 6 Quality Gates: ✅ Headless Unification Complete
- [ ] CLI tool (`axiom-cli screenshot`) uses same `axiom-renderer` crate
- [ ] CLI generates identical screenshots to web version (pixel-perfect match)
- [ ] HPC batch rendering script uses same renderer (tested on Alpine)
- [ ] Zero code duplication between web/CLI/headless rendering paths
- [ ] Performance benchmarks: Web 60 FPS interactive, CLI <1s per screenshot

---

## Success Criteria (Final Validation)

**Before declaring Axiom web version "production-ready", ALL must pass:**

1. **Functional Completeness**
   - All features from Tauri desktop version work in web version
   - No regressions in functionality
   - User experience equal to or better than desktop

2. **Performance**
   - 60 FPS rotation for structures <100K atoms
   - <3s initial load time
   - Smooth zoom, pan, selection interactions

3. **Quality**
   - Visual rendering matches or exceeds desktop quality
   - No visual artifacts (broken bonds, incorrect colors, clipping issues)
   - Professional-grade UI polish (no jank, smooth animations)

4. **Testing**
   - 100% Playwright test coverage for critical paths
   - Visual regression tests pass (no unintended UI changes)
   - Cross-browser compatibility confirmed (Chrome, Firefox, Safari)

5. **Documentation**
   - User documentation complete (how to use web app)
   - Developer documentation complete (how to build, deploy, contribute)
   - Migration guide for Tauri desktop users

6. **Deployment**
   - Production URL live and stable
   - CI/CD pipeline automated (git push → deploy in <5 min)
   - Monitoring and error tracking configured (Sentry or similar)

7. **Headless Unification**
   - CLI, web, and HPC rendering use same codebase
   - Zero duplication between rendering paths
   - Consistent output across all platforms

---

## Implementation Timeline (6-8 Weeks)

**Quality over speed** — timeline is flexible, gates must pass before moving forward.

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| **Phase 1:** wgpu-WASM Foundation | 1-2 weeks | WASM bundle loads, renders sphere at 60 FPS |
| **Phase 2:** CIF Parsing & Rendering | 1-2 weeks | Water, methane, ACACEP render correctly |
| **Phase 3:** Feature Parity | 2-3 weeks | All desktop features work in web |
| **Phase 4:** Playwright Testing | 1 week | Full test suite passes in CI |
| **Phase 5:** Production Deployment | 1 week | Staging URL live, tested with real files |
| **Phase 6:** Headless Unification | 1 week | CLI uses same renderer, zero duplication |
| **Total:** | **6-8 weeks** | Production-ready web app + unified codebase |

**No hard deadlines** — if a phase takes longer to meet quality gates, that's acceptable.

---

## Risk Mitigation

### Risk: WASM bundle size too large (>10 MB)
**Mitigation:**
- Use `wasm-opt` for aggressive optimization
- Lazy-load non-critical WASM modules
- Profile bundle size at Phase 1 gate, iterate if needed

### Risk: WebGPU browser support issues
**Mitigation:**
- Test on Chrome, Firefox, Safari early (Phase 1)
- Document minimum browser versions required
- Consider WebGL fallback if critical users can't upgrade (low priority)

### Risk: WASM debugging complexity slows development
**Mitigation:**
- Set up source maps and debugging tools early
- Maintain native CLI version for easier debugging
- Use browser DevTools WASM debugging features

### Risk: Performance doesn't meet 60 FPS target
**Mitigation:**
- Profile GPU utilization at Phase 2
- Optimize shaders, reduce draw calls if needed
- Implement LOD (level of detail) for large structures
- Leverage existing octree/frustum culling from `renderer.rs`

### Risk: Playwright tests flaky or hard to maintain
**Mitigation:**
- Use visual regression testing (screenshot diffs) for stability
- Implement retry logic for network-dependent tests
- Keep test suite focused on critical paths (not exhaustive)

---

## Why We're NOT Doing a POC

**Typical approach:** Build Three.js POC (2-3 days) → validate web works → commit to wgpu-WASM

**Why we're skipping it:**

1. **We already KNOW web works** — every modern molecular viewer uses it (3Dmol.js, Mol*, NGL Viewer)
2. **POC validation = unnecessary** when the answer is obvious
3. **POC creates tech debt** — throwaway code that tempts "let's just ship this"
4. **Quality over speed directive** — build it RIGHT from day one, not fast then redo

**Decision: Go straight to wgpu-WASM production implementation.**

---

## Deprecation Plan for Tauri Desktop

**Once web version reaches production (Phase 5 complete):**

1. Mark Tauri repo as `[LEGACY]` in README
2. Add banner: "Axiom has moved to web — visit [axiom.heinz-lab.com]"
3. Archive Tauri desktop releases (keep for reference, no new versions)
4. Redirect all new users to web version
5. Maintain Tauri desktop for 3-6 months (security fixes only)
6. After 6 months: Fully deprecate, direct all users to web

**No hybrid approach** — web is the future, desktop is legacy.

---

## Alignment with Lab Principles

### ✅ Quality Over Speed
- 6-8 weeks for robust solution vs 2-3 days for throwaway POC
- No shortcuts, build final architecture from day one
- Comprehensive testing (Playwright, visual regression, cross-browser)

### ✅ Minimize Tech Debt
- Reuse existing Rust renderer (no JS rewrite)
- Unified codebase (web/CLI/headless)
- No future rewrites needed

### ✅ Agent-Native, Headless-First
- Same renderer for web UI and CLI screenshot generation
- HTTP API ready for LLM integration
- Enables future HPC batch rendering workflows

### ✅ Research Impact
- Faster iteration (40x deployment speed)
- Easier collaboration (URL sharing vs installer distribution)
- Better testing (Playwright mature on web)

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| Mar 2, 2026 | Pivot to web architecture | Aligns with "online native" vision, enables Playwright testing, 40x faster iteration |
| Mar 2, 2026 | Use wgpu-WASM (not Three.js) | Reuses existing Rust code, zero tech debt, future-proof, highest quality |
| Mar 2, 2026 | Skip POC phase | We know web works, build final architecture from day one (quality over speed) |
| Mar 2, 2026 | 6-8 week timeline acceptable | Quality gates more important than speed, no hard deadlines |
| Mar 2, 2026 | Deprecate Tauri desktop after web v1 | Web is the future, no hybrid maintenance burden |

---

## Execution

**Owner:** axiom-agent
**Start Date:** March 2, 2026
**Target Completion:** April 13-27, 2026 (flexible based on quality gates)
**Status Reporting:** Weekly updates to Sean F via #lab-agent Slack channel

**Next Steps:**
1. axiom-agent reads comprehensive technical implementation plan (separate document)
2. axiom-agent begins Phase 1: Extract `renderer.rs` into `axiom-renderer` crate with WASM support
3. Daily progress updates in axiom project STATE.md
4. Weekly summary reports to Sean F

---

**This is the final decision. Implementation begins now.**

---

## References

- [wgpu Documentation](https://wgpu.rs/)
- [WebGPU Specification](https://www.w3.org/TR/webgpu/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [Playwright Documentation](https://playwright.dev/)
- [Axiom ARCHITECTURAL_ANALYSIS_WEB_VS_DESKTOP.md](./ARCHITECTURAL_ANALYSIS_WEB_VS_DESKTOP.md) (739 lines, Mar 2, 2026)
