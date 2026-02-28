// GPU-based renderer using wgpu (WebGPU)
// Phase 2: Imposter sphere rendering with headless mode

use crate::atoms::{Atoms, Bonds};
use crate::colors::{element_to_ball_stick_radius, element_to_cpk_color};
use crate::errors::{AxiomError, Result};
use bytemuck::{Pod, Zeroable};
use wgpu;
use wgpu::util::DeviceExt;

/// Helper for buffer dimensions with proper padding
struct BufferDimensions {
    #[allow(dead_code)]
    width: u32,
    height: u32,
    padded_bytes_per_row: u32,
}

impl BufferDimensions {
    fn new(width: u32, height: u32) -> Self {
        let bytes_per_row = width * 4; // RGBA = 4 bytes per pixel
        let padded_bytes_per_row = Self::padded_bytes_per_row(bytes_per_row);
        Self {
            width,
            height,
            padded_bytes_per_row,
        }
    }

    fn padded_bytes_per_row(bytes_per_row: u32) -> u32 {
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded = (bytes_per_row + align - 1) / align * align;
        padded
    }
}

/// Renderer configuration
pub struct RendererConfig {
    pub width: u32,
    pub height: u32,
    pub headless: bool,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            headless: true,
        }
    }
}

/// Vertex data for GPU (per-instance data for each atom)
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct AtomVertex {
    position: [f32; 3],
    radius: f32,
    color: [f32; 3],
    _padding: f32, // Align to 16 bytes
}

/// Camera uniform data
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    position: [f32; 3],
    _padding: f32,
}

/// Main renderer struct
pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: RendererConfig,
    // Camera state
    camera_position: [f32; 3],
    camera_target: [f32; 3],
    camera_up: [f32; 3],
    // GPU resources
    render_pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
}

