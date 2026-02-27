// CPU-based renderer using software rasterization
// Replacement for GPU renderer when GPU acceleration unavailable

use crate::atoms::{Atoms, Bonds};
use crate::colors::{element_to_ball_stick_radius, element_to_cpk_color};
use crate::errors::{AxiomError, Result};
use crate::octree::Octree;
use crate::lod::{LODConfig, LODLevel, LODStats};
use crate::perf_metrics::{PerformanceTracker, PerfSummary, FrameMetrics};
use image::{Rgba, RgbaImage};
use rayon::prelude::*;

/// Background color preset
#[derive(Clone, Copy, Debug)]
pub enum BackgroundColor {
    Black,
    White,
    Transparent,
    Custom(u8, u8, u8, u8),  // RGBA
}

impl BackgroundColor {
    pub fn to_rgba(&self) -> [u8; 4] {
        match self {
            BackgroundColor::Black => [0, 0, 0, 255],
            BackgroundColor::White => [255, 255, 255, 255],
            BackgroundColor::Transparent => [0, 0, 0, 0],
            BackgroundColor::Custom(r, g, b, a) => [*r, *g, *b, *a],
        }
    }
}

impl Default for BackgroundColor {
    fn default() -> Self {
        BackgroundColor::Black
    }
}

/// Renderer configuration
#[derive(Clone)]
pub struct RendererConfig {
    pub width: u32,
    pub height: u32,
    pub ssaa_factor: u32,  // Supersampling factor (1 = no AA, 2 = 2x2 SSAA, etc.)
    pub specular_enabled: bool,  // Enable Blinn-Phong specular highlights
    pub specular_power: f32,  // Shininess exponent for specular highlights
    pub background: BackgroundColor,  // Background color (black/white/transparent/custom)
    pub ao_enabled: bool,  // Enable ambient occlusion
    pub ao_samples: u32,  // Number of AO samples (8-64, more = better quality but slower)
    pub ao_radius: f32,  // AO sampling radius in world space
    pub ao_strength: f32,  // AO darkening strength (0.0-1.0)
    // Performance optimizations
    pub enable_frustum_culling: bool,  // Enable frustum culling (skip off-screen atoms)
    pub enable_lod: bool,  // Enable Level of Detail rendering
    pub lod_config: LODConfig,  // LOD distance thresholds
    pub enable_octree: bool,  // Enable octree spatial indexing (for 10K+ atoms)
    pub octree_max_depth: u32,  // Max octree depth (default 8)
    pub octree_max_atoms_per_node: usize,  // Max atoms per octree leaf (default 32)
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            ssaa_factor: 2,  // 2x2 SSAA by default for better quality
            specular_enabled: true,
            specular_power: 50.0,  // Moderate shininess
            background: BackgroundColor::default(),  // Black background
            ao_enabled: false,  // AO disabled by default (performance)
            ao_samples: 16,  // Moderate quality
            ao_radius: 2.0,  // World-space sampling radius
            ao_strength: 0.5,  // Moderate darkening
            // Performance optimizations (enabled by default)
            enable_frustum_culling: true,
            enable_lod: true,
            lod_config: LODConfig::default(),
            enable_octree: true,
            octree_max_depth: 8,
            octree_max_atoms_per_node: 32,
        }
    }
}

/// Main CPU renderer struct
pub struct Renderer {
    config: RendererConfig,
    // Camera state
    camera_position: [f32; 3],
    camera_target: [f32; 3],
    camera_up: [f32; 3],
    // Performance tracking
    perf_tracker: PerformanceTracker,
    // Cached octree (rebuilt when atoms change)
    octree_cache: Option<Octree>,
    atoms_hash: u64,  // Hash of atoms to detect changes
}

/// Projected atom data for rendering
#[derive(Clone)]
struct ProjectedAtom {
    screen_x: f32,
    screen_y: f32,
    depth: f32,
    radius_px: f32,
    color: [f32; 3],
    world_pos: [f32; 3],
    world_radius: f32,  // World-space radius
    ao_factor: f32,  // Pre-computed AO factor (1.0 = bright, 0.0 = dark)
}

impl Renderer {
    /// Initialize the CPU renderer
    pub fn new(config: RendererConfig) -> Result<Self> {
        Ok(Self {
            config,
            camera_position: [0.0, 0.0, 50.0],
            camera_target: [0.0, 0.0, 0.0],
            camera_up: [0.0, 1.0, 0.0],
            perf_tracker: PerformanceTracker::new(60),  // Track last 60 frames
            octree_cache: None,
            atoms_hash: 0,
        })
    }

