// Structure of Arrays (SoA) for GPU-optimized atomic data
use serde::{Deserialize, Serialize};

/// Structure of Arrays for atomic data
///
/// This layout is optimized for GPU memory coalescing.
/// Instead of storing atoms as `Vec<Atom>` where each Atom is a struct,
/// we store separate arrays for each property.
///
/// Benefits:
/// - GPU memory coalescing (sequential access to x, y, z)
/// - Cache efficiency (only load needed properties)
/// - SIMD-friendly (vectorized operations on arrays)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Atoms {
    /// X coordinates (Angstroms)
    pub x: Vec<f32>,
    /// Y coordinates (Angstroms)
    pub y: Vec<f32>,
    /// Z coordinates (Angstroms)
    pub z: Vec<f32>,
    /// Atomic numbers (1 = H, 6 = C, 8 = O, etc.)
    pub elements: Vec<u8>,
    /// Optional: Partial charges (for force field calculations)
    pub charges: Option<Vec<f32>>,
    /// Optional: Atom types (LAMMPS ff types, or other typing schemes)
    pub atom_types: Option<Vec<u32>>,
    /// Optional: Molecule IDs (for LAMMPS multi-molecule systems)
    pub molecule_ids: Option<Vec<u32>>,
    /// Optional: Residue names (e.g., "ALA", "GLY", "WAT")
    pub residue_names: Option<Vec<String>>,
    /// Optional: Chain IDs (e.g., "A", "B")
    pub chain_ids: Option<Vec<String>>,
    /// Optional: Residue indices
    pub residue_indices: Option<Vec<u32>>,
}

impl Atoms {
    /// Create a new empty Atoms structure
    pub fn new() -> Self {
        Atoms {
            x: Vec::new(),
            y: Vec::new(),
            z: Vec::new(),
            elements: Vec::new(),
            charges: None,
            atom_types: None,
            molecule_ids: None,
            residue_names: None,
            chain_ids: None,
            residue_indices: None,
        }
    }

    /// Create Atoms with a specific capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Atoms {
            x: Vec::with_capacity(capacity),
            y: Vec::with_capacity(capacity),
            z: Vec::with_capacity(capacity),
            elements: Vec::with_capacity(capacity),
            charges: None,
            atom_types: None,
            molecule_ids: None,
            residue_names: None,
            chain_ids: None,
            residue_indices: None,
        }
    }

    /// Number of atoms
    pub fn len(&self) -> usize {
        self.x.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.x.is_empty()
    }

    /// Add an atom
    pub fn push(&mut self, x: f32, y: f32, z: f32, element: u8) {
        self.x.push(x);
        self.y.push(y);
        self.z.push(z);
        self.elements.push(element);
    }

    /// Get atom position
    pub fn position(&self, index: usize) -> Option<[f32; 3]> {
        if index < self.len() {
            Some([self.x[index], self.y[index], self.z[index]])
        } else {
            None
        }
    }

    /// Get element at index
    pub fn element(&self, index: usize) -> Option<u8> {
        self.elements.get(index).copied()
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.x.clear();
        self.y.clear();
        self.z.clear();
        self.elements.clear();
        self.charges = None;
        self.atom_types = None;
        self.molecule_ids = None;
        self.residue_names = None;
        self.chain_ids = None;
        self.residue_indices = None;
    }

    /// Reserve capacity for additional atoms
    pub fn reserve(&mut self, additional: usize) {
        self.x.reserve(additional);
        self.y.reserve(additional);
        self.z.reserve(additional);
        self.elements.reserve(additional);
    }
}

impl Default for Atoms {
    fn default() -> Self {
        Self::new()
    }
}

/// Bond data (pairs of atom indices)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bonds {
    /// First atom index
    pub atom1: Vec<u32>,
    /// Second atom index
    pub atom2: Vec<u32>,
    /// Bond order (1 = single, 2 = double, 3 = triple)
    pub order: Vec<u8>,
}

