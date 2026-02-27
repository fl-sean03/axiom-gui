// Bond computation algorithms
//
// Computes bonds between atoms based on distances and covalent radii

use crate::atoms::{Atoms, Bonds};

/// Covalent radii for elements (in Angstroms)
/// Source: Cordero et al. (2008) "Covalent radii revisited"
const COVALENT_RADII: [f32; 119] = [
    0.00, // 0: placeholder
    0.31, // 1: H
    0.28, // 2: He
    1.28, // 3: Li
    0.96, // 4: Be
    0.84, // 5: B
    0.76, // 6: C (sp3)
    0.71, // 7: N
    0.66, // 8: O
    0.57, // 9: F
    0.58, // 10: Ne
    1.66, // 11: Na
    1.41, // 12: Mg
    1.21, // 13: Al
    1.11, // 14: Si
    1.07, // 15: P
    1.05, // 16: S
    1.02, // 17: Cl
    1.06, // 18: Ar
    2.03, // 19: K
    1.76, // 20: Ca
    1.70, // 21: Sc
    1.60, // 22: Ti
    1.53, // 23: V
    1.39, // 24: Cr
    1.39, // 25: Mn
    1.32, // 26: Fe
    1.26, // 27: Co
    1.24, // 28: Ni
    1.32, // 29: Cu
    1.22, // 30: Zn
    // Fill in more as needed, for now use 1.5 as default
    1.22, 1.20, 1.19, 1.20, 1.20, 1.16, // 31-36
    2.20, 1.95, 1.90, 1.75, 1.64, 1.54, // 37-42
    1.47, 1.46, 1.42, 1.39, 1.45, 1.44, // 43-48
    1.42, 1.39, 1.39, 1.38, 1.39, 1.40, // 49-54
    2.44, 2.15, 2.07, 2.04, 2.03, 2.01, // 55-60
    1.99, 1.98, 1.98, 1.96, 1.94, 1.92, // 61-66
    1.92, 1.89, 1.90, 1.87, 1.87, 1.75, // 67-72
    1.70, 1.62, 1.51, 1.44, 1.41, 1.36, // 73-78
    1.36, 1.32, 1.45, 1.46, 1.48, 1.40, // 79-84
    1.50, 1.50, 2.60, 2.21, 2.15, 2.06, // 85-90
    2.00, 1.96, 1.90, 1.87, 1.80, 1.69, // 91-96
    1.50, 1.50, 1.50, 1.50, 1.50, 1.50, // 97-102
    1.50, 1.50, 1.50, 1.50, 1.50, 1.50, // 103-108
    1.50, 1.50, 1.50, 1.50, 1.50, 1.50, // 109-114
    1.50, 1.50, 1.50, 1.50, // 115-118
];

/// Get covalent radius for an element
fn covalent_radius(element: u8) -> f32 {
    if (element as usize) < COVALENT_RADII.len() {
        COVALENT_RADII[element as usize]
    } else {
        1.5 // Default radius for unknown elements
    }
}

