// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

// Import axiom-core types (use correct API)
use axiom_core::{Atoms, Renderer, RendererConfig, BackgroundColor, compute_bonds};
use axiom_core::parsers;

/// Application state (shared across commands)
struct AppState {
    atoms: Mutex<Option<Atoms>>,
    renderer: Mutex<Option<Renderer>>,
}

/// Atoms data transferred to frontend (serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AtomsData {
    count: usize,
    elements: Vec<u8>,
    positions: Vec<f32>,
    bounds: BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BoundingBox {
    min: [f32; 3],
    max: [f32; 3],
    center: [f32; 3],
    radius: f32,
}

/// Rendering configuration from frontend
#[derive(Debug, Clone, Deserialize)]
struct RenderConfigInput {
    width: u32,
    height: u32,
    ssaa: u8,
    enable_ao: bool,
    ao_samples: u8,
    background: String,
}

/// Camera state from frontend
#[derive(Debug, Clone, Deserialize)]
struct CameraState {
    position: [f32; 3],
    target: [f32; 3],
    up: [f32; 3],
    fov: f32,
}

/// Result type for Tauri commands
type CommandResult<T> = Result<T, String>;

/// Load structure from file
#[tauri::command]
async fn load_structure(
    path: String,
    format: Option<String>,
    state: State<'_, AppState>,
) -> CommandResult<AtomsData> {
    let path = PathBuf::from(&path);

    // Determine format from extension if not provided
    let format = format.unwrap_or_else(|| {
        path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default()
    });

    // Load atoms based on format
    let atoms = match format.as_str() {
        "pdb" => parsers::pdb::parse_pdb(&path)
            .map_err(|e| format!("Failed to parse PDB: {}", e))?,
        "xyz" => parsers::xyz::parse_xyz(&path)
            .map_err(|e| format!("Failed to parse XYZ: {}", e))?,
        "gro" => parsers::gro::parse_gro(&path)
            .map_err(|e| format!("Failed to parse GRO: {}", e))?,
        "lammpstrj" | "lammps" => parsers::lammps::parse_lammps(&path)
            .map_err(|e| format!("Failed to parse LAMMPS: {}", e))?,
        _ => return Err(format!("Unsupported format: {}", format)),
    };

    // Calculate bounding box manually from Atoms SoA
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];

    for i in 0..atoms.len() {
        min[0] = min[0].min(atoms.x[i]);
        max[0] = max[0].max(atoms.x[i]);
        min[1] = min[1].min(atoms.y[i]);
        max[1] = max[1].max(atoms.y[i]);
        min[2] = min[2].min(atoms.z[i]);
        max[2] = max[2].max(atoms.z[i]);
    }

    let center = [
        (min[0] + max[0]) / 2.0,
        (min[1] + max[1]) / 2.0,
        (min[2] + max[2]) / 2.0,
    ];
    let radius = [
        (max[0] - min[0]) / 2.0,
        (max[1] - min[1]) / 2.0,
        (max[2] - min[2]) / 2.0,
    ]
    .iter()
    .fold(0.0f32, |a, &b| a.max(b));

    // Flatten positions into Vec<f32>
    let mut positions = Vec::with_capacity(atoms.len() * 3);
    for i in 0..atoms.len() {
        positions.push(atoms.x[i]);
        positions.push(atoms.y[i]);
        positions.push(atoms.z[i]);
    }

    let atoms_data = AtomsData {
        count: atoms.len(),
        elements: atoms.elements.clone(),
        positions,
        bounds: BoundingBox {
            min,
            max,
            center,
            radius,
        },
    };

    // Store atoms in state
    *state.atoms.lock().unwrap() = Some(atoms);

    Ok(atoms_data)
}