    /// Synchronous initialization (for API compatibility)
    pub fn new_blocking(config: RendererConfig) -> Result<Self> {
        Self::new(config)
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

    /// Auto-frame camera to fit all atoms with proper margins
    /// margin_factor: 1.0 = no margin, 1.3 = 30% padding around content
    pub fn auto_frame(&mut self, atoms: &Atoms, margin_factor: f32) {
        if atoms.len() == 0 {
            self.reset_camera();
            return;
        }

        // Calculate bounding box
        let mut min_x = atoms.x[0];
        let mut max_x = atoms.x[0];
        let mut min_y = atoms.y[0];
        let mut max_y = atoms.y[0];
        let mut min_z = atoms.z[0];
        let mut max_z = atoms.z[0];

        for i in 0..atoms.len() {
            let radius = element_to_ball_stick_radius(atoms.elements[i]);
            min_x = min_x.min(atoms.x[i] - radius);
            max_x = max_x.max(atoms.x[i] + radius);
            min_y = min_y.min(atoms.y[i] - radius);
            max_y = max_y.max(atoms.y[i] + radius);
            min_z = min_z.min(atoms.z[i] - radius);
            max_z = max_z.max(atoms.z[i] + radius);
        }

        // Center of bounding box
        let center = [
            (min_x + max_x) / 2.0,
            (min_y + max_y) / 2.0,
            (min_z + max_z) / 2.0,
        ];

        // Bounding box dimensions
        let size_x = max_x - min_x;
        let size_y = max_y - min_y;
        let size_z = max_z - min_z;
        let max_size = size_x.max(size_y).max(size_z);

        // FOV is 45 degrees, aspect ratio
        let fov_y = 45.0_f32.to_radians();
        let aspect = self.config.width as f32 / self.config.height as f32;

        // Calculate required distance to fit the object with margin
        // The visible height at distance d is: h = 2 * d * tan(fov_y/2)
        // We want: h = max_size * margin_factor
        // So: d = (max_size * margin_factor) / (2 * tan(fov_y/2))
        let dist_vertical = (max_size * margin_factor) / (2.0 * (fov_y / 2.0).tan());

        // For horizontal constraint (width-limited), calculate horizontal FOV
        // fov_x = 2 * atan(aspect * tan(fov_y/2))
        let fov_x = 2.0 * (aspect * (fov_y / 2.0).tan()).atan();
        let dist_horizontal = (max_size * margin_factor) / (2.0 * (fov_x / 2.0).tan());

        // Use the larger distance to ensure both constraints are met
        let distance = dist_vertical.max(dist_horizontal);

        // Position camera along +Z axis from center
        self.camera_target = center;
        self.camera_position = [
            center[0],
            center[1],
            center[2] + distance,
        ];
        self.camera_up = [0.0, 1.0, 0.0];

        // Debug logging to file
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/axiom_debug.log")
        {
            let _ = writeln!(file, "[Auto-frame] BBox: ({:.2}, {:.2}, {:.2}) to ({:.2}, {:.2}, {:.2})",
                     min_x, min_y, min_z, max_x, max_y, max_z);
            let _ = writeln!(file, "[Auto-frame] Size: {:.2} × {:.2} × {:.2}, max={:.2}",
                     size_x, size_y, size_z, max_size);
            let _ = writeln!(file, "[Auto-frame] Distance: vert={:.2}, horiz={:.2}, final={:.2}",
                     dist_vertical, dist_horizontal, distance);
            let _ = writeln!(file, "[Auto-frame] Camera: pos=({:.2}, {:.2}, {:.2}), target=({:.2}, {:.2}, {:.2})",
                     self.camera_position[0], self.camera_position[1], self.camera_position[2],
                     self.camera_target[0], self.camera_target[1], self.camera_target[2]);
        }
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

    /// Build perspective projection matrix (column-major: mat[col][row])
    fn build_projection_matrix(&self) -> [[f32; 4]; 4] {
        let aspect = self.config.width as f32 / self.config.height as f32;
        let fov_y = 45.0_f32.to_radians();
        let near = 0.1;
        let far = 1000.0;

        let f = 1.0 / (fov_y / 2.0).tan();

        // Column-major perspective projection (OpenGL convention)
        [
            [f / aspect, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, (far + near) / (near - far), -1.0],
            [0.0, 0.0, (2.0 * far * near) / (near - far), 0.0],
        ]
    }

    /// Transform point by 4x4 matrix (column-major: mat[col][row])
    fn transform_point(mat: [[f32; 4]; 4], p: [f32; 3]) -> [f32; 4] {
        let x = mat[0][0] * p[0] + mat[1][0] * p[1] + mat[2][0] * p[2] + mat[3][0];
        let y = mat[0][1] * p[0] + mat[1][1] * p[1] + mat[2][1] * p[2] + mat[3][1];
        let z = mat[0][2] * p[0] + mat[1][2] * p[1] + mat[2][2] * p[2] + mat[3][2];
        let w = mat[0][3] * p[0] + mat[1][3] * p[1] + mat[2][3] * p[2] + mat[3][3];
        [x, y, z, w]
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

    /// Extract frustum planes from view-projection matrix
    /// Returns 6 planes [left, right, bottom, top, near, far] in form [a, b, c, d]
    /// where ax + by + cz + d = 0
    fn extract_frustum_planes(view_proj: [[f32; 4]; 4]) -> [[f32; 4]; 6] {
        let mut planes = [[0.0; 4]; 6];

        // Left plane: add 4th column to 1st column
        planes[0] = [
            view_proj[3][0] + view_proj[0][0],
            view_proj[3][1] + view_proj[0][1],
            view_proj[3][2] + view_proj[0][2],
            view_proj[3][3] + view_proj[0][3],
        ];

        // Right plane: subtract 1st column from 4th column
        planes[1] = [
            view_proj[3][0] - view_proj[0][0],
            view_proj[3][1] - view_proj[0][1],
            view_proj[3][2] - view_proj[0][2],
            view_proj[3][3] - view_proj[0][3],
        ];

        // Bottom plane: add 4th column to 2nd column
        planes[2] = [
            view_proj[3][0] + view_proj[1][0],
            view_proj[3][1] + view_proj[1][1],
            view_proj[3][2] + view_proj[1][2],
            view_proj[3][3] + view_proj[1][3],
        ];

        // Top plane: subtract 2nd column from 4th column
        planes[3] = [
            view_proj[3][0] - view_proj[1][0],
            view_proj[3][1] - view_proj[1][1],
            view_proj[3][2] - view_proj[1][2],
            view_proj[3][3] - view_proj[1][3],
        ];

        // Near plane: add 4th column to 3rd column
        planes[4] = [
            view_proj[3][0] + view_proj[2][0],
            view_proj[3][1] + view_proj[2][1],
            view_proj[3][2] + view_proj[2][2],
            view_proj[3][3] + view_proj[2][3],
        ];

        // Far plane: subtract 3rd column from 4th column
        planes[5] = [
            view_proj[3][0] - view_proj[2][0],
            view_proj[3][1] - view_proj[2][1],
            view_proj[3][2] - view_proj[2][2],
            view_proj[3][3] - view_proj[2][3],
        ];

        // Normalize planes
        for plane in &mut planes {
            let length = (plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2]).sqrt();
            if length > 1e-6 {
                plane[0] /= length;
                plane[1] /= length;
                plane[2] /= length;
                plane[3] /= length;
            }
        }

        planes
    }

    /// Compute hash of atoms for cache invalidation
    fn compute_atoms_hash(atoms: &Atoms) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        atoms.len().hash(&mut hasher);
        // Hash first/last few atoms as a quick fingerprint
        if atoms.len() > 0 {
            atoms.x[0].to_bits().hash(&mut hasher);
            atoms.y[0].to_bits().hash(&mut hasher);
            atoms.z[0].to_bits().hash(&mut hasher);
            if atoms.len() > 1 {
                let last = atoms.len() - 1;
                atoms.x[last].to_bits().hash(&mut hasher);
                atoms.y[last].to_bits().hash(&mut hasher);
                atoms.z[last].to_bits().hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    /// Get or build octree for current atoms
    fn get_or_build_octree(&mut self, atoms: &Atoms) -> Option<&Octree> {
        if !self.config.enable_octree {
            return None;
        }

        let current_hash = Self::compute_atoms_hash(atoms);

        // Rebuild octree if atoms changed
        if self.atoms_hash != current_hash || self.octree_cache.is_none() {
            if atoms.len() > 100 {  // Only build octree for larger structures
                let octree = Octree::build(
                    atoms,
                    self.config.octree_max_depth,
                    self.config.octree_max_atoms_per_node,
                );
                self.octree_cache = Some(octree);
                self.atoms_hash = current_hash;
            } else {
                self.octree_cache = None;
            }
        }

        self.octree_cache.as_ref()
    }

    /// Project atoms to screen space with performance optimizations
    fn project_atoms(&mut self, atoms: &Atoms) -> Vec<ProjectedAtom> {
        self.project_atoms_with_lod(atoms, None)
    }

    /// Project atoms with optional LOD stats tracking
    fn project_atoms_with_lod(&mut self, atoms: &Atoms, _lod_stats: Option<&mut LODStats>) -> Vec<ProjectedAtom> {
        // Build or retrieve octree for large structures
        let _octree = self.get_or_build_octree(atoms);
        let view = self.build_view_matrix();
        let proj = self.build_projection_matrix();

        eprintln!("[Projection] Camera: pos=({}, {}, {}), target=({}, {}, {})",
                 self.camera_position[0], self.camera_position[1], self.camera_position[2],
                 self.camera_target[0], self.camera_target[1], self.camera_target[2]);

        let view_proj = Self::mat4_mul(proj, view);
        let frustum_planes = Self::extract_frustum_planes(view_proj);

        let width = self.config.width as f32;
        let height = self.config.height as f32;

        // Determine which atoms to render (frustum culling with octree if available)
        let atom_indices: Vec<usize> = if self.config.enable_frustum_culling && self.octree_cache.is_some() {
            // Use octree for fast frustum culling
            self.octree_cache.as_ref().unwrap().query_visible(&frustum_planes)
        } else {
            // No octree: render all atoms (or do per-atom culling below)
            (0..atoms.len()).collect()
        };

        // Parallelize atom projection using rayon
        let projected: Vec<ProjectedAtom> = atom_indices
            .into_par_iter()
            .filter_map(|i| {
                let world_pos = [atoms.x[i], atoms.y[i], atoms.z[i]];
                let view_pos = Self::transform_point(view, world_pos);
                let clip = Self::transform_point(proj, [view_pos[0], view_pos[1], view_pos[2]]);

                // Perspective divide
                if clip[3].abs() < 1e-6 {
                    return None; // Skip degenerate points
                }
                let ndc = [clip[0] / clip[3], clip[1] / clip[3], clip[2] / clip[3]];

                // Clip to view frustum
                if ndc[2] < -1.0 || ndc[2] > 1.0 {
                    return None;
                }

                // Convert to screen coordinates
                let screen_x = (ndc[0] + 1.0) * 0.5 * width;
                let screen_y = (1.0 - ndc[1]) * 0.5 * height; // Flip Y

                // Get color and radius
                let atomic_num = atoms.elements[i];
                let color = element_to_cpk_color(atomic_num);
                let mut world_radius = element_to_ball_stick_radius(atomic_num);

                // LOD: Determine level based on distance from camera
                let distance_from_camera = LODConfig::calculate_distance(self.camera_position, world_pos);
                let lod_level = if self.config.enable_lod {
                    self.config.lod_config.get_lod_level(distance_from_camera)
                } else {
                    LODLevel::High
                };

                // Apply LOD radius multiplier
                world_radius *= lod_level.radius_multiplier();

                // Project radius to screen space using view-space depth
                // View-space Z is negative (camera looks down -Z), so use -view_pos[2]
                let view_depth = -view_pos[2];

                // Perspective projection: radius_px = world_radius * focal_length / view_depth
                // focal_length = (1 / tan(fov_y/2)) * height / 2
                let fov_y = 45.0_f32.to_radians();
                let focal_length = (1.0 / (fov_y / 2.0).tan()) * height / 2.0;
                let radius_px = if view_depth > 0.0 {
                    world_radius * focal_length / view_depth
                } else {
                    0.0  // Behind camera
                };

                // Skip rendering atoms that are too small (< 0.5 pixel)
                if radius_px < 0.5 && self.config.enable_lod {
                    return None;
                }

                let depth = clip[2];

                if i == 0 {
                    use std::io::Write;
                    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/axiom_projection_log.txt") {
                        let _ = writeln!(file, "[Atom 0] world=({:.2}, {:.2}, {:.2}), view=({:.2}, {:.2}, {:.2}, {:.2}), clip=({:.2}, {:.2}, {:.2}, {:.2}), ndc=({:.2}, {:.2}, {:.2}), screen=({:.1}, {:.1}), view_depth={:.2}, focal_length={:.2}, radius_px={:.1}",
                                 world_pos[0], world_pos[1], world_pos[2],
                                 view_pos[0], view_pos[1], view_pos[2], view_pos[3],
                                 clip[0], clip[1], clip[2], clip[3],
                                 ndc[0], ndc[1], ndc[2],
                                 screen_x, screen_y, view_depth, focal_length, radius_px);
                    }
                }

                Some(ProjectedAtom {
                    screen_x,
                    screen_y,
                    depth,
                    radius_px,
                    color,
                    world_pos,
                    world_radius,
                    ao_factor: 1.0,  // Will be calculated below if AO enabled
                })
            })
            .collect();

        // Calculate AO factors if enabled (once per atom, not per pixel!)
        // Parallelized using rayon for better performance on large structures
        let mut projected = if self.config.ao_enabled {
            let ao_radius = self.config.ao_radius;
            let ao_strength = self.config.ao_strength;

            // Clone projected for read-only access in parallel computation
            let projected_ref = projected.clone();

            projected
                .into_par_iter()
                .map(|mut atom| {
                    let mut neighbor_count = 0;

                    // Count atoms within AO radius
                    for other in &projected_ref {
                        // Skip self-comparison (same position)
                        if (atom.world_pos[0] - other.world_pos[0]).abs() < 1e-6
                            && (atom.world_pos[1] - other.world_pos[1]).abs() < 1e-6
                            && (atom.world_pos[2] - other.world_pos[2]).abs() < 1e-6
                        {
                            continue;
                        }

                        let dx = atom.world_pos[0] - other.world_pos[0];
                        let dy = atom.world_pos[1] - other.world_pos[1];
                        let dz = atom.world_pos[2] - other.world_pos[2];
                        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

                        // Check if within AO radius (accounting for both radii)
                        if dist < (ao_radius + atom.world_radius + other.world_radius) {
                            neighbor_count += 1;
                        }
                    }

                    // Convert neighbor count to occlusion factor
                    // More neighbors = darker (lower factor)
                    let occlusion = (neighbor_count as f32 / 10.0).min(1.0); // Normalize roughly
                    atom.ao_factor = 1.0 - (occlusion * ao_strength);

                    atom
                })
                .collect()
        } else {
            projected
        };

        // Sort by depth (back to front for painter's algorithm)
        projected.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap());

        projected
    }

    /// Render a single atom and return pixels (parallel-safe, no mutation)
    /// Used for parallel atom rendering across multiple atoms
    fn render_atom_parallel(
        atom: &ProjectedAtom,
        light_dir: [f32; 3],
        camera_pos: [f32; 3],
        specular_enabled: bool,
        specular_power: f32,
        width: u32,
        height: u32,
    ) -> Vec<(u32, u32, Rgba<u8>)> {
        // Bounding box for rasterization
        let min_x = (atom.screen_x - atom.radius_px).floor().max(0.0) as u32;
        let max_x = (atom.screen_x + atom.radius_px).ceil().min(width as f32) as u32;
        let min_y = (atom.screen_y - atom.radius_px).floor().max(0.0) as u32;
        let max_y = (atom.screen_y + atom.radius_px).ceil().min(height as f32) as u32;

        let center_x = atom.screen_x;
        let center_y = atom.screen_y;
        let radius = atom.radius_px;
        let radius_sq = radius * radius;

        // Normalize light direction
        let light_len =
            (light_dir[0] * light_dir[0] + light_dir[1] * light_dir[1] + light_dir[2] * light_dir[2]).sqrt();
        let light_norm = [
            light_dir[0] / light_len,
            light_dir[1] / light_len,
            light_dir[2] / light_len,
        ];

        // Collect pixels (per scanline in parallel)
        (min_y..max_y)
            .into_par_iter()
            .flat_map(|y| {
                let mut scanline_pixels = Vec::new();

                for x in min_x..max_x {
                    let dx = x as f32 - center_x;
                    let dy = y as f32 - center_y;
                    let dist_sq = dx * dx + dy * dy;

                    if dist_sq <= radius_sq {
                        // Point is inside the circle
                        // Calculate sphere normal at this point (ray-sphere intersection)
                        let t = (radius_sq - dist_sq).sqrt();
                        let normal = [dx / radius, dy / radius, t / radius];

                        // Normalize
                        let normal_len =
                            (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
                        let normal_norm = [
                            normal[0] / normal_len,
                            normal[1] / normal_len,
                            normal[2] / normal_len,
                        ];

                        // Lambertian diffuse lighting
                        let n_dot_l = (normal_norm[0] * light_norm[0]
                            + normal_norm[1] * light_norm[1]
                            + normal_norm[2] * light_norm[2])
                            .max(0.0);

                        // Ambient + diffuse
                        let ambient = 0.2;
                        let diffuse = 0.6 * n_dot_l;

                        // Blinn-Phong specular highlights
                        let specular = if specular_enabled && n_dot_l > 0.0 {
                            // View direction (from surface point to camera)
                            let view_dir = [
                                camera_pos[0] - atom.world_pos[0],
                                camera_pos[1] - atom.world_pos[1],
                                camera_pos[2] - atom.world_pos[2],
                            ];
                            let view_len = (view_dir[0] * view_dir[0] + view_dir[1] * view_dir[1] + view_dir[2] * view_dir[2]).sqrt();
                            let view_norm = [view_dir[0] / view_len, view_dir[1] / view_len, view_dir[2] / view_len];

                            // Half-vector between light and view
                            let half_x = light_norm[0] + view_norm[0];
                            let half_y = light_norm[1] + view_norm[1];
                            let half_z = light_norm[2] + view_norm[2];
                            let half_len = (half_x * half_x + half_y * half_y + half_z * half_z).sqrt();
                            let half_norm = [half_x / half_len, half_y / half_len, half_z / half_len];

                            // Specular intensity
                            let n_dot_h = (normal_norm[0] * half_norm[0]
                                + normal_norm[1] * half_norm[1]
                                + normal_norm[2] * half_norm[2])
                                .max(0.0);
                            0.4 * n_dot_h.powf(specular_power)
                        } else {
                            0.0
                        };

                        let mut intensity = (ambient + diffuse).min(1.0);

                        // Apply pre-computed ambient occlusion
                        intensity *= atom.ao_factor;

                        // Apply lighting to color (diffuse + specular + AO)
                        let r = ((atom.color[0] * intensity + specular) * 255.0).min(255.0) as u8;
                        let g = ((atom.color[1] * intensity + specular) * 255.0).min(255.0) as u8;
                        let b = ((atom.color[2] * intensity + specular) * 255.0).min(255.0) as u8;

                        scanline_pixels.push((x, y, Rgba([r, g, b, 255])));
                    }
                }

                scanline_pixels
            })
            .collect()
    }


    /// Draw a line between two points (Bresenham's algorithm)
    fn draw_line(
        img: &mut RgbaImage,
        x0: i32, y0: i32,
        x1: i32, y1: i32,
        color: Rgba<u8>,
        thickness: u32,
    ) {
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = x0;
        let mut y = y0;

        let width = img.width() as i32;
        let height = img.height() as i32;

        loop {
            // Draw circle for thickness
            for dy_offset in -(thickness as i32)..=(thickness as i32) {
                for dx_offset in -(thickness as i32)..=(thickness as i32) {
                    if dx_offset * dx_offset + dy_offset * dy_offset <= (thickness * thickness) as i32 {
                        let px = x + dx_offset;
                        let py = y + dy_offset;
                        if px >= 0 && px < width && py >= 0 && py < height {
                            img.put_pixel(px as u32, py as u32, color);
                        }
                    }
                }
            }

            if x == x1 && y == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Render atoms and bonds to PNG using CPU
    pub fn render_with_bonds(&mut self, atoms: &Atoms, bonds: &Bonds) -> Result<Vec<u8>> {
        // Renders atoms at high-res, draws bonds, then downsamples
        use std::io::Write;
        eprintln!("[RENDER_WITH_BONDS] CALLED! {} atoms, {} bonds", atoms.len(), bonds.len());
        let _ = std::fs::write("/tmp/render_with_bonds_CALLED.txt", format!("{} atoms, {} bonds\n", atoms.len(), bonds.len()));

        // We need to render atoms at high-res, draw bonds, THEN downsample
        // The current render() function downsamples before returning, so we need
        // to duplicate the rendering logic here

        let width = self.config.width;
        let height = self.config.height;
        let ssaa_factor = self.config.ssaa_factor;
        let render_width = width * ssaa_factor;
        let render_height = height * ssaa_factor;

        // Create background at high resolution
        let bg_color = self.config.background.to_rgba();
        let mut img_highres = RgbaImage::from_pixel(render_width, render_height, Rgba(bg_color));

        // Temporarily scale config for projection at high resolution
        let original_width = self.config.width;
        let original_height = self.config.height;
        self.config.width = render_width;
        self.config.height = render_height;

        // Project atoms at high resolution
        eprintln!("[RENDER_WITH_BONDS] About to project atoms...");
        let projected = self.project_atoms(atoms);

        // Restore original config
        self.config.width = original_width;
        self.config.height = original_height;

        // Light direction (from top-right-front)
        let light_dir = [0.5, 0.5, 1.0];

        // CRITICAL: Draw bonds FIRST, then atoms on top
        // This ensures atoms occlude bonds naturally (bonds are "behind" atoms)

        // Bond appearance: subtle gray, moderate thickness
        let bond_color = Rgba([180, 180, 180, 255]);  // Light gray
        let bond_thickness = 2 * ssaa_factor;  // Moderate thickness scaled by SSAA

        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).write(true).truncate(true).open("/tmp/axiom_bond_debug_v2.txt") {
            let _ = writeln!(file, "[Bond Rendering v2] Drawing {} bonds on {}x{} image", bonds.len(), render_width, render_height);
            let _ = writeln!(file, "Bond thickness: {}, SSAA factor: {}", bond_thickness, ssaa_factor);
            for i in 0..bonds.len() {
                let atom1_idx = bonds.atom1[i] as usize;
                let atom2_idx = bonds.atom2[i] as usize;

                if atom1_idx < projected.len() && atom2_idx < projected.len() {
                    let proj1 = &projected[atom1_idx];
                    let proj2 = &projected[atom2_idx];

                    let _ = writeln!(file, "[Bond {}] atom {} ({:.1}, {:.1}) -> atom {} ({:.1}, {:.1})",
                             i, atom1_idx, proj1.screen_x, proj1.screen_y,
                             atom2_idx, proj2.screen_x, proj2.screen_y);

                    Self::draw_line(
                        &mut img_highres,
                        proj1.screen_x as i32,
                        proj1.screen_y as i32,
                        proj2.screen_x as i32,
                        proj2.screen_y as i32,
                        bond_color,
                        bond_thickness,
                    );
                }
            }
            let _ = writeln!(file, "Bond drawing complete. Now rendering atoms...");
        }

        // Now render atoms on top of bonds
        // Parallelize atom rendering: collect all pixels from all atoms, then write
        let all_pixels: Vec<(u32, u32, Rgba<u8>)> = projected
            .par_iter()
            .flat_map(|atom| {
                Self::render_atom_parallel(
                    atom,
                    light_dir,
                    self.camera_position,
                    self.config.specular_enabled,
                    self.config.specular_power,
                    render_width,
                    render_height,
                )
            })
            .collect();

        // Write all pixels to image (sequential to avoid race conditions)
        for (x, y, color) in all_pixels {
            img_highres.put_pixel(x, y, color);
        }

        // Downsample to final resolution
        let img = if ssaa_factor > 1 {
            image::imageops::resize(&img_highres, width, height, image::imageops::FilterType::Lanczos3)
        } else {
            img_highres
        };

        // Encode to PNG
        let mut png_bytes = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut png_bytes),
            image::ImageFormat::Png,
        )
        .map_err(|e| AxiomError::RenderError(format!("PNG encoding failed: {}", e)))?;

        Ok(png_bytes)
    }

    /// Render atoms to PNG using CPU
    pub fn render(&mut self, atoms: &Atoms) -> Result<Vec<u8>> {
        // Debug: Write to file to confirm this function is called
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/axiom_render_log.txt")
        {
            let _ = writeln!(file, "[render] CALLED with {} atoms, camera=({:.2}, {:.2}, {:.2})",
                           atoms.len(), self.camera_position[0], self.camera_position[1], self.camera_position[2]);
        }

        let width = self.config.width;
        let height = self.config.height;

        eprintln!("[CPU Renderer] Starting render: {}x{}, {} atoms", width, height, atoms.len());
        eprintln!("[CPU Renderer] Camera: pos=({}, {}, {}), target=({}, {}, {})",
                 self.camera_position[0], self.camera_position[1], self.camera_position[2],
                 self.camera_target[0], self.camera_target[1], self.camera_target[2]);

        // NOTE: Auto-framing disabled - respect user's camera settings
        // If you want auto-framing, call renderer.auto_frame() manually before render()
        // self.auto_frame(atoms, 2.0);

        // SSAA: render at higher resolution then downsample
        let ssaa_factor = self.config.ssaa_factor;
        let render_width = width * ssaa_factor;
        let render_height = height * ssaa_factor;

        eprintln!("[CPU Renderer] SSAA {}x: rendering at {}x{}, downsampling to {}x{}",
                  ssaa_factor, render_width, render_height, width, height);

        // Create background at high resolution (configurable color)
        let bg_color = self.config.background.to_rgba();
        let mut img_highres = RgbaImage::from_pixel(render_width, render_height, Rgba(bg_color));

        // If no atoms, return blank image
        if atoms.len() == 0 {
            let img_final = if ssaa_factor > 1 {
                image::imageops::resize(&img_highres, width, height, image::imageops::FilterType::Lanczos3)
            } else {
                img_highres
            };
            let mut png_bytes = Vec::new();
            img_final.write_to(
                &mut std::io::Cursor::new(&mut png_bytes),
                image::ImageFormat::Png,
            )
            .map_err(|e| AxiomError::RenderError(format!("PNG encoding failed: {}", e)))?;
            return Ok(png_bytes);
        }

        // Temporarily scale config for projection
        let original_width = self.config.width;
        let original_height = self.config.height;
        self.config.width = render_width;
        self.config.height = render_height;

        // Project atoms to screen space (at high resolution)
        eprintln!("[RENDER] About to project atoms...");
        let projected = self.project_atoms(atoms);

        // Restore original config
        self.config.width = original_width;
        self.config.height = original_height;

        // Light direction (from top-right-front)
        let light_dir = [0.5, 0.5, 1.0];

        // Render each atom (back to front) at high resolution
        // Parallelize atom rendering: collect all pixels from all atoms, then write
        let all_pixels: Vec<(u32, u32, Rgba<u8>)> = projected
            .par_iter()
            .flat_map(|atom| {
                Self::render_atom_parallel(
                    atom,
                    light_dir,
                    self.camera_position,
                    self.config.specular_enabled,
                    self.config.specular_power,
                    render_width,
                    render_height,
                )
            })
            .collect();

        // Write all pixels to image (sequential to avoid race conditions)
        for (x, y, color) in all_pixels {
            img_highres.put_pixel(x, y, color);
        }

        // Downsample to final resolution
        let img = if ssaa_factor > 1 {
            eprintln!("[CPU Renderer] Downsampling {}x{} -> {}x{}...", render_width, render_height, width, height);
            image::imageops::resize(&img_highres, width, height, image::imageops::FilterType::Lanczos3)
        } else {
            img_highres
        };

        // Encode to PNG
        let mut png_bytes = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut png_bytes),
            image::ImageFormat::Png,
        )
        .map_err(|e| AxiomError::RenderError(format!("PNG encoding failed: {}", e)))?;

        Ok(png_bytes)
    }

