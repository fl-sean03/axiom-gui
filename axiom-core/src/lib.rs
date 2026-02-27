// Axiom Core Library
// Next-generation atomic visualization for agents and humans

pub mod atoms;
pub mod bonds;
pub mod colors;
pub mod errors;
pub mod parsers;
pub mod renderer;      // GPU renderer (wgpu) - currently non-functional on ARM server
pub mod renderer_cpu;  // CPU renderer (software rasterization) - ACTIVE
pub mod selection;     // Semantic selection parser for agent-native queries
pub mod octree;        // Spatial indexing for large structures
pub mod lod;           // Level of Detail rendering system
pub mod perf_metrics;  // Performance tracking and monitoring

// Re-exports (use CPU renderer by default)
pub use atoms::{Atoms, Bonds, UnitCell};
pub use bonds::{compute_bonds, compute_bonds_default};
pub use colors::{element_to_cpk_color, element_to_vdw_radius, element_to_ball_stick_radius};
pub use errors::{AxiomError, Result};
pub use renderer_cpu::{Renderer, RendererConfig, BackgroundColor};  // CPU renderer is now default
pub use selection::{select, parse_selection, evaluate_selection, SelectionAST};
pub use octree::{Octree, AABB, OctreeStats};
pub use lod::{LODLevel, LODConfig, LODStats};
pub use perf_metrics::{PerformanceTracker, PerfSummary, FrameMetrics};