/// Render structure to PNG bytes
#[tauri::command]
async fn render_structure(
    config: RenderConfigInput,
    camera: Option<CameraState>,
    state: State<'_, AppState>,
) -> CommandResult<Vec<u8>> {
    // Get atoms from state
    let atoms_guard = state.atoms.lock().unwrap();
    let atoms = atoms_guard
        .as_ref()
        .ok_or("No structure loaded")?;

    // Create renderer
    let background = match config.background.as_str() {
        "white" => BackgroundColor::White,
        "transparent" => BackgroundColor::Transparent,
        _ => BackgroundColor::Black,
    };

    let renderer_config = RendererConfig {
        width: config.width,
        height: config.height,
        ssaa_factor: config.ssaa as u32,
        ao_enabled: config.enable_ao,
        ao_samples: config.ao_samples as u32,
        background,
        specular_enabled: true,
        specular_power: 50.0,
        ao_radius: 2.0,
        ao_strength: 0.5,
        // Performance optimizations (Phase 6)
        enable_frustum_culling: true,
        enable_lod: true,
        lod_config: axiom_core::LODConfig::default(),
        enable_octree: true,
        octree_max_depth: 8,
        octree_max_atoms_per_node: 32,
    };

    let mut renderer = Renderer::new(renderer_config)
        .map_err(|e| format!("Renderer creation failed: {}", e))?;

    // Set camera if provided, otherwise use auto camera
    if let Some(cam) = camera {
        renderer.set_camera(cam.position, cam.target, cam.up);
    } else {
        // Auto-fit camera to structure bounds - calculate manually from SoA
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];

        for i in 0..atoms.len() {
            min[0] = min[0].min(atoms.x[i]);
            max[0] = max[0].max(atoms.x[i]);
            min[1] = min[1].min(atoms.y[i]);
            max[1] = max[1].max(atoms.y[i]);
            min[2] = min[2].min(atoms.z[i]);
            max[2] = max[2].max(atoms.z[i]);
        }
        let center = [
            (min[0] + max[0]) / 2.0,
            (min[1] + max[1]) / 2.0,
            (min[2] + max[2]) / 2.0,
        ];
        let radius = [
            (max[0] - min[0]) / 2.0,
            (max[1] - min[1]) / 2.0,
            (max[2] - min[2]) / 2.0,
        ]
        .iter()
        .fold(0.0f32, |a, &b| a.max(b)) * 1.5; // 1.5x for padding

        // Camera position: look from front-top-right
        let cam_distance = radius * 0.25; // 10% of previous zoom (was 2.5, now 0.25)
        let cam_pos = [
            center[0] + cam_distance * 0.7,
            center[1] + cam_distance * 0.5,
            center[2] + cam_distance * 0.7,
        ];
        renderer.set_camera(cam_pos, center, [0.0, 1.0, 0.0]);
    }

    // Render - returns PNG bytes directly
    let png_bytes = renderer.render(atoms)
        .map_err(|e| format!("Rendering failed: {}", e))?;

    Ok(png_bytes)
}

/// Select atoms using semantic query
#[tauri::command]
async fn select_atoms(
    query: String,
    state: State<'_, AppState>,
) -> CommandResult<Vec<usize>> {
    let atoms_guard = state.atoms.lock().unwrap();
    let atoms = atoms_guard
        .as_ref()
        .ok_or("No structure loaded")?;

    let selection = axiom_core::select(atoms, &query)
        .map_err(|e| format!("Selection parse error: {}", e))?;

    Ok(selection)
}

/// Save image to file
#[tauri::command]
async fn save_image(path: String, image_data: Vec<u8>) -> CommandResult<()> {
    std::fs::write(&path, &image_data)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}

/// Get structure statistics
#[tauri::command]
async fn get_statistics(state: State<'_, AppState>) -> CommandResult<StructureStats> {
    let atoms_guard = state.atoms.lock().unwrap();
    let atoms = atoms_guard
        .as_ref()
        .ok_or("No structure loaded")?;

    let mut element_counts = std::collections::HashMap::new();
    for &elem in &atoms.elements {
        *element_counts.entry(elem).or_insert(0) += 1;
    }

    let stats = StructureStats {
        total_atoms: atoms.len(),
        element_counts,
        has_bonds: false,  // Bonds not stored in Atoms struct
        bond_count: 0,     // Would need separate bond tracking
    };

    Ok(stats)
}