/// Compute bonds between atoms based on distances
///
/// A bond is created if the distance between two atoms is less than
/// the sum of their covalent radii multiplied by a tolerance factor.
///
/// # Arguments
/// * `atoms` - The atomic structure
/// * `tolerance` - Multiplier for covalent radii sum (default: 1.2)
/// * `max_distance` - Maximum distance to consider for bonding (optimization)
///
/// # Returns
/// A Bonds structure containing all detected bonds
pub fn compute_bonds(atoms: &Atoms, tolerance: f32, max_distance: f32) -> Bonds {
    let n = atoms.len();
    let mut bonds = Bonds::new();

    // Estimate capacity (rough heuristic: ~2-3 bonds per atom on average)
    bonds.atom1.reserve(n * 2);
    bonds.atom2.reserve(n * 2);
    bonds.order.reserve(n * 2);

    // Simple O(n^2) algorithm
    // TODO: Optimize with spatial hashing or cell lists for large systems
    for i in 0..n {
        let x1 = atoms.x[i];
        let y1 = atoms.y[i];
        let z1 = atoms.z[i];
        let elem1 = atoms.elements[i];
        let r1 = covalent_radius(elem1);

        for j in (i + 1)..n {
            let x2 = atoms.x[j];
            let y2 = atoms.y[j];
            let z2 = atoms.z[j];
            let elem2 = atoms.elements[j];
            let r2 = covalent_radius(elem2);

            // Compute distance
            let dx = x2 - x1;
            let dy = y2 - y1;
            let dz = z2 - z1;
            let dist_sq = dx * dx + dy * dy + dz * dz;

            // Quick rejection if too far
            if dist_sq > max_distance * max_distance {
                continue;
            }

            let dist = dist_sq.sqrt();

            // Bond threshold: sum of covalent radii * tolerance
            let bond_threshold = (r1 + r2) * tolerance;

            if dist < bond_threshold {
                bonds.atom1.push(i as u32);
                bonds.atom2.push(j as u32);
                bonds.order.push(1); // Default to single bond
            }
        }
    }

    bonds
}

/// Compute bonds with default parameters
///
/// Uses tolerance=1.2 and max_distance=3.0 Angstroms
pub fn compute_bonds_default(atoms: &Atoms) -> Bonds {
    compute_bonds(atoms, 1.2, 3.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_covalent_radius() {
        assert_eq!(covalent_radius(1), 0.31);  // H
        assert_eq!(covalent_radius(6), 0.76);  // C
        assert_eq!(covalent_radius(8), 0.66);  // O
        assert_eq!(covalent_radius(26), 1.32); // Fe
    }

    #[test]
    fn test_compute_bonds_water() {
        // Water molecule: O at origin, H atoms at bonding distance
        let mut atoms = Atoms::new();
        atoms.push(0.0, 0.0, 0.0, 8);     // O
        atoms.push(0.96, 0.0, 0.0, 1);    // H (typical O-H bond: ~0.96 Å)
        atoms.push(-0.24, 0.93, 0.0, 1);  // H

        let bonds = compute_bonds_default(&atoms);

        // Should have 2 bonds: O-H1 and O-H2
        assert_eq!(bonds.len(), 2);
        assert_eq!(bonds.atom1[0], 0); // O
        assert_eq!(bonds.atom2[0], 1); // H
        assert_eq!(bonds.atom1[1], 0); // O
        assert_eq!(bonds.atom2[1], 2); // H
        assert_eq!(bonds.order[0], 1);
        assert_eq!(bonds.order[1], 1);
    }

    #[test]
    fn test_compute_bonds_no_bonds() {
        // Two atoms far apart - no bonds
        let mut atoms = Atoms::new();
        atoms.push(0.0, 0.0, 0.0, 6);  // C
        atoms.push(10.0, 0.0, 0.0, 6); // C (10 Å away)

        let bonds = compute_bonds_default(&atoms);

        assert_eq!(bonds.len(), 0);
    }

    #[test]
    fn test_compute_bonds_ethane() {
        // Ethane: C-C bond (~1.54 Å) with H atoms
        let mut atoms = Atoms::new();
        atoms.push(0.0, 0.0, 0.0, 6);     // C1
        atoms.push(1.54, 0.0, 0.0, 6);    // C2
        atoms.push(-0.5, 0.87, 0.0, 1);   // H
        atoms.push(-0.5, -0.87, 0.0, 1);  // H
        atoms.push(2.04, 0.87, 0.0, 1);   // H
        atoms.push(2.04, -0.87, 0.0, 1);  // H

        let bonds = compute_bonds_default(&atoms);

        // Should have: 1 C-C bond + multiple C-H bonds
        assert!(bonds.len() >= 1);

        // Check that C-C bond exists
        let has_cc_bond = bonds.atom1.iter().zip(bonds.atom2.iter())
            .any(|(&a1, &a2)| (a1 == 0 && a2 == 1) || (a1 == 1 && a2 == 0));
        assert!(has_cc_bond);
    }
}