    /// Save rendered image to file
    pub fn save_image(&mut self, atoms: &Atoms, path: &str) -> Result<()> {
        let png_bytes = self.render(atoms)?;
        std::fs::write(path, png_bytes)
            .map_err(|e| AxiomError::RenderError(format!("Failed to write file: {}", e)))?;
        Ok(())
    }

    /// Get performance summary (FPS, render time, atom counts, LOD stats)
    pub fn get_performance_summary(&self) -> PerfSummary {
        self.perf_tracker.summary()
    }

    /// Get recent frame metrics
    pub fn get_latest_frame_metrics(&self) -> Option<&FrameMetrics> {
        self.perf_tracker.latest()
    }

    /// Reset performance tracking
    pub fn reset_performance_metrics(&mut self) {
        self.perf_tracker = PerformanceTracker::new(60);
    }

    /// Get octree statistics (if built)
    pub fn get_octree_stats(&self) -> Option<crate::octree::OctreeStats> {
        self.octree_cache.as_ref().map(|octree| octree.stats())
    }

    /// Get device info (for debugging)
    pub fn device_info(&self) -> String {
        format!(
            "Axiom CPU Renderer\nResolution: {}x{}\nMode: software rasterization",
            self.config.width, self.config.height
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_init() {
        let mut config = RendererConfig::default();
        config.width = 800;
        config.height = 600;

        let renderer = Renderer::new_blocking(config);
        assert!(renderer.is_ok(), "CPU renderer should initialize successfully");
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
    fn test_render_empty() {
        let mut config = RendererConfig::default();
        config.width = 100;
        config.height = 100;
        let mut renderer = Renderer::new_blocking(config).unwrap();

        let atoms = Atoms::new();
        let png_bytes = renderer.render(&atoms).unwrap();
        assert!(!png_bytes.is_empty(), "PNG should not be empty");
        assert!(png_bytes.starts_with(b"\x89PNG"), "Should be valid PNG");
    }
}