#[derive(Debug, Serialize)]
struct StructureStats {
    total_atoms: usize,
    element_counts: std::collections::HashMap<u8, usize>,
    has_bonds: bool,
    bond_count: usize,
}

/// Compute bonds for loaded structure
#[tauri::command]
async fn compute_bonds_cmd(
    tolerance: f32,
    max_distance: f32,
    state: State<'_, AppState>
) -> CommandResult<usize> {
    let atoms_guard = state.atoms.lock().unwrap();
    let atoms = atoms_guard
        .as_ref()
        .ok_or("No structure loaded")?;

    // Use axiom_core::compute_bonds free function (3 args: atoms, tolerance, max_distance)
    let bonds = compute_bonds(atoms, tolerance, max_distance);
    let bond_count = bonds.atom1.len();

    // Note: Bonds are returned but not stored in Atoms (Atoms struct doesn't have bonds field)
    // Frontend would need to handle bond storage if needed

    Ok(bond_count)
}

/// Atom details for click selection
#[derive(Debug, Clone, Serialize)]
struct AtomDetails {
    index: usize,
    element: u8,
    position: [f32; 3],
}

/// Get atom details by index
#[tauri::command]
async fn get_atom_details(
    index: usize,
    state: State<'_, AppState>,
) -> CommandResult<AtomDetails> {
    let atoms_guard = state.atoms.lock().unwrap();
    let atoms = atoms_guard
        .as_ref()
        .ok_or("No structure loaded")?;

    if index >= atoms.len() {
        return Err(format!("Atom index {} out of bounds (total: {})", index, atoms.len()));
    }

    let details = AtomDetails {
        index,
        element: atoms.elements[index],
        position: [
            atoms.x[index],
            atoms.y[index],
            atoms.z[index],
        ],
    };

    Ok(details)
}

/// Pick atom at screen coordinates (returns closest atom to click)
/// This is a simplified version - for production, would use GPU picking or ray-casting
#[tauri::command]
async fn pick_atom_at_screen(
    _screen_x: f32,
    _screen_y: f32,
    _width: u32,
    _height: u32,
    state: State<'_, AppState>,
) -> CommandResult<Option<AtomDetails>> {
    let _atoms_guard = state.atoms.lock().unwrap();
    let _atoms = _atoms_guard
        .as_ref()
        .ok_or("No structure loaded")?;

    // For now, return None - this requires camera projection matrix
    // In production, this would project atoms to screen space and find closest
    // For Phase 4, we'll implement a simpler approach on frontend using canvas coordinates

    // Placeholder: Could implement basic ray-casting here with camera info
    Ok(None)
}