impl Renderer {
    /// Initialize the renderer (headless or windowed)
    pub async fn new(config: RendererConfig) -> Result<Self> {
        // Create wgpu instance (allow software/CPU rendering if no GPU)
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Request adapter (try hardware first, allow software fallback)
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None, // Headless
                force_fallback_adapter: false,
            })
            .await
            .or_else(|| {
                // Fallback: try with software adapter
                pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: None,
                    force_fallback_adapter: true,
                }))
            })
            .ok_or_else(|| AxiomError::RenderError("Failed to find GPU or software adapter. wgpu requires Vulkan, Metal, DX12, or software backend.".to_string()))?;

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Axiom Renderer Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .map_err(|e| AxiomError::RenderError(format!("Failed to create device: {}", e)))?;

        // Default camera position (looking down -Z axis)
        let camera_position = [0.0, 0.0, 50.0];
        let camera_target = [0.0, 0.0, 0.0];
        let camera_up = [0.0, 1.0, 0.0];

        // Load shader
        let shader_source = include_str!("shaders.wgsl");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Axiom Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // Create camera uniform buffer
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout for camera
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Create bind group
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Create render pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<AtomVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        // position
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        // radius
                        wgpu::VertexAttribute {
                            offset: 12,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32,
                        },
                        // color
                        wgpu::VertexAttribute {
                            offset: 16,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(Self {
            device,
            queue,
            config,
            camera_position,
            camera_target,
            camera_up,
            render_pipeline,
            camera_buffer,
            camera_bind_group,
        })
    }

    /// Synchronous initialization wrapper
    pub fn new_blocking(config: RendererConfig) -> Result<Self> {
        pollster::block_on(Self::new(config))
    }

    /// Set camera position
    pub fn set_camera(&mut self, position: [f32; 3], target: [f32; 3], up: [f32; 3]) {
        self.camera_position = position;
        self.camera_target = target;
        self.camera_up = up;
    }

    /// Reset camera to default
    pub fn reset_camera(&mut self) {
        self.camera_position = [0.0, 0.0, 50.0];
        self.camera_target = [0.0, 0.0, 0.0];
        self.camera_up = [0.0, 1.0, 0.0];
    }

    /// Build view matrix (look-at matrix)
    fn build_view_matrix(&self) -> [[f32; 4]; 4] {
        let pos = self.camera_position;
        let target = self.camera_target;
        let up = self.camera_up;

        // Forward vector (camera to target)
        let f = [
            target[0] - pos[0],
            target[1] - pos[1],
            target[2] - pos[2],
        ];
        let f_len = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt();
        let f = [f[0] / f_len, f[1] / f_len, f[2] / f_len];

        // Right vector (cross product: f × up)
        let r = [
            f[1] * up[2] - f[2] * up[1],
            f[2] * up[0] - f[0] * up[2],
            f[0] * up[1] - f[1] * up[0],
        ];
        let r_len = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
        let r = [r[0] / r_len, r[1] / r_len, r[2] / r_len];

        // True up vector (cross product: r × f)
        let u = [
            r[1] * f[2] - r[2] * f[1],
            r[2] * f[0] - r[0] * f[2],
            r[0] * f[1] - r[1] * f[0],
        ];

        // View matrix (inverse of camera transform)
        [
            [r[0], u[0], -f[0], 0.0],
            [r[1], u[1], -f[1], 0.0],
            [r[2], u[2], -f[2], 0.0],
            [
                -(r[0] * pos[0] + r[1] * pos[1] + r[2] * pos[2]),
                -(u[0] * pos[0] + u[1] * pos[1] + u[2] * pos[2]),
                f[0] * pos[0] + f[1] * pos[1] + f[2] * pos[2],
                1.0,
            ],
        ]
    }

    /// Build perspective projection matrix
    fn build_projection_matrix(&self) -> [[f32; 4]; 4] {
        let aspect = self.config.width as f32 / self.config.height as f32;
        let fov_y = 45.0_f32.to_radians();
        let near = 0.1;
        let far = 1000.0;

        let f = 1.0 / (fov_y / 2.0).tan();

        [
            [f / aspect, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, (far + near) / (near - far), -1.0],
            [0.0, 0.0, (2.0 * far * near) / (near - far), 0.0],
        ]
    }

    /// Multiply two 4x4 matrices
    fn mat4_mul(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
        let mut result = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result[i][j] += a[i][k] * b[k][j];
                }
            }
        }
        result
    }

    /// Render atoms to PNG using GPU
    pub fn render(&self, atoms: &Atoms) -> Result<Vec<u8>> {
        let width = self.config.width;
        let height = self.config.height;

        // VALIDATION: Prevent "Zero width not allowed" PNG encoding error
        if width == 0 || height == 0 {
            return Err(AxiomError::RenderError(format!(
                "Invalid render dimensions: {}x{} (width and height must be > 0)",
                width, height
            )));
        }

        // If no atoms, return blank image
        if atoms.len() == 0 {
            let img_buffer = image::RgbaImage::from_pixel(width, height, image::Rgba([0, 0, 0, 255]));
            let mut png_bytes = Vec::new();
            img_buffer
                .write_to(
                    &mut std::io::Cursor::new(&mut png_bytes),
                    image::ImageFormat::Png,
                )
                .map_err(|e| AxiomError::RenderError(format!("PNG encoding failed: {}", e)))?;
            return Ok(png_bytes);
        }

        // Build vertex data from atoms
        let mut vertices = Vec::new();

        for i in 0..atoms.len() {
            let atomic_num = atoms.elements[i];
            let color = element_to_cpk_color(atomic_num);
            let radius = element_to_ball_stick_radius(atomic_num);

            vertices.push(AtomVertex {
                position: [atoms.x[i], atoms.y[i], atoms.z[i]],
                radius,
                color,
                _padding: 0.0,
            });
        }

        // Create vertex buffer
        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Update camera uniform
        let view = self.build_view_matrix();
        let proj = self.build_projection_matrix();
        let view_proj = Self::mat4_mul(proj, view);

        let camera_uniform = CameraUniform {
            view_proj,
            view,
            position: self.camera_position,
            _padding: 0.0,
        };

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );

        // Create output texture
        let texture_desc = wgpu::TextureDescriptor {
            label: Some("Render Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };
        let output_texture = self.device.create_texture(&texture_desc);
        let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 1.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            // 6 vertices per instance (2 triangles per quad)
            render_pass.draw(0..6, 0..vertices.len() as u32);
        }

        // Copy texture to buffer
        let buffer_dimensions = BufferDimensions::new(width, height);
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: buffer_dimensions.padded_bytes_per_row as u64
                * buffer_dimensions.height as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(buffer_dimensions.padded_bytes_per_row),
                    rows_per_image: Some(buffer_dimensions.height),
                },
            },
            texture_desc.size,
        );

        self.queue.submit(Some(encoder.finish()));

        // Map buffer and read pixels
        let buffer_slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv()
            .unwrap()
            .map_err(|e| AxiomError::RenderError(format!("Failed to map buffer: {:?}", e)))?;

        let data = buffer_slice.get_mapped_range();

        // Copy to image buffer (remove padding)
        let mut img_buffer = image::RgbaImage::new(width, height);
        for y in 0..height {
            let start = (y * buffer_dimensions.padded_bytes_per_row) as usize;
            let end = start + (width * 4) as usize;
            let row = &data[start..end];
            for x in 0..width {
                let pixel_start = (x * 4) as usize;
                img_buffer.put_pixel(
                    x,
                    y,
                    image::Rgba([
                        row[pixel_start],
                        row[pixel_start + 1],
                        row[pixel_start + 2],
                        row[pixel_start + 3],
                    ]),
                );
            }
        }

        drop(data);
        output_buffer.unmap();

        // Encode to PNG
        let mut png_bytes = Vec::new();
        img_buffer
            .write_to(
                &mut std::io::Cursor::new(&mut png_bytes),
                image::ImageFormat::Png,
            )
            .map_err(|e| AxiomError::RenderError(format!("PNG encoding failed: {}", e)))?;

        Ok(png_bytes)
    }

    /// Save rendered image to file
    pub fn save_image(&self, atoms: &Atoms, path: &str) -> Result<()> {
        let png_bytes = self.render(atoms)?;
        std::fs::write(path, png_bytes)
            .map_err(|e| AxiomError::RenderError(format!("Failed to write file: {}", e)))?;
        Ok(())
    }

    /// Render atoms with bonds
    pub fn render_with_bonds(&self, atoms: &Atoms, bonds: &Bonds) -> Result<Vec<u8>> {
        // For GPU renderer, delegate to CPU renderer since GPU bond rendering not implemented yet
        use crate::renderer_cpu::{Renderer as CPURenderer, RendererConfig as CPUConfig, BackgroundColor};

        let bg_color = if self.config.headless {
            BackgroundColor::White  // Default for headless
        } else {
            BackgroundColor::Black
        };

        let cpu_config = CPUConfig {
            width: self.config.width,
            height: self.config.height,
            ssaa_factor: 2,  // Use SSAA for better quality
            specular_enabled: true,
            specular_power: 50.0,
            background: bg_color,
            ao_enabled: false,
            ao_samples: 16,
            ao_radius: 2.0,
            ao_strength: 0.5,
            // Performance optimizations (Phase 6)
            enable_frustum_culling: true,
            enable_lod: true,
            lod_config: crate::lod::LODConfig::default(),
            enable_octree: true,
            octree_max_depth: 8,
            octree_max_atoms_per_node: 32,
        };

        let mut cpu_renderer = CPURenderer::new(cpu_config)?;

        // Copy camera settings from GPU renderer to CPU renderer
        cpu_renderer.set_camera(self.camera_position, self.camera_target, self.camera_up);

        cpu_renderer.render_with_bonds(atoms, bonds)
    }

    /// Get device info (for debugging)
    pub fn device_info(&self) -> String {
        format!(
            "Axiom Renderer\nResolution: {}x{}\nMode: {}",
            self.config.width,
            self.config.height,
            if self.config.headless { "headless" } else { "windowed" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_init() {
        let config = RendererConfig {
            width: 800,
            height: 600,
            headless: true,
        };

        let renderer = Renderer::new_blocking(config);
        assert!(renderer.is_ok(), "Renderer should initialize successfully");
    }

    #[test]
    fn test_camera_controls() {
        let config = RendererConfig::default();
        let mut renderer = Renderer::new_blocking(config).unwrap();

        // Test set_camera
        renderer.set_camera([10.0, 20.0, 30.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(renderer.camera_position, [10.0, 20.0, 30.0]);

        // Test reset_camera
        renderer.reset_camera();
        assert_eq!(renderer.camera_position, [0.0, 0.0, 50.0]);
    }

    #[test]
    fn test_render_test_image() {
        let config = RendererConfig {
            width: 100,
            height: 100,
            headless: true,
        };
        let renderer = Renderer::new_blocking(config).unwrap();

        // Create empty atoms (just testing renderer, not actual rendering yet)
        let atoms = Atoms::new();

        let png_bytes = renderer.render(&atoms).unwrap();
        assert!(!png_bytes.is_empty(), "PNG should not be empty");
        assert!(png_bytes.starts_with(b"\x89PNG"), "Should be valid PNG");
    }
}
