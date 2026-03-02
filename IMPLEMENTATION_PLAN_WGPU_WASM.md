# Comprehensive Technical Implementation Plan: Axiom wgpu-WASM Web Architecture

**Version:** 1.0
**Date:** March 2, 2026
**Owner:** axiom-agent
**Status:** APPROVED — Implementation Starting Immediately

---

## Table of Contents

1. [Overview](#overview)
2. [Phase 1: wgpu-WASM Foundation](#phase-1-wgpu-wasm-foundation)
3. [Phase 2: CIF Parsing & Rendering](#phase-2-cif-parsing--rendering)
4. [Phase 3: Feature Parity](#phase-3-feature-parity)
5. [Phase 4: Playwright Testing Infrastructure](#phase-4-playwright-testing-infrastructure)
6. [Phase 5: Production Deployment](#phase-5-production-deployment)
7. [Phase 6: Headless Unification](#phase-6-headless-unification)
8. [Testing Strategy](#testing-strategy)
9. [Validation Gates](#validation-gates)
10. [Troubleshooting Guide](#troubleshooting-guide)

---

## Overview

**Objective:** Build production-grade web-based molecular structure viewer using Rust wgpu renderer compiled to WASM, achieving 60 FPS interactive rendering with comprehensive Playwright testing.

**Architecture:**
- **Frontend:** React (Vite + TypeScript) + axiom-renderer.wasm
- **Rendering:** wgpu-WASM → WebGPU (browser GPU)
- **Parsing:** Rust CIF parser (in WASM or backend API)
- **Deployment:** Vercel or Cloudflare Pages
- **Testing:** Playwright (visual regression + functional + cross-browser)

**Timeline:** 6-8 weeks (quality gates dictate pace, not calendar)

---

## Phase 1: wgpu-WASM Foundation

**Goal:** Extract `renderer.rs` into reusable `axiom-renderer` crate with WASM support, render single sphere at 60 FPS in browser.

### Step 1.1: Create axiom-renderer Crate

**Implementation:**

```bash
cd ~/projects/axiom/axiom-gui/
cargo new --lib axiom-renderer
cd axiom-renderer
```

**Edit `Cargo.toml`:**

```toml
[package]
name = "axiom-renderer"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]  # cdylib for WASM, rlib for native

[dependencies]
wgpu = "0.19"           # Supports WebGPU via wgpu-web backend
bytemuck = "1.14"        # For casting structs to byte slices
glam = "0.25"            # Math library (vectors, matrices)

# WASM-specific dependencies (only when targeting wasm32)
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = [
    "Window",
    "Document",
    "HtmlCanvasElement",
    "WebGl2RenderingContext",
] }
console_error_panic_hook = "0.1"  # Better panic messages in browser console

[dev-dependencies]
pollster = "0.3"  # For async executor in tests
```

**Validation:**
```bash
cargo build --target wasm32-unknown-unknown
# Should compile without errors
```

**Quality Gate:**
- [ ] `axiom-renderer` crate created
- [ ] Compiles for `wasm32-unknown-unknown` target
- [ ] Compiles for native target (x86_64-unknown-linux-gnu)

---

### Step 1.2: Extract Rendering Logic from renderer.rs

**Implementation:**

1. **Identify core rendering components in existing `renderer.rs`:**
   - GPU context initialization (wgpu::Instance, Adapter, Device, Queue)
   - Render pipeline setup (shaders, vertex buffers, bind groups)
   - Camera matrix calculations
   - Atom sphere rendering logic
   - Bond rendering logic

2. **Create modular architecture in `axiom-renderer/src/`:**

```
axiom-renderer/src/
├── lib.rs              # Public API, re-exports
├── context.rs          # GPU context (Device, Queue, Surface)
├── camera.rs           # Camera matrices, controls
├── pipeline.rs         # Render pipeline, shaders
├── geometry.rs         # Sphere/cylinder meshes for atoms/bonds
├── renderer.rs         # Main renderer struct
├── wasm.rs             # WASM bindings (target = wasm32 only)
└── shaders/
    ├── atom.wgsl       # Atom sphere shader
    └── bond.wgsl       # Bond cylinder shader
```

3. **Implement `lib.rs` (public API):**

```rust
// axiom-renderer/src/lib.rs

mod context;
mod camera;
mod pipeline;
mod geometry;
mod renderer;

#[cfg(target_arch = "wasm32")]
mod wasm;

pub use renderer::Renderer;
pub use camera::Camera;

// Re-export for external use
pub struct AtomData {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub radius: f32,
}

pub struct BondData {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub color: [f32; 3],
    pub radius: f32,
}
```

4. **Implement `context.rs` (GPU initialization with native/WASM support):**

```rust
// axiom-renderer/src/context.rs

use wgpu;

pub struct RenderContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: Option<wgpu::Surface>,  // None for headless
    pub surface_config: Option<wgpu::SurfaceConfiguration>,
}

impl RenderContext {
    #[cfg(target_arch = "wasm32")]
    pub async fn new_for_canvas(canvas: web_sys::HtmlCanvasElement) -> Result<Self, String> {
        // WebGPU backend initialization
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });

        let surface = instance.create_surface_from_canvas(canvas)
            .map_err(|e| format!("Failed to create surface: {:?}", e))?;

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .ok_or("Failed to find GPU adapter")?;

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("axiom-renderer-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
        }, None)
        .await
        .map_err(|e| format!("Failed to create device: {:?}", e))?;

        let size = (canvas.width(), canvas.height());
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: size.0,
            height: size.1,
            present_mode: wgpu::PresentMode::Fifo,  // VSync
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        Ok(Self {
            device,
            queue,
            surface: Some(surface),
            surface_config: Some(surface_config),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn new_headless() -> Result<Self, String> {
        // Native headless backend (for CLI screenshot generation)
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .ok_or("Failed to find GPU adapter")?;

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("axiom-renderer-device-headless"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        }, None)
        .await
        .map_err(|e| format!("Failed to create device: {:?}", e))?;

        Ok(Self {
            device,
            queue,
            surface: None,
            surface_config: None,
        })
    }
}
```

5. **Implement `camera.rs` (camera matrices):**

```rust
// axiom-renderer/src/camera.rs

use glam::{Mat4, Vec3};

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fovy: f32,       // Field of view (radians)
    pub aspect: f32,     // Aspect ratio (width/height)
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            eye: Vec3::new(0.0, 0.0, 10.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fovy: 45.0_f32.to_radians(),
            aspect: width as f32 / height as f32,
            near: 0.1,
            far: 100.0,
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye, self.target, self.up)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy, self.aspect, self.near, self.far)
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    pub fn rotate(&mut self, delta_x: f32, delta_y: f32) {
        // Rotate camera around target (trackball-style)
        // Implementation: convert eye to spherical coords, rotate, convert back
        // (Full implementation omitted for brevity — reuse from existing renderer.rs)
    }

    pub fn zoom(&mut self, delta: f32) {
        let direction = (self.eye - self.target).normalize();
        self.eye += direction * delta;
    }
}
```

6. **Implement `geometry.rs` (sphere/cylinder meshes):**

```rust
// axiom-renderer/src/geometry.rs

pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

/// Generate UV sphere mesh (used for atoms)
pub fn generate_sphere(segments: u32, rings: u32) -> (Vec<Vertex>, Vec<u16>) {
    // Implementation: standard UV sphere generation
    // Returns (vertices, indices)
    // (Full implementation omitted — reuse from existing renderer.rs or use icosphere)
    todo!()
}

/// Generate cylinder mesh (used for bonds)
pub fn generate_cylinder(segments: u32) -> (Vec<Vertex>, Vec<u16>) {
    // Implementation: standard cylinder generation
    // (Full implementation omitted)
    todo!()
}
```

7. **Implement `pipeline.rs` (render pipeline with shaders):**

```rust
// axiom-renderer/src/pipeline.rs

use wgpu;

pub struct RenderPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl RenderPipeline {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let shader_source = include_str!("shaders/atom.wgsl");
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("atom-shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera-bind-group-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render-pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("atom-render-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[/* vertex buffer layout */],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }
}
```

8. **Create `shaders/atom.wgsl` (basic atom rendering shader):**

```wgsl
// axiom-renderer/src/shaders/atom.wgsl

struct CameraUniforms {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.world_normal = in.normal;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple diffuse lighting
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(in.world_normal, light_dir), 0.3);  // Ambient = 0.3
    return vec4<f32>(vec3<f32>(0.8, 0.2, 0.2) * diffuse, 1.0);  // Red sphere
}
```

9. **Implement `renderer.rs` (main renderer struct):**

```rust
// axiom-renderer/src/renderer.rs

use crate::{context::RenderContext, camera::Camera, pipeline::RenderPipeline, AtomData};

pub struct Renderer {
    context: RenderContext,
    camera: Camera,
    pipeline: RenderPipeline,
    // atom_buffer, bond_buffer, etc.
}

impl Renderer {
    #[cfg(target_arch = "wasm32")]
    pub async fn new(canvas: web_sys::HtmlCanvasElement) -> Result<Self, String> {
        let context = RenderContext::new_for_canvas(canvas).await?;
        let camera = Camera::new(
            context.surface_config.as_ref().unwrap().width,
            context.surface_config.as_ref().unwrap().height,
        );
        let pipeline = RenderPipeline::new(
            &context.device,
            context.surface_config.as_ref().unwrap().format,
        );

        Ok(Self {
            context,
            camera,
            pipeline,
        })
    }

    pub fn render(&mut self, atoms: &[AtomData]) -> Result<(), String> {
        // 1. Get current surface texture
        // 2. Create render pass
        // 3. Bind camera uniform buffer
        // 4. Render atoms (instances or individual draws)
        // 5. Submit command buffer
        // 6. Present surface

        // (Full implementation omitted for brevity)
        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(ref mut config) = self.context.surface_config {
            config.width = width;
            config.height = height;
            if let Some(ref surface) = self.context.surface {
                surface.configure(&self.context.device, config);
            }
        }
        self.camera.aspect = width as f32 / height as f32;
    }

    pub fn rotate_camera(&mut self, delta_x: f32, delta_y: f32) {
        self.camera.rotate(delta_x, delta_y);
    }

    pub fn zoom_camera(&mut self, delta: f32) {
        self.camera.zoom(delta);
    }
}
```

10. **Implement `wasm.rs` (WASM bindings for JavaScript):**

```rust
// axiom-renderer/src/wasm.rs

use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use crate::{Renderer, AtomData};

#[wasm_bindgen]
pub struct WasmRenderer {
    inner: Option<Renderer>,
}

#[wasm_bindgen]
impl WasmRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Set panic hook for better error messages
        console_error_panic_hook::set_once();
        Self { inner: None }
    }

    #[wasm_bindgen]
    pub async fn initialize(&mut self, canvas: HtmlCanvasElement) -> Result<(), JsValue> {
        let renderer = Renderer::new(canvas).await
            .map_err(|e| JsValue::from_str(&e))?;
        self.inner = Some(renderer);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn render(&mut self, atoms_js: JsValue) -> Result<(), JsValue> {
        let renderer = self.inner.as_mut()
            .ok_or_else(|| JsValue::from_str("Renderer not initialized"))?;

        // Deserialize atoms from JavaScript
        let atoms: Vec<AtomData> = serde_wasm_bindgen::from_value(atoms_js)?;

        renderer.render(&atoms)
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(ref mut renderer) = self.inner {
            renderer.resize(width, height);
        }
    }

    #[wasm_bindgen]
    pub fn rotate(&mut self, delta_x: f32, delta_y: f32) {
        if let Some(ref mut renderer) = self.inner {
            renderer.rotate_camera(delta_x, delta_y);
        }
    }

    #[wasm_bindgen]
    pub fn zoom(&mut self, delta: f32) {
        if let Some(ref mut renderer) = self.inner {
            renderer.zoom_camera(delta);
        }
    }
}
```

**Validation:**

```bash
# Build WASM
cd ~/projects/axiom/axiom-gui/axiom-renderer
wasm-pack build --target web --out-dir pkg

# Check output
ls pkg/  # Should see: axiom_renderer.js, axiom_renderer_bg.wasm, etc.

# Check bundle size
du -h pkg/axiom_renderer_bg.wasm  # Should be <5 MB
```

**Quality Gate:**
- [ ] `axiom-renderer` crate structure created (lib.rs, context.rs, camera.rs, pipeline.rs, geometry.rs, renderer.rs, wasm.rs)
- [ ] Shaders created (atom.wgsl)
- [ ] Compiles to WASM without errors (`wasm-pack build --target web`)
- [ ] WASM bundle size ≤ 5 MB
- [ ] pkg/ directory contains .wasm and .js files

---

### Step 1.3: Create Minimal React Frontend

**Implementation:**

```bash
cd ~/projects/axiom/axiom-gui/
npm create vite@latest axiom-web -- --template react-ts
cd axiom-web
npm install
```

**Install WASM loader:**

```bash
npm install vite-plugin-wasm vite-plugin-top-level-await
```

**Edit `vite.config.ts`:**

```typescript
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import wasm from 'vite-plugin-wasm'
import topLevelAwait from 'vite-plugin-top-level-await'

export default defineConfig({
  plugins: [react(), wasm(), topLevelAwait()],
})
```

**Create `src/wasm/axiom-renderer.ts` (TypeScript wrapper):**

```typescript
// src/wasm/axiom-renderer.ts

import init, { WasmRenderer } from '../../../axiom-renderer/pkg/axiom_renderer'

let wasmInitialized = false

export async function initWasm() {
  if (!wasmInitialized) {
    await init()
    wasmInitialized = true
  }
}

export { WasmRenderer }
```

**Create `src/components/MoleculeViewer.tsx` (React component):**

```typescript
// src/components/MoleculeViewer.tsx

import React, { useEffect, useRef, useState } from 'react'
import { initWasm, WasmRenderer } from '../wasm/axiom-renderer'

export const MoleculeViewer: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const rendererRef = useRef<WasmRenderer | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const initialize = async () => {
      try {
        // Initialize WASM module
        await initWasm()

        // Create renderer
        const renderer = new WasmRenderer()
        await renderer.initialize(canvasRef.current!)

        // Test render: single red sphere at origin
        const testAtom = [{
          position: [0.0, 0.0, 0.0],
          color: [1.0, 0.0, 0.0],  // Red
          radius: 1.0,
        }]

        renderer.render(testAtom)

        rendererRef.current = renderer
        setLoading(false)

        // Animation loop
        const animate = () => {
          renderer.render(testAtom)
          requestAnimationFrame(animate)
        }
        animate()

      } catch (err) {
        console.error('Failed to initialize renderer:', err)
        setError(err instanceof Error ? err.message : String(err))
        setLoading(false)
      }
    }

    initialize()

    return () => {
      // Cleanup
      rendererRef.current = null
    }
  }, [])

  const handleMouseMove = (e: React.MouseEvent) => {
    if (e.buttons === 1 && rendererRef.current) {  // Left mouse button
      const deltaX = e.movementX * 0.01
      const deltaY = e.movementY * 0.01
      rendererRef.current.rotate(deltaX, deltaY)
    }
  }

  const handleWheel = (e: React.WheelEvent) => {
    if (rendererRef.current) {
      const delta = e.deltaY * 0.01
      rendererRef.current.zoom(delta)
    }
  }

  return (
    <div style={{ width: '100vw', height: '100vh', display: 'flex', flexDirection: 'column' }}>
      {loading && <div>Loading WASM renderer...</div>}
      {error && <div style={{ color: 'red' }}>Error: {error}</div>}
      <canvas
        ref={canvasRef}
        width={800}
        height={600}
        style={{ border: '1px solid black', cursor: loading ? 'wait' : 'grab' }}
        onMouseMove={handleMouseMove}
        onWheel={handleWheel}
      />
    </div>
  )
}
```

**Edit `src/App.tsx`:**

```typescript
import { MoleculeViewer } from './components/MoleculeViewer'

function App() {
  return <MoleculeViewer />
}

export default App
```

**Validation:**

```bash
cd ~/projects/axiom/axiom-gui/axiom-web
npm run dev
# Open browser to http://localhost:5173
# Should see canvas with red sphere
# Drag mouse → sphere should rotate
# Scroll wheel → camera should zoom in/out
```

**Quality Gate:**
- [ ] React app created with Vite + TypeScript
- [ ] WASM loader configured (vite-plugin-wasm, vite-plugin-top-level-await)
- [ ] axiom-renderer.wasm loads in browser without errors (check browser console)
- [ ] Canvas displays red sphere
- [ ] Mouse drag rotates camera (visual confirmation)
- [ ] Mouse wheel zooms camera (visual confirmation)
- [ ] 60 FPS maintained (check browser DevTools Performance tab)

---

### Step 1.4: Optimize WASM Bundle Size

**Implementation:**

```bash
cd ~/projects/axiom/axiom-gui/axiom-renderer

# Install wasm-opt (part of Binaryen)
# On Ubuntu:
sudo apt install binaryen

# Build with optimizations
wasm-pack build --target web --release --out-dir pkg

# Further optimize with wasm-opt
wasm-opt -Oz pkg/axiom_renderer_bg.wasm -o pkg/axiom_renderer_bg.wasm

# Check size
du -h pkg/axiom_renderer_bg.wasm
```

**Add to `Cargo.toml` for release optimizations:**

```toml
[profile.release]
opt-level = 'z'       # Optimize for size
lto = true            # Link-time optimization
codegen-units = 1     # Better optimization, slower compile
panic = 'abort'       # Smaller binary
strip = true          # Remove debug symbols
```

**Validation:**

```bash
# Bundle size should be <5 MB after optimization
du -h pkg/axiom_renderer_bg.wasm

# If still too large, profile with twiggy:
cargo install twiggy
twiggy top -n 20 pkg/axiom_renderer_bg.wasm
# Identify large dependencies, consider removing or lazy-loading
```

**Quality Gate:**
- [ ] WASM bundle size ≤ 5 MB (optimized with `wasm-opt -Oz`)
- [ ] Release build configured with `opt-level = 'z'`, `lto = true`
- [ ] No unused dependencies in Cargo.toml

---

### Step 1.5: Source Maps for Debugging

**Implementation:**

```bash
# Build with source maps
cd ~/projects/axiom/axiom-gui/axiom-renderer
RUSTFLAGS='-C debuginfo=2' wasm-pack build --target web --dev --out-dir pkg-dev
```

**Configure browser DevTools to load source maps:**

1. Open Chrome DevTools → Sources tab
2. Enable "Enable JavaScript source maps" in settings
3. WASM source should appear under `wasm://` protocol

**Validation:**

1. Trigger an error in Rust code (e.g., `panic!("Test panic")`)
2. Check browser console — should see Rust stack trace with file names and line numbers
3. Click on stack trace — should open Rust source code in DevTools

**Quality Gate:**
- [ ] Source maps generated (`pkg-dev/` contains .wasm with debuginfo)
- [ ] Browser DevTools displays Rust source code on error
- [ ] Stack traces show Rust function names and line numbers

---

### Phase 1 Final Validation

**Comprehensive Test:**

1. **Build and run:**
   ```bash
   cd ~/projects/axiom/axiom-gui/axiom-renderer
   wasm-pack build --target web --release --out-dir pkg
   wasm-opt -Oz pkg/axiom_renderer_bg.wasm -o pkg/axiom_renderer_bg.wasm

   cd ~/projects/axiom/axiom-gui/axiom-web
   npm run dev
   ```

2. **Open browser:** http://localhost:5173

3. **Visual validation:**
   - [ ] Red sphere visible on screen
   - [ ] Sphere centered in canvas
   - [ ] No console errors (check DevTools Console tab)

4. **Interaction validation:**
   - [ ] Drag mouse → sphere rotates smoothly
   - [ ] Scroll wheel → camera zooms in/out smoothly
   - [ ] No stuttering or lag

5. **Performance validation:**
   - [ ] Open DevTools → Performance tab → Record 10 seconds of interaction
   - [ ] Check FPS counter → should be 60 FPS consistently
   - [ ] GPU utilization visible (Chrome → More Tools → Task Manager → GPU Process)

6. **Cross-browser validation:**
   - [ ] Test in Chrome (WebGPU support: ✓)
   - [ ] Test in Firefox (WebGPU support: ✓)
   - [ ] Test in Safari (WebGPU support: ✓ since Safari 17+)

**Quality Gates (Must ALL Pass):**

- [ ] `axiom-renderer` compiles for wasm32-unknown-unknown
- [ ] WASM bundle loads in browser without errors
- [ ] WebGPU context initializes successfully
- [ ] Single sphere renders at 60 FPS
- [ ] Mouse drag rotates camera smoothly
- [ ] Mouse wheel zooms camera smoothly
- [ ] Bundle size ≤ 5 MB
- [ ] Source maps work (Rust stack traces visible in DevTools)
- [ ] Cross-browser compatibility (Chrome, Firefox, Safari)

**If ANY gate fails:** Debug and fix before proceeding to Phase 2.

**Success Criteria:** All gates pass → Phase 1 COMPLETE.

---

## Phase 2: CIF Parsing & Rendering

**Goal:** Load real CIF files (water, methane, ACACEP), parse atom coordinates, render with correct positions and colors.

### Step 2.1: CIF Parser Integration (WASM or Backend)

**Decision:** Parse CIF in WASM (no backend needed for MVP)

**Implementation:**

1. **Add dependency to `axiom-renderer/Cargo.toml`:**

```toml
[dependencies]
# ... existing dependencies ...
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"  # For JS ↔ Rust serialization

# Assuming axiom-core has CIF parser:
axiom-core = { path = "../../axiom-core" }  # Adjust path as needed
```

2. **Update `wasm.rs` to accept CIF string:**

```rust
// axiom-renderer/src/wasm.rs

use axiom_core::cif::parse_cif;  // Assuming axiom-core exports this

#[wasm_bindgen]
impl WasmRenderer {
    // ... existing methods ...

    #[wasm_bindgen]
    pub fn load_cif(&mut self, cif_content: &str) -> Result<JsValue, JsValue> {
        // Parse CIF
        let structure = parse_cif(cif_content)
            .map_err(|e| JsValue::from_str(&format!("CIF parse error: {:?}", e)))?;

        // Convert to AtomData
        let atoms: Vec<AtomData> = structure.atoms.iter().map(|atom| {
            AtomData {
                position: atom.position,
                color: element_color(&atom.element),  // Helper function
                radius: element_radius(&atom.element),
            }
        }).collect();

        // Render
        if let Some(ref mut renderer) = self.inner {
            renderer.render(&atoms)
                .map_err(|e| JsValue::from_str(&e))?;
        }

        // Return atom count for confirmation
        Ok(JsValue::from_f64(atoms.len() as f64))
    }
}

fn element_color(element: &str) -> [f32; 3] {
    match element {
        "H" => [1.0, 1.0, 1.0],  // White
        "C" => [0.2, 0.2, 0.2],  // Dark gray
        "N" => [0.0, 0.0, 1.0],  // Blue
        "O" => [1.0, 0.0, 0.0],  // Red
        "Br" => [0.6, 0.1, 0.1], // Dark red
        "Pb" => [0.3, 0.3, 0.3], // Gray
        "I" => [0.5, 0.0, 0.5],  // Purple
        _ => [0.5, 0.5, 0.5],    // Default gray
    }
}

fn element_radius(element: &str) -> f32 {
    match element {
        "H" => 0.3,
        "C" => 0.7,
        "N" => 0.65,
        "O" => 0.6,
        "Br" => 1.0,
        "Pb" => 1.2,
        "I" => 1.1,
        _ => 0.5,
    }
}
```

3. **Update React frontend to load CIF files:**

```typescript
// src/components/MoleculeViewer.tsx

const MoleculeViewer: React.FC = () => {
  // ... existing state ...

  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file || !rendererRef.current) return

    try {
      const cifContent = await file.text()
      const atomCount = rendererRef.current.load_cif(cifContent)
      console.log(`Loaded ${atomCount} atoms from ${file.name}`)
    } catch (err) {
      console.error('Failed to load CIF:', err)
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  return (
    <div>
      <input type="file" accept=".cif" onChange={handleFileUpload} />
      {/* ... canvas ... */}
    </div>
  )
}
```

**Validation:**

```bash
# Rebuild WASM
cd ~/projects/axiom/axiom-gui/axiom-renderer
wasm-pack build --target web --release --out-dir pkg

# Test in browser
cd ~/projects/axiom/axiom-gui/axiom-web
npm run dev

# Open browser, upload water.cif (2 H + 1 O)
# Should see 3 spheres: 2 white (H), 1 red (O)
# Check console for "Loaded 3 atoms from water.cif"
```

**Quality Gate:**
- [ ] CIF parser integrated into WASM
- [ ] File upload input works
- [ ] Water molecule (H2O.cif) renders correctly: 2 white + 1 red sphere
- [ ] Atom count logged to console matches CIF file
- [ ] No errors in browser console

---

### Step 2.2: Bond Rendering

**Implementation:**

1. **Add bond detection to CIF parser (or compute in Rust):**

```rust
// In axiom-core or axiom-renderer

pub fn detect_bonds(atoms: &[Atom]) -> Vec<(usize, usize)> {
    // Simple distance-based bond detection
    let mut bonds = Vec::new();
    for i in 0..atoms.len() {
        for j in (i+1)..atoms.len() {
            let dist = distance(&atoms[i].position, &atoms[j].position);
            let max_bond_dist = atoms[i].covalent_radius() + atoms[j].covalent_radius() + 0.4;
            if dist < max_bond_dist {
                bonds.push((i, j));
            }
        }
    }
    bonds
}
```

2. **Update renderer to accept bonds:**

```rust
// axiom-renderer/src/wasm.rs

#[wasm_bindgen]
impl WasmRenderer {
    #[wasm_bindgen]
    pub fn load_cif(&mut self, cif_content: &str) -> Result<JsValue, JsValue> {
        let structure = parse_cif(cif_content)
            .map_err(|e| JsValue::from_str(&format!("CIF parse error: {:?}", e)))?;

        let atoms: Vec<AtomData> = /* ... */;
        let bonds: Vec<BondData> = detect_bonds(&structure.atoms)
            .iter()
            .map(|(i, j)| BondData {
                start: structure.atoms[*i].position,
                end: structure.atoms[*j].position,
                color: [0.5, 0.5, 0.5],  // Gray bonds
                radius: 0.1,
            })
            .collect();

        if let Some(ref mut renderer) = self.inner {
            renderer.render_with_bonds(&atoms, &bonds)
                .map_err(|e| JsValue::from_str(&e))?;
        }

        Ok(JsValue::from_f64(atoms.len() as f64))
    }
}
```

3. **Implement `render_with_bonds` in `renderer.rs`:**

```rust
// axiom-renderer/src/renderer.rs

impl Renderer {
    pub fn render_with_bonds(&mut self, atoms: &[AtomData], bonds: &[BondData]) -> Result<(), String> {
        // 1. Render bonds first (behind atoms)
        self.render_bonds(bonds)?;

        // 2. Render atoms on top
        self.render_atoms(atoms)?;

        Ok(())
    }

    fn render_bonds(&mut self, bonds: &[BondData]) -> Result<(), String> {
        // Use cylinder geometry
        // For each bond: position, scale, rotate cylinder to connect start→end
        // (Implementation similar to atom rendering but with instanced cylinders)
        todo!()
    }

    fn render_atoms(&mut self, atoms: &[AtomData]) -> Result<(), String> {
        // Existing sphere rendering
        todo!()
    }
}
```

**Validation:**

```bash
# Test with water.cif (should see 2 H-O bonds)
# Test with methane.cif (should see 4 C-H bonds)
# Test with ACACEP.cif (complex structure with many bonds)
```

**Quality Gate:**
- [ ] Water molecule renders with 2 H-O bonds
- [ ] Methane molecule renders with 4 C-H bonds
- [ ] Bonds connect atom centers (no floating/disconnected bonds)
- [ ] Bonds render behind atoms (depth test works)

---

### Step 2.3: Camera Auto-Framing

**Implementation:**

```rust
// axiom-renderer/src/camera.rs

impl Camera {
    pub fn frame_atoms(&mut self, atoms: &[AtomData]) {
        if atoms.is_empty() { return; }

        // Compute bounding box
        let mut min = atoms[0].position;
        let mut max = atoms[0].position;
        for atom in atoms {
            for i in 0..3 {
                min[i] = min[i].min(atom.position[i] - atom.radius);
                max[i] = max[i].max(atom.position[i] + atom.radius);
            }
        }

        // Compute center and size
        let center = [
            (min[0] + max[0]) / 2.0,
            (min[1] + max[1]) / 2.0,
            (min[2] + max[2]) / 2.0,
        ];
        let size = [
            max[0] - min[0],
            max[1] - min[1],
            max[2] - min[2],
        ];
        let max_size = size[0].max(size[1]).max(size[2]);

        // Position camera to fit structure
        self.target = Vec3::from_array(center);
        let distance = max_size / (2.0 * (self.fovy / 2.0).tan());
        self.eye = self.target + Vec3::new(0.0, 0.0, distance * 1.5);  // 1.5x for padding
    }
}
```

```rust
// axiom-renderer/src/wasm.rs

#[wasm_bindgen]
impl WasmRenderer {
    #[wasm_bindgen]
    pub fn load_cif(&mut self, cif_content: &str) -> Result<JsValue, JsValue> {
        // ... parse CIF, create atoms ...

        if let Some(ref mut renderer) = self.inner {
            renderer.camera.frame_atoms(&atoms);  // Auto-frame camera
            renderer.render_with_bonds(&atoms, &bonds)?;
        }

        Ok(JsValue::from_f64(atoms.len() as f64))
    }
}
```

**Validation:**

```bash
# Load water.cif → should auto-fit to screen
# Load ACACEP.cif (large structure) → should auto-fit to screen
# No manual zoom needed to see full structure
```

**Quality Gate:**
- [ ] Camera auto-frames loaded structure (full structure visible on load)
- [ ] Works for small (water) and large (ACACEP) structures
- [ ] Structure centered in viewport

---

### Phase 2 Final Validation

**Comprehensive Test Cases:**

1. **Water (H2O.cif):**
   - [ ] 3 atoms render: 2 white (H), 1 red (O)
   - [ ] 2 bonds render: H-O, H-O
   - [ ] Bonds connect atom centers
   - [ ] Structure auto-fits to screen
   - [ ] 60 FPS rotation

2. **Methane (CH4.cif):**
   - [ ] 5 atoms render: 1 dark gray (C), 4 white (H)
   - [ ] 4 bonds render: C-H × 4
   - [ ] Tetrahedral geometry visible
   - [ ] 60 FPS rotation

3. **ACACEP.cif (complex perovskite):**
   - [ ] All atoms render with correct colors (Br, Pb, I, C, N, H)
   - [ ] Bonds render correctly (no missing/extra bonds)
   - [ ] Structure auto-fits to screen
   - [ ] 60 FPS rotation (performance test)
   - [ ] No console errors
   - [ ] No visual artifacts (clipping, z-fighting)

**Cross-Browser Validation:**
- [ ] Test all 3 structures in Chrome
- [ ] Test all 3 structures in Firefox
- [ ] Test all 3 structures in Safari

**Quality Gates (Must ALL Pass):**

- [ ] CIF parser works in WASM
- [ ] Water, methane, ACACEP all render correctly
- [ ] Bonds connect atom centers (no floating bonds)
- [ ] Element names correct (check against periodic table)
- [ ] Colors match CPK coloring convention
- [ ] Camera auto-framing works
- [ ] 60 FPS maintained for all test cases
- [ ] No console errors
- [ ] Cross-browser compatibility confirmed

**If ANY gate fails:** Debug and fix before proceeding to Phase 3.

**Success Criteria:** All gates pass → Phase 2 COMPLETE.

---

## Phase 3: Feature Parity

**Goal:** Implement all features from Tauri desktop version in web version.

### Feature List (From Tauri Desktop):

1. Drag-and-drop CIF file upload
2. File picker dialog
3. Recent files list
4. Export PNG screenshot
5. Selection tools (click atom → highlight)
6. Info panel (show selected atom details)
7. Measurements (distance, angle tools)
8. Rendering settings UI (sphere/stick, colors, quality)
9. Camera controls (reset, orthographic/perspective)

### Step 3.1: Drag-and-Drop File Upload

**Implementation:**

```typescript
// src/components/MoleculeViewer.tsx

const MoleculeViewer: React.FC = () => {
  const [isDragging, setIsDragging] = useState(false)

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(true)
  }

  const handleDragLeave = () => {
    setIsDragging(false)
  }

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(false)

    const file = e.dataTransfer.files[0]
    if (!file || !file.name.endsWith('.cif')) {
      setError('Please drop a .cif file')
      return
    }

    try {
      const cifContent = await file.text()
      const atomCount = rendererRef.current?.load_cif(cifContent)
      console.log(`Loaded ${atomCount} atoms from ${file.name}`)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  return (
    <div
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      style={{
        width: '100vw',
        height: '100vh',
        border: isDragging ? '3px dashed blue' : 'none',
      }}
    >
      {isDragging && <div className="drop-overlay">Drop CIF file here</div>}
      {/* ... canvas ... */}
    </div>
  )
}
```

**Validation:**

```bash
# Open browser
# Drag water.cif from file manager → drop on canvas
# Should load and render water molecule
# Check console for "Loaded 3 atoms from water.cif"
```

**Quality Gate:**
- [ ] Drag-and-drop works (visual feedback on drag)
- [ ] Dropping .cif file loads and renders structure
- [ ] Dropping non-.cif file shows error message
- [ ] No page reload on drop

---

### Step 3.2: Recent Files List (localStorage)

**Implementation:**

```typescript
// src/hooks/useRecentFiles.ts

export function useRecentFiles() {
  const [recentFiles, setRecentFiles] = useState<string[]>([])

  useEffect(() => {
    const saved = localStorage.getItem('axiom-recent-files')
    if (saved) {
      setRecentFiles(JSON.parse(saved))
    }
  }, [])

  const addRecentFile = (filename: string) => {
    const updated = [filename, ...recentFiles.filter(f => f !== filename)].slice(0, 10)
    setRecentFiles(updated)
    localStorage.setItem('axiom-recent-files', JSON.stringify(updated))
  }

  return { recentFiles, addRecentFile }
}
```

```typescript
// src/components/MoleculeViewer.tsx

const MoleculeViewer: React.FC = () => {
  const { recentFiles, addRecentFile } = useRecentFiles()

  const handleFileUpload = async (file: File) => {
    // ... load CIF ...
    addRecentFile(file.name)
  }

  return (
    <div>
      <aside>
        <h3>Recent Files</h3>
        <ul>
          {recentFiles.map(name => (
            <li key={name}>{name}</li>
          ))}
        </ul>
      </aside>
      {/* ... canvas ... */}
    </div>
  )
}
```

**Validation:**

```bash
# Load water.cif → should appear in recent files list
# Load methane.cif → should appear at top of list
# Reload page → recent files should persist
# Load 11 files → oldest should be removed (max 10)
```

**Quality Gate:**
- [ ] Recent files list displays loaded files
- [ ] List persists across page reloads (localStorage works)
- [ ] Maximum 10 files in list
- [ ] Most recent file at top

---

### Step 3.3: Export PNG Screenshot

**Implementation:**

```typescript
// src/components/MoleculeViewer.tsx

const exportPng = () => {
  if (!canvasRef.current) return

  const dataUrl = canvasRef.current.toDataURL('image/png')
  const link = document.createElement('a')
  link.download = 'molecule-screenshot.png'
  link.href = dataUrl
  link.click()
}

return (
  <div>
    <button onClick={exportPng}>Export PNG</button>
    {/* ... canvas ... */}
  </div>
)
```

**Validation:**

```bash
# Load water.cif
# Click "Export PNG" button
# PNG file should download
# Open PNG → should match canvas rendering
```

**Quality Gate:**
- [ ] Export PNG button works
- [ ] Downloaded PNG matches canvas rendering
- [ ] PNG has correct dimensions (canvas width × height)

---

### Step 3.4: Selection Tools (Click Atom → Highlight)

**Implementation:**

1. **Add raycasting to detect clicked atom:**

```rust
// axiom-renderer/src/wasm.rs

#[wasm_bindgen]
impl WasmRenderer {
    #[wasm_bindgen]
    pub fn pick_atom(&self, screen_x: f32, screen_y: f32) -> Option<usize> {
        // Convert screen coords to world ray
        // Test ray against all atom bounding spheres
        // Return index of closest intersected atom
        // (Full implementation requires inverse projection, ray-sphere intersection)
        None  // Placeholder
    }
}
```

2. **Add click handler in React:**

```typescript
// src/components/MoleculeViewer.tsx

const [selectedAtomIndex, setSelectedAtomIndex] = useState<number | null>(null)

const handleCanvasClick = (e: React.MouseEvent) => {
  if (!rendererRef.current || !canvasRef.current) return

  const rect = canvasRef.current.getBoundingClientRect()
  const x = e.clientX - rect.left
  const y = e.clientY - rect.top

  const atomIndex = rendererRef.current.pick_atom(x, y)
  setSelectedAtomIndex(atomIndex ?? null)
}

return (
  <div>
    {selectedAtomIndex !== null && (
      <div className="info-panel">
        <h3>Selected Atom #{selectedAtomIndex}</h3>
        {/* Show atom details: element, position, etc. */}
      </div>
    )}
    <canvas onClick={handleCanvasClick} />
  </div>
)
```

**Validation:**

```bash
# Load water.cif
# Click on oxygen atom → info panel should show "O" element
# Click on hydrogen atom → info panel should show "H" element
# Clicked atom should highlight (different color or outline)
```

**Quality Gate:**
- [ ] Click on atom selects it
- [ ] Info panel shows selected atom details (element, position)
- [ ] Visual feedback (highlight or outline) on selected atom
- [ ] Click on empty space deselects atom

---

(Remaining features: measurements, rendering settings UI, camera controls follow similar patterns. Implementation details omitted for brevity to stay within token limits.)

### Phase 3 Final Validation

**Comprehensive Feature Test:**

| Feature | Test Case | Pass/Fail |
|---------|-----------|-----------|
| Drag-and-drop | Drop water.cif → renders | [ ] |
| File picker | Click "Open" → select file → renders | [ ] |
| Recent files | Load 3 files → all appear in list → persist on reload | [ ] |
| Export PNG | Click "Export" → PNG downloads → matches canvas | [ ] |
| Selection | Click atom → highlights → info panel shows details | [ ] |
| Measurements | Click distance tool → click 2 atoms → shows distance | [ ] |
| Rendering settings | Toggle sphere/stick → visual changes | [ ] |
| Camera reset | Click "Reset" → camera returns to default view | [ ] |

**Quality Gates (Must ALL Pass):**

- [ ] All features from Tauri desktop implemented
- [ ] No regressions (existing features still work)
- [ ] UI polish (smooth animations, no jank)
- [ ] Accessibility (keyboard navigation, ARIA labels)

**Success Criteria:** All gates pass → Phase 3 COMPLETE.

---

## Phase 4: Playwright Testing Infrastructure

**Goal:** Comprehensive automated testing with visual regression, functional, cross-browser, performance, and accessibility tests.

### Step 4.1: Install Playwright

**Implementation:**

```bash
cd ~/projects/axiom/axiom-gui/axiom-web
npm install -D @playwright/test
npx playwright install  # Installs Chrome, Firefox, Safari browsers
```

**Create `playwright.config.ts`:**

```typescript
import { defineConfig, devices } from '@playwright/test'

export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:5173',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'firefox', use: { ...devices['Desktop Firefox'] } },
    { name: 'webkit', use: { ...devices['Desktop Safari'] } },
  ],
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
  },
})
```

**Validation:**

```bash
npx playwright test --list  # Should show 0 tests (none written yet)
```

**Quality Gate:**
- [ ] Playwright installed
- [ ] Config file created
- [ ] Browsers installed (Chrome, Firefox, Safari)

---

### Step 4.2: Visual Regression Tests

**Implementation:**

**Create `tests/visual-regression.spec.ts`:**

```typescript
import { test, expect } from '@playwright/test'
import path from 'path'

test.describe('Visual Regression Tests', () => {
  test('water molecule renders correctly', async ({ page }) => {
    await page.goto('/')

    // Upload water.cif
    const filePath = path.join(__dirname, '../test-data/water.cif')
    await page.setInputFiles('input[type="file"]', filePath)

    // Wait for rendering to complete
    await page.waitForTimeout(1000)

    // Take screenshot and compare to baseline
    const canvas = page.locator('canvas')
    await expect(canvas).toHaveScreenshot('water-molecule.png', {
      maxDiffPixels: 100,  // Allow small differences
    })
  })

  test('methane molecule renders correctly', async ({ page }) => {
    await page.goto('/')

    const filePath = path.join(__dirname, '../test-data/methane.cif')
    await page.setInputFiles('input[type="file"]', filePath)

    await page.waitForTimeout(1000)

    const canvas = page.locator('canvas')
    await expect(canvas).toHaveScreenshot('methane-molecule.png', {
      maxDiffPixels: 100,
    })
  })

  test('ACACEP perovskite renders correctly', async ({ page }) => {
    await page.goto('/')

    const filePath = path.join(__dirname, '../test-data/ACACEP.cif')
    await page.setInputFiles('input[type="file"]', filePath)

    await page.waitForTimeout(2000)  // Larger structure needs more time

    const canvas = page.locator('canvas')
    await expect(canvas).toHaveScreenshot('ACACEP-perovskite.png', {
      maxDiffPixels: 500,  // Larger structure, more tolerance
    })
  })
})
```

**Create baseline screenshots:**

```bash
# First run generates baseline screenshots
npx playwright test tests/visual-regression.spec.ts --update-snapshots

# Subsequent runs compare to baseline
npx playwright test tests/visual-regression.spec.ts
```

**Validation:**

```bash
# Run tests
npx playwright test tests/visual-regression.spec.ts

# Should PASS (no visual regressions)
# If changes detected, review diffs:
npx playwright show-report
```

**Quality Gate:**
- [ ] Visual regression tests written for water, methane, ACACEP
- [ ] Baseline screenshots generated
- [ ] Tests pass (no visual regressions)
- [ ] HTML report generated

---

### Step 4.3: Functional Tests

**Create `tests/functional.spec.ts`:**

```typescript
import { test, expect } from '@playwright/test'
import path from 'path'

test.describe('Functional Tests', () => {
  test('file upload works', async ({ page }) => {
    await page.goto('/')

    const filePath = path.join(__dirname, '../test-data/water.cif')
    await page.setInputFiles('input[type="file"]', filePath)

    await page.waitForTimeout(1000)

    // Check console for success message
    const logs: string[] = []
    page.on('console', msg => logs.push(msg.text()))

    expect(logs.some(log => log.includes('Loaded 3 atoms'))).toBeTruthy()
  })

  test('rotation with mouse drag works', async ({ page }) => {
    await page.goto('/')

    const filePath = path.join(__dirname, '../test-data/water.cif')
    await page.setInputFiles('input[type="file"]', filePath)
    await page.waitForTimeout(1000)

    const canvas = page.locator('canvas')

    // Take before screenshot
    const before = await canvas.screenshot()

    // Simulate mouse drag
    await canvas.hover()
    await page.mouse.down()
    await page.mouse.move(100, 100)  // Drag 100px right, 100px down
    await page.mouse.up()

    await page.waitForTimeout(500)

    // Take after screenshot
    const after = await canvas.screenshot()

    // Images should be different (rotation occurred)
    expect(before.equals(after)).toBe(false)
  })

  test('zoom with mouse wheel works', async ({ page }) => {
    await page.goto('/')

    const filePath = path.join(__dirname, '../test-data/water.cif')
    await page.setInputFiles('input[type="file"]', filePath)
    await page.waitForTimeout(1000)

    const canvas = page.locator('canvas')
    const before = await canvas.screenshot()

    // Simulate mouse wheel zoom
    await canvas.hover()
    await page.mouse.wheel(0, 100)  // Scroll down (zoom out)

    await page.waitForTimeout(500)

    const after = await canvas.screenshot()

    expect(before.equals(after)).toBe(false)
  })

  test('export PNG works', async ({ page }) => {
    await page.goto('/')

    const filePath = path.join(__dirname, '../test-data/water.cif')
    await page.setInputFiles('input[type="file"]', filePath)
    await page.waitForTimeout(1000)

    // Click export button
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.click('button:has-text("Export PNG")'),
    ])

    expect(download.suggestedFilename()).toContain('.png')
  })

  test('atom selection works', async ({ page }) => {
    await page.goto('/')

    const filePath = path.join(__dirname, '../test-data/water.cif')
    await page.setInputFiles('input[type="file"]', filePath)
    await page.waitForTimeout(1000)

    const canvas = page.locator('canvas')
    await canvas.click({ position: { x: 400, y: 300 } })  // Click center (likely O atom)

    // Info panel should appear
    await expect(page.locator('.info-panel')).toBeVisible()

    // Should show element name
    await expect(page.locator('.info-panel')).toContainText(/[OHC]/)
  })
})
```

**Validation:**

```bash
npx playwright test tests/functional.spec.ts
# All tests should PASS
```

**Quality Gate:**
- [ ] File upload test passes
- [ ] Rotation test passes (visual change detected)
- [ ] Zoom test passes (visual change detected)
- [ ] Export PNG test passes (download triggered)
- [ ] Atom selection test passes (info panel appears)

---

### Step 4.4: Cross-Browser Tests

**Already configured in `playwright.config.ts` via `projects`**

**Validation:**

```bash
# Run all tests across Chrome, Firefox, Safari
npx playwright test

# Check report for cross-browser results
npx playwright show-report
```

**Quality Gate:**
- [ ] Visual regression tests pass in Chrome
- [ ] Visual regression tests pass in Firefox
- [ ] Visual regression tests pass in Safari (WebKit)
- [ ] Functional tests pass in all 3 browsers

---

### Step 4.5: Performance Tests

**Create `tests/performance.spec.ts`:**

```typescript
import { test, expect } from '@playwright/test'
import path from 'path'

test.describe('Performance Tests', () => {
  test('maintains 60 FPS during rotation', async ({ page }) => {
    await page.goto('/')

    const filePath = path.join(__dirname, '../test-data/ACACEP.cif')
    await page.setInputFiles('input[type="file"]', filePath)
    await page.waitForTimeout(2000)

    // Start performance recording
    await page.evaluate(() => {
      (window as any).fpsLog = []
      let lastTime = performance.now()
      function measureFPS() {
        const now = performance.now()
        const fps = 1000 / (now - lastTime)
        ;(window as any).fpsLog.push(fps)
        lastTime = now
        if ((window as any).fpsLog.length < 60) {  // Record 60 frames
          requestAnimationFrame(measureFPS)
        }
      }
      requestAnimationFrame(measureFPS)
    })

    // Simulate rotation during measurement
    const canvas = page.locator('canvas')
    await canvas.hover()
    await page.mouse.down()
    for (let i = 0; i < 10; i++) {
      await page.mouse.move(i * 10, i * 10)
      await page.waitForTimeout(16)  // ~60 FPS
    }
    await page.mouse.up()

    // Wait for FPS measurement to complete
    await page.waitForTimeout(1000)

    // Check FPS
    const fpsLog = await page.evaluate(() => (window as any).fpsLog)
    const avgFPS = fpsLog.reduce((a: number, b: number) => a + b, 0) / fpsLog.length

    expect(avgFPS).toBeGreaterThan(55)  // Allow small margin (60 FPS target)
  })

  test('initial load time under 3 seconds', async ({ page }) => {
    const start = Date.now()

    await page.goto('/')

    // Wait for WASM to load and first render
    await page.waitForSelector('canvas')
    await page.waitForTimeout(500)  // Buffer for WASM init

    const loadTime = Date.now() - start

    expect(loadTime).toBeLessThan(3000)  // 3 second target
  })
})
```

**Validation:**

```bash
npx playwright test tests/performance.spec.ts
```

**Quality Gate:**
- [ ] 60 FPS maintained during rotation test
- [ ] Initial load time <3 seconds
- [ ] No performance regressions vs baseline

---

### Step 4.6: Accessibility Tests

**Install axe-playwright:**

```bash
npm install -D @axe-core/playwright
```

**Create `tests/accessibility.spec.ts`:**

```typescript
import { test, expect } from '@playwright/test'
import { injectAxe, checkA11y } from '@axe-core/playwright'

test.describe('Accessibility Tests', () => {
  test('homepage is accessible', async ({ page }) => {
    await page.goto('/')
    await injectAxe(page)

    await checkA11y(page, null, {
      detailedReport: true,
      detailedReportOptions: { html: true },
    })
  })

  test('keyboard navigation works', async ({ page }) => {
    await page.goto('/')

    // Tab through UI elements
    await page.keyboard.press('Tab')  // Focus file input
    await expect(page.locator('input[type="file"]')).toBeFocused()

    await page.keyboard.press('Tab')  // Focus export button
    await expect(page.locator('button:has-text("Export PNG")')).toBeFocused()
  })

  test('screen reader landmarks exist', async ({ page }) => {
    await page.goto('/')

    // Check for semantic HTML
    await expect(page.locator('main')).toBeVisible()
    await expect(page.locator('nav')).toBeVisible()
    await expect(page.locator('aside')).toBeVisible()  // Recent files sidebar
  })
})
```

**Validation:**

```bash
npx playwright test tests/accessibility.spec.ts
```

**Quality Gate:**
- [ ] No axe accessibility violations
- [ ] Keyboard navigation works (tab, enter, esc)
- [ ] Semantic HTML landmarks present (main, nav, aside)
- [ ] ARIA labels on interactive elements

---

### Phase 4 Final Validation

**Run full test suite:**

```bash
npx playwright test
```

**Expected output:**

```
Running 20 tests using 3 workers

  ✓ tests/visual-regression.spec.ts:5:3 › water molecule renders correctly (chromium)
  ✓ tests/visual-regression.spec.ts:5:3 › water molecule renders correctly (firefox)
  ✓ tests/visual-regression.spec.ts:5:3 › water molecule renders correctly (webkit)
  ✓ tests/functional.spec.ts:5:3 › file upload works (chromium)
  ... (all tests pass)

20 passed (45s)
```

**Quality Gates (Must ALL Pass):**

- [ ] Visual regression tests pass (Chrome, Firefox, Safari)
- [ ] Functional tests pass (upload, rotation, zoom, export, selection)
- [ ] Performance tests pass (60 FPS, <3s load)
- [ ] Accessibility tests pass (no axe violations, keyboard nav works)
- [ ] CI integration configured (GitHub Actions runs tests on push)

**Success Criteria:** All gates pass → Phase 4 COMPLETE.

---

## Phase 5: Production Deployment

**Goal:** Deploy to staging URL, test with real lab files, optimize for production.

### Step 5.1: Deploy to Vercel

**Implementation:**

```bash
cd ~/projects/axiom/axiom-gui/axiom-web

# Install Vercel CLI
npm install -g vercel

# Login (use GitHub OAuth)
vercel login

# Deploy to staging
vercel

# Follow prompts:
# - Project name: axiom-web
# - Framework: Vite
# - Build command: npm run build
# - Output directory: dist
```

**Validation:**

```bash
# Vercel outputs staging URL: https://axiom-web-abc123.vercel.app
# Open URL in browser
# Upload water.cif → should render correctly
```

**Quality Gate:**
- [ ] Deploys successfully to Vercel
- [ ] Staging URL accessible
- [ ] Water.cif renders on staging (same as localhost)

---

### Step 5.2: Production URL Configuration

**Implementation:**

```bash
# Set production domain (if custom domain available)
vercel domains add axiom.heinz-lab.com

# Deploy to production
vercel --prod
```

**Edit `CLAUDE.md` and memory to record production URL:**

```markdown
## Axiom Web Production URL

https://axiom.heinz-lab.com (or https://axiom-web.vercel.app)
```

**Quality Gate:**
- [ ] Production URL configured
- [ ] URL documented in memory

---

### Step 5.3: Test with Real Lab Files

**Test Cases (Use Sean's actual CIF files):**

1. **Water (H2O.cif)**
2. **Methane (CH4.cif)**
3. **ACACEP perovskite**
4. **2-BrPEA2PbI4 perovskite** (from Sean's messages)
5. **NaCl crystal**
6. **Large MXene structure** (stress test, if available)

**Validation Checklist:**

For EACH file:
- [ ] Upload successful (no errors)
- [ ] Structure renders correctly (correct atom count, positions)
- [ ] Element names correct (check periodic table)
- [ ] Bonds render correctly (no floating/disconnected bonds)
- [ ] Camera auto-frames structure
- [ ] 60 FPS rotation maintained
- [ ] Export PNG works
- [ ] No console errors

**Quality Gate:**
- [ ] All 6 test files render correctly on production URL
- [ ] No regressions vs localhost

---

### Step 5.4: Performance Optimization

**Implementation:**

1. **Enable compression (Vercel config):**

```json
// vercel.json
{
  "headers": [
    {
      "source": "/(.*)",
      "headers": [
        {
          "key": "Content-Encoding",
          "value": "gzip"
        }
      ]
    }
  ]
}
```

2. **Optimize WASM bundle (already done in Phase 1, verify):**

```bash
cd ~/projects/axiom/axiom-gui/axiom-renderer
wasm-opt -Oz pkg/axiom_renderer_bg.wasm -o pkg/axiom_renderer_bg.wasm
du -h pkg/axiom_renderer_bg.wasm  # Should be <5 MB
```

3. **Lighthouse audit:**

```bash
# Run Lighthouse in Chrome DevTools
# Target scores:
# - Performance: >90
# - Accessibility: >95
# - Best Practices: >90
# - SEO: >90
```

**Quality Gate:**
- [ ] WASM bundle <5 MB
- [ ] Initial load time <3s (tested on staging URL)
- [ ] Lighthouse Performance score >90

---

### Step 5.5: Error Tracking (Sentry Integration)

**Implementation:**

```bash
npm install @sentry/react @sentry/vite-plugin
```

**Edit `vite.config.ts`:**

```typescript
import { sentryVitePlugin } from '@sentry/vite-plugin'

export default defineConfig({
  plugins: [
    react(),
    wasm(),
    topLevelAwait(),
    sentryVitePlugin({
      org: 'heinz-lab',
      project: 'axiom-web',
      authToken: process.env.SENTRY_AUTH_TOKEN,
    }),
  ],
})
```

**Edit `src/main.tsx`:**

```typescript
import * as Sentry from '@sentry/react'

Sentry.init({
  dsn: 'https://your-sentry-dsn',
  environment: import.meta.env.MODE,
  tracesSampleRate: 1.0,
})
```

**Validation:**

```bash
# Trigger test error
# throw new Error('Test Sentry integration')

# Check Sentry dashboard → error should appear
```

**Quality Gate:**
- [ ] Sentry integrated
- [ ] Test error appears in Sentry dashboard
- [ ] Production errors will be tracked

---

### Phase 5 Final Validation

**Production Readiness Checklist:**

- [ ] Deployed to production URL
- [ ] All test files render correctly
- [ ] Performance optimized (WASM <5 MB, load time <3s)
- [ ] Lighthouse scores: Performance >90, Accessibility >95
- [ ] Error tracking configured (Sentry)
- [ ] CI/CD pipeline automated (GitHub Actions → Vercel deploy on push)
- [ ] Documentation updated (README, user guide)

**Quality Gates (Must ALL Pass):**

- [ ] Production URL accessible and stable
- [ ] All real lab files render correctly
- [ ] No console errors on production
- [ ] Monitoring configured

**Success Criteria:** All gates pass → Phase 5 COMPLETE.

---

## Phase 6: Headless Unification

**Goal:** CLI tool uses same `axiom-renderer` crate, achieving zero code duplication between web/CLI/headless rendering.

### Step 6.1: Extract Renderer into Shared Crate

**Already done in Phase 1** — `axiom-renderer` crate supports both WASM and native targets.

**Validation:**

```bash
cd ~/projects/axiom/axiom-gui/axiom-renderer

# Build for native (headless)
cargo build --release

# Build for WASM (web)
wasm-pack build --target web --release --out-dir pkg

# Both should succeed
```

**Quality Gate:**
- [ ] `axiom-renderer` builds for native target
- [ ] `axiom-renderer` builds for WASM target
- [ ] No target-specific code in core rendering logic

---

### Step 6.2: CLI Screenshot Tool (Native Headless)

**Implementation:**

**Create `axiom-cli/src/main.rs` (if doesn't exist, or update existing):**

```rust
// axiom-cli/src/main.rs

use axiom_renderer::{Renderer, AtomData};
use axiom_core::cif::parse_cif;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: axiom-cli <input.cif> <output.png>");
        std::process::exit(1);
    }

    let cif_path = &args[1];
    let output_path = &args[2];

    // Read CIF file
    let cif_content = fs::read_to_string(cif_path)?;
    let structure = parse_cif(&cif_content)?;

    // Convert to AtomData
    let atoms: Vec<AtomData> = structure.atoms.iter().map(|atom| {
        AtomData {
            position: atom.position,
            color: element_color(&atom.element),
            radius: element_radius(&atom.element),
        }
    }).collect();

    // Create headless renderer
    let mut renderer = Renderer::new_headless(800, 600).await?;

    // Auto-frame and render
    renderer.camera.frame_atoms(&atoms);
    let image_buffer = renderer.render_to_buffer(&atoms)?;

    // Save PNG
    image_buffer.save(output_path)?;

    println!("Screenshot saved to {}", output_path);

    Ok(())
}

fn element_color(element: &str) -> [f32; 3] {
    // Same as WASM version
    match element {
        "H" => [1.0, 1.0, 1.0],
        "C" => [0.2, 0.2, 0.2],
        "O" => [1.0, 0.0, 0.0],
        _ => [0.5, 0.5, 0.5],
    }
}

fn element_radius(element: &str) -> f32 {
    // Same as WASM version
    match element {
        "H" => 0.3,
        "C" => 0.7,
        "O" => 0.6,
        _ => 0.5,
    }
}
```

**Add `render_to_buffer` method in `axiom-renderer/src/renderer.rs`:**

```rust
impl Renderer {
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn new_headless(width: u32, height: u32) -> Result<Self, String> {
        let context = RenderContext::new_headless().await?;
        let camera = Camera::new(width, height);
        let pipeline = RenderPipeline::new(&context.device, wgpu::TextureFormat::Rgba8UnormSrgb);

        Ok(Self {
            context,
            camera,
            pipeline,
            width,
            height,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn render_to_buffer(&mut self, atoms: &[AtomData]) -> Result<image::RgbaImage, String> {
        // Create offscreen texture
        let texture = self.context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("render-target"),
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        // Render to texture
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        // ... (render atoms to view) ...

        // Copy texture to buffer
        let buffer_size = (self.width * self.height * 4) as wgpu::BufferAddress;
        let buffer = self.context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("output-buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self.context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("copy-encoder"),
        });

        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(self.width * 4),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        self.context.queue.submit(std::iter::once(encoder.finish()));

        // Map buffer and read pixels
        let buffer_slice = buffer.slice(..);
        let (tx, rx) = futures::channel::oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, |result| {
            tx.send(result).unwrap();
        });
        self.context.device.poll(wgpu::Maintain::Wait);
        pollster::block_on(rx).unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();
        let image = image::RgbaImage::from_raw(self.width, self.height, data.to_vec())
            .ok_or("Failed to create image from buffer")?;

        Ok(image)
    }
}
```

**Validation:**

```bash
cd ~/projects/axiom/axiom-gui/axiom-cli
cargo build --release

# Test screenshot generation
./target/release/axiom-cli ~/test-data/water.cif water-screenshot.png

# Open water-screenshot.png → should match web version rendering
```

**Quality Gate:**
- [ ] CLI compiles successfully
- [ ] CLI generates PNG screenshot
- [ ] Screenshot matches web version (pixel-perfect or very close)
- [ ] CLI uses same `axiom-renderer` crate as web

---

### Step 6.3: HPC Batch Rendering Script

**Implementation:**

**Create `scripts/batch-render.sh`:**

```bash
#!/bin/bash
# Batch render CIF files on Alpine HPC

CIF_DIR=$1
OUTPUT_DIR=$2

if [[ -z "$CIF_DIR" || -z "$OUTPUT_DIR" ]]; then
    echo "Usage: batch-render.sh <cif_directory> <output_directory>"
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

for cif_file in "$CIF_DIR"/*.cif; do
    filename=$(basename "$cif_file" .cif)
    echo "Rendering $filename..."
    axiom-cli "$cif_file" "$OUTPUT_DIR/${filename}.png"
done

echo "Batch rendering complete. Output: $OUTPUT_DIR"
```

**Create SLURM job script `scripts/render-job.slurm`:**

```bash
#!/bin/bash
#SBATCH --job-name=axiom-render
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --time=01:00:00
#SBATCH --partition=amilan
#SBATCH --output=axiom-render-%j.out

module load gcc/11.2.0
module load cuda/11.7  # If GPU rendering available

# Run batch rendering
./batch-render.sh /path/to/cif/files /path/to/output
```

**Validation:**

```bash
# SSH to Alpine (once configured)
ssh seflo4443@login.rc.colorado.edu

# Upload axiom-cli binary and scripts
scp axiom-cli scripts/batch-render.sh scripts/render-job.slurm alpine:~/

# Submit job
sbatch render-job.slurm

# Check job status
squeue -u seflo4443

# Once complete, check output
ls /path/to/output/*.png
```

**Quality Gate:**
- [ ] CLI works on Alpine HPC
- [ ] Batch rendering script works
- [ ] SLURM job submits successfully
- [ ] Output PNGs match web version

---

### Phase 6 Final Validation

**Zero Code Duplication Check:**

```bash
# Count lines of rendering code
cd ~/projects/axiom/axiom-gui/axiom-renderer/src
cloc renderer.rs camera.rs pipeline.rs geometry.rs shaders/

# Same code should power:
# 1. Web UI (via wasm.rs wrapper)
# 2. CLI (via native Rust)
# 3. HPC batch rendering (via native Rust)
```

**Quality Gates (Must ALL Pass):**

- [ ] CLI uses `axiom-renderer` crate (not duplicate code)
- [ ] CLI screenshot matches web version
- [ ] HPC batch rendering works
- [ ] Zero duplication between web/CLI/HPC rendering paths
- [ ] Performance: CLI <1s per screenshot (headless)

**Success Criteria:** All gates pass → Phase 6 COMPLETE → **PROJECT COMPLETE**

---

## Testing Strategy

### Testing Pyramid

```
         /\
        /  \  E2E Tests (Playwright)
       /____\
      /      \  Integration Tests
     /________\
    /          \  Unit Tests
   /____________\
```

**Unit Tests (Rust):**
- Test individual functions (camera matrices, geometry generation, bond detection)
- Fast, run locally and in CI
- Tools: `cargo test`

**Integration Tests (Rust):**
- Test renderer with real CIF data
- Validate output (pixel comparison)
- Tools: `cargo test --test integration`

**E2E Tests (Playwright):**
- Test full user workflows in browser
- Visual regression, functional, performance, accessibility
- Tools: `npx playwright test`

### CI/CD Pipeline (GitHub Actions)

**Create `.github/workflows/ci.yml`:**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test-rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run Rust tests
        run: |
          cd axiom-renderer
          cargo test
          cargo test --test integration

  test-wasm:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-unknown-unknown
      - name: Build WASM
        run: |
          cd axiom-renderer
          cargo install wasm-pack
          wasm-pack build --target web --release

  test-playwright:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: 18
      - name: Install dependencies
        run: |
          cd axiom-web
          npm ci
      - name: Install Playwright browsers
        run: npx playwright install --with-deps
      - name: Run Playwright tests
        run: |
          cd axiom-web
          npx playwright test
      - name: Upload test report
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: playwright-report
          path: axiom-web/playwright-report/

  deploy:
    needs: [test-rust, test-wasm, test-playwright]
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v3
      - name: Deploy to Vercel
        run: |
          npm install -g vercel
          cd axiom-web
          vercel --prod --token=${{ secrets.VERCEL_TOKEN }}
```

---

## Validation Gates

### Gate Philosophy

**NO GATE SKIPPING** — Each gate must pass before proceeding. If a gate fails:

1. **Debug:** Identify root cause
2. **Fix:** Implement solution
3. **Verify:** Re-run gate test
4. **Document:** Add note to memory if issue was non-obvious

**Quality > Speed** — If a gate takes 2 weeks to pass, that's acceptable.

### Gate Categories

1. **Functional Gates:** Does it work as specified?
2. **Performance Gates:** Does it meet speed/FPS targets?
3. **Visual Gates:** Does it look correct (no artifacts)?
4. **Cross-Browser Gates:** Does it work in Chrome/Firefox/Safari?
5. **Testing Gates:** Do automated tests pass?

---

## Troubleshooting Guide

### Common Issues

**Issue 1: WASM bundle >10 MB**

**Diagnosis:**
```bash
twiggy top -n 20 pkg/axiom_renderer_bg.wasm
```

**Solutions:**
- Remove unused dependencies in `Cargo.toml`
- Use `opt-level = 'z'` in release profile
- Run `wasm-opt -Oz` on bundle
- Lazy-load non-critical WASM modules

---

**Issue 2: WebGPU initialization fails**

**Diagnosis:**
Check browser console for error message.

**Possible causes:**
- Browser doesn't support WebGPU (check https://caniuse.com/webgpu)
- GPU blocked by browser settings (enable hardware acceleration)
- Adapter request failed (no compatible GPU)

**Solutions:**
- Test in Chrome Canary (latest WebGPU features)
- Enable `chrome://flags/#enable-unsafe-webgpu`
- Add WebGL fallback (if critical)

---

**Issue 3: FPS <60 during rotation**

**Diagnosis:**
```typescript
// In browser DevTools console:
const fpsLog = []
let lastTime = performance.now()
function measureFPS() {
  const now = performance.now()
  fpsLog.push(1000 / (now - lastTime))
  lastTime = now
  if (fpsLog.length < 60) requestAnimationFrame(measureFPS)
  else console.log('Avg FPS:', fpsLog.reduce((a,b) => a+b) / fpsLog.length)
}
requestAnimationFrame(measureFPS)
```

**Possible causes:**
- Too many draw calls (not using instancing)
- No frustum culling (rendering offscreen atoms)
- Shader too complex (expensive fragment shader)

**Solutions:**
- Use instanced rendering for atoms (single draw call for all spheres)
- Implement octree frustum culling
- Optimize shaders (reduce texture lookups, simplify lighting)

---

**Issue 4: Playwright visual regression tests failing**

**Diagnosis:**
```bash
npx playwright show-report
# Click on failed test → view diff image
```

**Possible causes:**
- Intentional visual change (update baseline)
- Font rendering difference (browser/OS)
- Anti-aliasing difference (GPU-specific)

**Solutions:**
- If change is intentional: `npx playwright test --update-snapshots`
- If flaky: Increase `maxDiffPixels` tolerance
- If GPU-specific: Disable anti-aliasing or use software renderer

---

**Issue 5: CLI screenshot doesn't match web version**

**Diagnosis:**
```bash
# Generate both versions
axiom-cli water.cif cli-water.png
# (Load water.cif in web, export PNG as web-water.png)

# Compare pixel-by-pixel
compare cli-water.png web-water.png diff.png
```

**Possible causes:**
- Different camera matrices (aspect ratio, FOV)
- Different shader precision (native vs WASM)
- Different texture filtering (GPU driver differences)

**Solutions:**
- Ensure camera parameters identical (hard-code test values)
- Use same shader source (atom.wgsl)
- Normalize texture filtering settings

---

## Summary

This implementation plan provides:

- **6 phases** with detailed step-by-step instructions
- **Comprehensive quality gates** at each step (functional, performance, visual, cross-browser)
- **Robust testing strategy** (unit, integration, E2E with Playwright)
- **Production deployment** with monitoring
- **Headless unification** (zero code duplication)
- **Troubleshooting guide** for common issues

**Timeline:** 6-8 weeks (quality gates dictate pace, not calendar)

**Owner:** axiom-agent (delegated from lab-agent)

**Status:** APPROVED — Implementation starts immediately upon receiving this plan.

---

**Next Action:** axiom-agent begins Phase 1, Step 1.1 (Create axiom-renderer crate).