/// Export structure to file (PDB, XYZ, or CIF)
#[tauri::command]
async fn export_structure(
    path: String,
    format: String,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let atoms_guard = state.atoms.lock().unwrap();
    let atoms = atoms_guard
        .as_ref()
        .ok_or("No structure loaded")?;

    let content = match format.as_str() {
        "pdb" => export_to_pdb(atoms),
        "xyz" => export_to_xyz(atoms),
        "cif" => export_to_cif(atoms),
        _ => return Err(format!("Unsupported export format: {}", format)),
    };

    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

/// Export atoms to PDB format
fn export_to_pdb(atoms: &Atoms) -> String {
    let mut output = String::new();
    output.push_str("HEADER    Exported from Axiom GUI\n");
    output.push_str("TITLE     Molecular Structure\n");
    output.push_str("REMARK    Generated by Axiom\n");

    for i in 0..atoms.len() {
        let element = atoms.elements[i];
        let symbol = element_symbol(element);
        let x = atoms.x[i];
        let y = atoms.y[i];
        let z = atoms.z[i];

        // PDB ATOM format (fixed width)
        // ATOM   1234 CA   ALA A   1      12.345  12.345  12.345  1.00  0.00           C
        output.push_str(&format!(
            "ATOM  {:5} {:>2}             {:8.3}{:8.3}{:8.3}  1.00  0.00          {:>2}\n",
            i + 1, symbol, x, y, z, symbol
        ));
    }

    output.push_str("END\n");
    output
}

/// Export atoms to XYZ format
fn export_to_xyz(atoms: &Atoms) -> String {
    let mut output = String::new();
    output.push_str(&format!("{}\n", atoms.len()));
    output.push_str("Exported from Axiom GUI\n");

    for i in 0..atoms.len() {
        let element = atoms.elements[i];
        let symbol = element_symbol(element);
        let x = atoms.x[i];
        let y = atoms.y[i];
        let z = atoms.z[i];

        output.push_str(&format!("{} {:.6} {:.6} {:.6}\n", symbol, x, y, z));
    }

    output
}

/// Export atoms to CIF format (basic)
fn export_to_cif(atoms: &Atoms) -> String {
    let mut output = String::new();
    output.push_str("data_axiom_export\n");
    output.push_str("_audit_creation_method 'Axiom GUI'\n");
    output.push_str("loop_\n");
    output.push_str("_atom_site_label\n");
    output.push_str("_atom_site_type_symbol\n");
    output.push_str("_atom_site_fract_x\n");
    output.push_str("_atom_site_fract_y\n");
    output.push_str("_atom_site_fract_z\n");

    for i in 0..atoms.len() {
        let element = atoms.elements[i];
        let symbol = element_symbol(element);
        let x = atoms.x[i];
        let y = atoms.y[i];
        let z = atoms.z[i];

        output.push_str(&format!(
            "{}{} {} {:.6} {:.6} {:.6}\n",
            symbol, i + 1, symbol, x, y, z
        ));
    }

    output
}

/// Get element symbol from atomic number
fn element_symbol(atomic_number: u8) -> &'static str {
    match atomic_number {
        1 => "H",
        6 => "C",
        7 => "N",
        8 => "O",
        9 => "F",
        11 => "Na",
        12 => "Mg",
        14 => "Si",
        15 => "P",
        16 => "S",
        17 => "Cl",
        19 => "K",
        20 => "Ca",
        22 => "Ti",
        26 => "Fe",
        29 => "Cu",
        30 => "Zn",
        _ => "X", // Unknown element
    }
}

/// Measurement data structures for CSV export
#[derive(Debug, Deserialize)]
struct DistanceMeasurement {
    id: String,
    atom1: SelectedAtomData,
    atom2: SelectedAtomData,
    distance: f32,
}

#[derive(Debug, Deserialize)]
struct AngleMeasurement {
    id: String,
    atom1: SelectedAtomData,
    atom2: SelectedAtomData,
    atom3: SelectedAtomData,
    angle: f32,
}

#[derive(Debug, Deserialize)]
struct SelectedAtomData {
    index: usize,
    element: u8,
    position: [f32; 3],
}

/// Export measurements to CSV
#[tauri::command]
async fn export_measurements(
    path: String,
    distances: Vec<DistanceMeasurement>,
    angles: Vec<AngleMeasurement>,
) -> CommandResult<()> {
    let mut output = String::new();

    // Header
    output.push_str("Measurement Type,Atom 1,Atom 2,Atom 3,Value,Unit\n");

    // Distance measurements
    for dist in distances {
        let symbol1 = element_symbol(dist.atom1.element);
        let symbol2 = element_symbol(dist.atom2.element);
        output.push_str(&format!(
            "Distance,{}{}({}),{}{}({}),,-,{:.3},Å\n",
            symbol1,
            dist.atom1.index,
            format_position(&dist.atom1.position),
            symbol2,
            dist.atom2.index,
            format_position(&dist.atom2.position),
            dist.distance
        ));
    }

    // Angle measurements
    for angle in angles {
        let symbol1 = element_symbol(angle.atom1.element);
        let symbol2 = element_symbol(angle.atom2.element);
        let symbol3 = element_symbol(angle.atom3.element);
        output.push_str(&format!(
            "Angle,{}{}({}),{}{}({}),{}{}({}),{:.2},°\n",
            symbol1,
            angle.atom1.index,
            format_position(&angle.atom1.position),
            symbol2,
            angle.atom2.index,
            format_position(&angle.atom2.position),
            symbol3,
            angle.atom3.index,
            format_position(&angle.atom3.position),
            angle.angle
        ));
    }

    std::fs::write(&path, output)
        .map_err(|e| format!("Failed to write CSV: {}", e))?;

    Ok(())
}