impl Bonds {
    pub fn new() -> Self {
        Bonds {
            atom1: Vec::new(),
            atom2: Vec::new(),
            order: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Bonds {
            atom1: Vec::with_capacity(capacity),
            atom2: Vec::with_capacity(capacity),
            order: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.atom1.len()
    }

    pub fn is_empty(&self) -> bool {
        self.atom1.is_empty()
    }

    pub fn push(&mut self, atom1: u32, atom2: u32, order: u8) {
        self.atom1.push(atom1);
        self.atom2.push(atom2);
        self.order.push(order);
    }

    pub fn get(&self, index: usize) -> Option<(u32, u32, u8)> {
        if index < self.len() {
            Some((self.atom1[index], self.atom2[index], self.order[index]))
        } else {
            None
        }
    }
}

impl Default for Bonds {
    fn default() -> Self {
        Self::new()
    }
}

/// Unit cell for periodic systems
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UnitCell {
    /// Cell matrix: [[a_x, b_x, c_x],
    ///               [a_y, b_y, c_y],
    ///               [a_z, b_z, c_z]]
    pub matrix: [[f32; 3]; 3],
}

impl UnitCell {
    /// Create unit cell from cell vectors
    pub fn from_vectors(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> Self {
        UnitCell {
            matrix: [a, b, c],
        }
    }

    /// Create unit cell from lengths and angles (degrees)
    pub fn from_lengths_angles(
        a: f32,
        b: f32,
        c: f32,
        alpha: f32,
        beta: f32,
        gamma: f32,
    ) -> Self {
        let alpha_rad = alpha.to_radians();
        let beta_rad = beta.to_radians();
        let gamma_rad = gamma.to_radians();

        let cos_alpha = alpha_rad.cos();
        let cos_beta = beta_rad.cos();
        let cos_gamma = gamma_rad.cos();
        let sin_gamma = gamma_rad.sin();

        let c_x = c * cos_beta;
        let c_y = c * (cos_alpha - cos_beta * cos_gamma) / sin_gamma;
        let c_z = (c * c - c_x * c_x - c_y * c_y).sqrt();

        UnitCell {
            matrix: [
                [a, 0.0, 0.0],
                [b * cos_gamma, b * sin_gamma, 0.0],
                [c_x, c_y, c_z],
            ],
        }
    }

    /// Get cell volume
    pub fn volume(&self) -> f32 {
        let a = self.matrix[0];
        let b = self.matrix[1];
        let c = self.matrix[2];

        // Volume = |a · (b × c)|
        let b_cross_c = [
            b[1] * c[2] - b[2] * c[1],
            b[2] * c[0] - b[0] * c[2],
            b[0] * c[1] - b[1] * c[0],
        ];

        (a[0] * b_cross_c[0] + a[1] * b_cross_c[1] + a[2] * b_cross_c[2]).abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atoms_creation() {
        let atoms = Atoms::new();
        assert_eq!(atoms.len(), 0);
        assert!(atoms.is_empty());
    }

    #[test]
    fn test_atoms_push() {
        let mut atoms = Atoms::new();
        atoms.push(1.0, 2.0, 3.0, 6); // Carbon at (1, 2, 3)

        assert_eq!(atoms.len(), 1);
        assert_eq!(atoms.position(0), Some([1.0, 2.0, 3.0]));
        assert_eq!(atoms.element(0), Some(6));
    }

    #[test]
    fn test_bonds_creation() {
        let mut bonds = Bonds::new();
        bonds.push(0, 1, 1); // Single bond between atoms 0 and 1

        assert_eq!(bonds.len(), 1);
        assert_eq!(bonds.get(0), Some((0, 1, 1)));
    }

    #[test]
    fn test_unit_cell_cubic() {
        let cell = UnitCell::from_lengths_angles(10.0, 10.0, 10.0, 90.0, 90.0, 90.0);
        let volume = cell.volume();

        // Cubic cell: volume = a³
        assert!((volume - 1000.0).abs() < 0.1);
    }
}