/// Format position vector for CSV
fn format_position(pos: &[f32; 3]) -> String {
    format!("{:.2},{:.2},{:.2}", pos[0], pos[1], pos[2])
}

/// Convert RGBA image to PNG bytes
fn image_to_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    use std::io::Cursor;

    let mut png_data = Vec::new();
    let mut encoder = png::Encoder::new(Cursor::new(&mut png_data), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header()
        .map_err(|e| format!("PNG header error: {}", e))?;
    writer.write_image_data(rgba)
        .map_err(|e| format!("PNG write error: {}", e))?;

    drop(writer); // Ensure all data is flushed

    Ok(png_data)
}

/// Performance metrics for display
#[derive(Debug, Clone, Serialize)]
struct PerformanceMetrics {
    avg_fps: f64,
    avg_render_ms: f64,
    atoms_total: usize,
    atoms_rendered: usize,
    atoms_culled: usize,
    lod_high: usize,
    lod_medium: usize,
    lod_low: usize,
    lod_minimal: usize,
    sample_count: usize,
}

/// Octree statistics
#[derive(Debug, Clone, Serialize)]
struct OctreeStatsData {
    total_nodes: usize,
    total_atoms: usize,
    max_depth: u32,
}

/// Get current performance metrics from renderer
#[tauri::command]
fn get_performance_metrics(state: State<AppState>) -> Result<PerformanceMetrics, String> {
    let renderer_lock = state.renderer.lock().map_err(|e| format!("Lock error: {}", e))?;

    if let Some(renderer) = renderer_lock.as_ref() {
        let summary = renderer.get_performance_summary();
        Ok(PerformanceMetrics {
            avg_fps: summary.avg_fps,
            avg_render_ms: summary.avg_render_ms,
            atoms_total: summary.atoms_total,
            atoms_rendered: summary.atoms_rendered,
            atoms_culled: summary.atoms_culled,
            lod_high: summary.lod_high,
            lod_medium: summary.lod_medium,
            lod_low: summary.lod_low,
            lod_minimal: summary.lod_minimal,
            sample_count: summary.sample_count,
        })
    } else {
        Err("No renderer initialized".to_string())
    }
}

/// Get octree statistics (if built)
#[tauri::command]
fn get_octree_stats(state: State<AppState>) -> Result<Option<OctreeStatsData>, String> {
    let renderer_lock = state.renderer.lock().map_err(|e| format!("Lock error: {}", e))?;

    if let Some(renderer) = renderer_lock.as_ref() {
        if let Some(stats) = renderer.get_octree_stats() {
            Ok(Some(OctreeStatsData {
                total_nodes: stats.total_nodes,
                total_atoms: stats.total_atoms,
                max_depth: stats.max_depth,
            }))
        } else {
            Ok(None)
        }
    } else {
        Err("No renderer initialized".to_string())
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            atoms: Mutex::new(None),
            renderer: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            load_structure,
            render_structure,
            select_atoms,
            save_image,
            get_statistics,
            compute_bonds_cmd,
            get_atom_details,
            pick_atom_at_screen,
            export_structure,
            export_measurements,
            get_performance_metrics,
            get_octree_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
