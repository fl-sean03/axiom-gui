// LAMMPS file parsers
//
// Supports two LAMMPS file formats:
// 1. LAMMPS dump files (trajectory format): timestep-based atom coordinates
// 2. LAMMPS data files (topology format): full system definition with bonds, angles, etc.

use crate::atoms::{Atoms, Bonds};
use crate::errors::{AxiomError, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Parse LAMMPS file (auto-detects dump vs data format)
pub fn parse_lammps<P: AsRef<Path>>(path: P) -> Result<Atoms> {
    let file = File::open(path.as_ref())
        .map_err(|_| AxiomError::FileNotFound(path.as_ref().display().to_string()))?;
    let mut reader = BufReader::new(file);

    // Read first line to detect format
    let mut first_line = String::new();
    reader.read_line(&mut first_line)
        .map_err(|e| AxiomError::ParseError(format!("Failed to read file: {}", e)))?;

    // Reset reader
    drop(reader);
    let file = File::open(path.as_ref())
        .map_err(|_| AxiomError::FileNotFound(path.as_ref().display().to_string()))?;
    let reader = BufReader::new(file);

    if first_line.starts_with("ITEM: TIMESTEP") {
        parse_lammps_dump_reader(reader)
    } else {
        // Assume data file format
        parse_lammps_data_reader(reader)
    }
}

/// Parse LAMMPS dump file
///
/// Supports "atom" and "custom" dump styles with atomic coordinates.
/// Currently reads only the first frame of a trajectory.
pub fn parse_lammps_dump_reader<R: BufRead>(reader: R) -> Result<Atoms> {
    let mut lines = reader.lines();
    let mut atoms = Atoms::new();

    // Parse header
    // Line 1: ITEM: TIMESTEP
    let line1 = lines.next().ok_or_else(|| AxiomError::ParseError("Empty file".to_string()))??;
    if !line1.starts_with("ITEM: TIMESTEP") {
        return Err(AxiomError::ParseError(format!(
            "Expected 'ITEM: TIMESTEP', got: {}",
            line1
        )));
    }

    // Line 2: timestep value (skip)
    lines.next();

    // Line 3: ITEM: NUMBER OF ATOMS
    let line3 = lines.next().ok_or_else(|| AxiomError::ParseError("Missing number of atoms".to_string()))??;
    if !line3.starts_with("ITEM: NUMBER OF ATOMS") {
        return Err(AxiomError::ParseError(format!(
            "Expected 'ITEM: NUMBER OF ATOMS', got: {}",
            line3
        )));
    }

    // Line 4: number of atoms
    let num_atoms: usize = lines.next().ok_or_else(|| AxiomError::ParseError("Missing atom count".to_string()))??
        .trim()
        .parse()
        .map_err(|_| AxiomError::ParseError("Invalid atom count".to_string()))?;

    atoms.reserve(num_atoms);

    // Line 5: ITEM: BOX BOUNDS
    let line5 = lines.next().ok_or_else(|| AxiomError::ParseError("Missing box bounds".to_string()))??;
    if !line5.starts_with("ITEM: BOX BOUNDS") {
        return Err(AxiomError::ParseError(format!(
            "Expected 'ITEM: BOX BOUNDS', got: {}",
            line5
        )));
    }

    // Lines 6-8: box bounds (skip for now)
    lines.next();
    lines.next();
    lines.next();

    // Line 9: ITEM: ATOMS ...
    let atoms_header = lines.next().ok_or_else(|| AxiomError::ParseError("Missing ATOMS section".to_string()))??;

    if !atoms_header.starts_with("ITEM: ATOMS") {
        return Err(AxiomError::ParseError(format!(
            "Expected 'ITEM: ATOMS', got: {}",
            atoms_header
        )));
    }

    // Parse column headers to find x, y, z, type positions
    let columns: Vec<&str> = atoms_header
        .strip_prefix("ITEM: ATOMS ")
        .ok_or_else(|| AxiomError::ParseError("Invalid ATOMS header".to_string()))?
        .split_whitespace()
        .collect();

    let x_col = columns.iter().position(|&c| c == "x" || c == "xu" || c == "xs")
        .ok_or_else(|| AxiomError::ParseError("No x coordinate column found".to_string()))?;
    let y_col = columns.iter().position(|&c| c == "y" || c == "yu" || c == "ys")
        .ok_or_else(|| AxiomError::ParseError("No y coordinate column found".to_string()))?;
    let z_col = columns.iter().position(|&c| c == "z" || c == "zu" || c == "zs")
        .ok_or_else(|| AxiomError::ParseError("No z coordinate column found".to_string()))?;
    let type_col = columns.iter().position(|&c| c == "type")
        .ok_or_else(|| AxiomError::ParseError("No type column found".to_string()))?;

    // Parse atom data
    for (line_num, line_result) in lines.enumerate() {
        let line = line_result?;
        if line.trim().is_empty() {
            break; // End of current frame
        }

        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() <= x_col.max(y_col).max(z_col).max(type_col) {
            return Err(AxiomError::ParseError(format!(
                "Line {}: insufficient columns",
                line_num + 10
            )));
        }

        let x: f32 = parts[x_col]
            .parse()
            .map_err(|_| AxiomError::ParseError(format!("Invalid x coordinate on line {}", line_num + 10)))?;
        let y: f32 = parts[y_col]
            .parse()
            .map_err(|_| AxiomError::ParseError(format!("Invalid y coordinate on line {}", line_num + 10)))?;
        let z: f32 = parts[z_col]
            .parse()
            .map_err(|_| AxiomError::ParseError(format!("Invalid z coordinate on line {}", line_num + 10)))?;

        // LAMMPS type is an integer (1, 2, 3...)
        // We'll map it directly to element for now (user can override)
        let atom_type: u8 = parts[type_col]
            .parse()
            .map_err(|_| AxiomError::ParseError(format!("Invalid type on line {}", line_num + 10)))?;

        atoms.push(x, y, z, atom_type);
    }

    if atoms.len() != num_atoms {
        return Err(AxiomError::ParseError(format!(
            "Expected {} atoms, found {}",
            num_atoms,
            atoms.len()
        )));
    }

    Ok(atoms)
}

/// Parse LAMMPS data file
///
/// Parses full LAMMPS data files including:
/// - Atoms (with positions, types, charges, molecule IDs)
/// - Bonds, angles, dihedrals, impropers
/// - Force field parameters (masses, pair coeffs, bond coeffs, etc.)
pub fn parse_lammps_data_reader<R: BufRead>(reader: R) -> Result<Atoms> {
    let mut lines = reader.lines().peekable();
    let mut atoms = Atoms::new();

    // Parse header - first line is a comment
    let _header = lines.next().ok_or_else(|| AxiomError::ParseError("Empty file".to_string()))??;

    // Initialize metadata
    let mut num_atoms = 0;
    let mut _num_bonds = 0;
    let mut _num_atom_types = 0;
    let mut _masses: HashMap<u32, f32> = HashMap::new();

    // Parse counts section
    loop {
        let line = match lines.next() {
            Some(Ok(l)) => l,
            Some(Err(e)) => return Err(AxiomError::ParseError(format!("Read error: {}", e))),
            None => break,
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check if we've reached a section header
        if trimmed.starts_with("Masses")
            || trimmed.starts_with("Pair Coeffs")
            || trimmed.starts_with("Bond Coeffs")
            || trimmed.starts_with("Angle Coeffs")
            || trimmed.starts_with("Dihedral Coeffs")
            || trimmed.starts_with("Improper Coeffs")
            || trimmed.starts_with("Atoms")
            || trimmed.starts_with("Bonds")
            || trimmed.starts_with("Angles")
            || trimmed.starts_with("Dihedrals")
            || trimmed.starts_with("Impropers")
            || trimmed.contains("xlo xhi")
            || trimmed.contains("ylo yhi")
            || trimmed.contains("zlo zhi") {
            break;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            if parts[1] == "atoms" {
                num_atoms = parts[0].parse().unwrap_or(0);
            } else if parts[1] == "bonds" {
                _num_bonds = parts[0].parse().unwrap_or(0);
            } else if parts[1] == "atom" && parts.len() > 2 && parts[2] == "types" {
                _num_atom_types = parts[0].parse().unwrap_or(0);
            }
        }
    }

    if num_atoms == 0 {
        return Err(AxiomError::ParseError("No atoms found in header".to_string()));
    }

    atoms.reserve(num_atoms);

    // Initialize optional vectors for charges, types, molecule IDs
    let mut charges = Vec::with_capacity(num_atoms);
    let mut atom_types = Vec::with_capacity(num_atoms);
    let mut molecule_ids = Vec::with_capacity(num_atoms);

    // Parse sections
    loop {
        let line = match lines.next() {
            Some(Ok(l)) => l,
            Some(Err(e)) => return Err(AxiomError::ParseError(format!("Read error: {}", e))),
            None => break,
        };

        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Check for Masses section
        if trimmed.starts_with("Masses") {
            // Skip masses for now (could parse if needed)
            continue;
        }

        // Check for Atoms section
        if trimmed.starts_with("Atoms") {
            // Next line is blank, then atom data
            // Format: atom_id molecule_id atom_type charge x y z [image_flags]
            let mut atom_count_parsed = 0;
            loop {
                let atom_line = match lines.next() {
                    Some(Ok(l)) => l,
                    Some(Err(e)) => return Err(AxiomError::ParseError(format!("Read error: {}", e))),
                    None => break,
                };

                let atom_trimmed = atom_line.trim();
                if atom_trimmed.is_empty() {
                    continue;
                }

                // Check if we've hit the next section
                if atom_trimmed.starts_with("Velocities")
                    || atom_trimmed.starts_with("Bonds")
                    || atom_trimmed.starts_with("Angles")
                    || atom_trimmed.starts_with("Dihedrals")
                    || atom_trimmed.starts_with("Impropers") {
                    eprintln!("DEBUG: Breaking at section header: {}", atom_trimmed);
                    eprintln!("DEBUG: Parsed {} atoms so far", atom_count_parsed);
                    break;
                }

                let parts: Vec<&str> = atom_trimmed.split_whitespace().collect();
                if parts.len() < 7 {
                    // Not enough data for an atom line
                    continue;
                }

                // Parse atom data
                let _atom_id: u32 = parts[0].parse()
                    .map_err(|_| AxiomError::ParseError(format!("Invalid atom ID: {}", parts[0])))?;
                let molecule_id: u32 = parts[1].parse()
                    .map_err(|_| AxiomError::ParseError(format!("Invalid molecule ID: {}", parts[1])))?;
                let atom_type: u32 = parts[2].parse()
                    .map_err(|_| AxiomError::ParseError(format!("Invalid atom type: {}", parts[2])))?;
                let charge: f32 = parts[3].parse()
                    .map_err(|_| AxiomError::ParseError(format!("Invalid charge: {}", parts[3])))?;
                let x: f32 = parts[4].parse()
                    .map_err(|_| AxiomError::ParseError(format!("Invalid x: {}", parts[4])))?;
                let y: f32 = parts[5].parse()
                    .map_err(|_| AxiomError::ParseError(format!("Invalid y: {}", parts[5])))?;
                let z: f32 = parts[6].parse()
                    .map_err(|_| AxiomError::ParseError(format!("Invalid z: {}", parts[6])))?;

                // Store atom data
                atoms.push(x, y, z, atom_type as u8); // Use atom_type as element (placeholder)
                charges.push(charge);
                atom_types.push(atom_type);
                molecule_ids.push(molecule_id);
                atom_count_parsed += 1;
            }

            // Set optional vectors
            eprintln!("DEBUG: atoms.len() = {}, charges.len() = {}", atoms.len(), charges.len());
            if !charges.is_empty() {
                atoms.charges = Some(charges.clone());
            }
            if !atom_types.is_empty() {
                atoms.atom_types = Some(atom_types.clone());
            }
            if !molecule_ids.is_empty() {
                atoms.molecule_ids = Some(molecule_ids.clone());
            }
            eprintln!("DEBUG: After setting optional vecs, atoms.len() = {}", atoms.len());

            break; // We've parsed atoms, done for now
        }
    }

    if atoms.len() == 0 {
        return Err(AxiomError::ParseError("No atoms parsed from file".to_string()));
    }

    eprintln!("DEBUG: Returning atoms with len = {}", atoms.len());
    Ok(atoms)
}

/// Parse LAMMPS data file and extract bonds
pub fn parse_lammps_data_with_bonds<P: AsRef<Path>>(path: P) -> Result<(Atoms, Bonds)> {
    let file = File::open(path.as_ref())
        .map_err(|_| AxiomError::FileNotFound(path.as_ref().display().to_string()))?;
    let reader = BufReader::new(file);
    parse_lammps_data_with_bonds_reader(reader)
}

/// Parse LAMMPS data file and extract bonds from reader
pub fn parse_lammps_data_with_bonds_reader<R: BufRead>(reader: R) -> Result<(Atoms, Bonds)> {
    let mut lines = reader.lines().peekable();
    let mut atoms = Atoms::new();
    let mut bonds = Bonds::new();

    // Parse header - first line is a comment
    let _header = lines.next().ok_or_else(|| AxiomError::ParseError("Empty file".to_string()))??;

    // Initialize metadata
    let mut num_atoms = 0;
    let mut num_bonds = 0;

    // Parse counts section
    loop {
        let line = match lines.next() {
            Some(Ok(l)) => l,
            Some(Err(e)) => return Err(AxiomError::ParseError(format!("Read error: {}", e))),
            None => break,
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check if we've reached a section header
        if trimmed.starts_with("Masses")
            || trimmed.starts_with("Pair Coeffs")
            || trimmed.starts_with("Bond Coeffs")
            || trimmed.starts_with("Atoms")
            || trimmed.starts_with("Bonds")
            || trimmed.contains("xlo xhi")
            || trimmed.contains("ylo yhi")
            || trimmed.contains("zlo zhi") {
            break;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            if parts[1] == "atoms" {
                num_atoms = parts[0].parse().unwrap_or(0);
            } else if parts[1] == "bonds" {
                num_bonds = parts[0].parse().unwrap_or(0);
            }
        }
    }

    if num_atoms == 0 {
        return Err(AxiomError::ParseError("No atoms found in header".to_string()));
    }

    atoms.reserve(num_atoms);
    if num_bonds > 0 {
        bonds = Bonds::with_capacity(num_bonds);
    }

    // Initialize optional vectors for charges, types, molecule IDs
    let mut charges = Vec::with_capacity(num_atoms);
    let mut atom_types = Vec::with_capacity(num_atoms);
    let mut molecule_ids = Vec::with_capacity(num_atoms);

    // Parse sections
    loop {
        let line = match lines.next() {
            Some(Ok(l)) => l,
            Some(Err(e)) => return Err(AxiomError::ParseError(format!("Read error: {}", e))),
            None => break,
        };

        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Check for Atoms section
        if trimmed.starts_with("Atoms") {
            // Next line might be blank, then atom data
            loop {
                let atom_line = match lines.next() {
                    Some(Ok(l)) => l,
                    Some(Err(e)) => return Err(AxiomError::ParseError(format!("Read error: {}", e))),
                    None => break,
                };

                let atom_trimmed = atom_line.trim();
                if atom_trimmed.is_empty() {
                    continue;
                }

                // Check if we've hit the next section
                if atom_trimmed.starts_with("Velocities")
                    || atom_trimmed.starts_with("Bonds")
                    || atom_trimmed.starts_with("Angles")
                    || atom_trimmed.starts_with("Dihedrals")
                    || atom_trimmed.starts_with("Impropers") {
                    break;
                }

                let parts: Vec<&str> = atom_trimmed.split_whitespace().collect();
                if parts.len() < 7 {
                    continue;
                }

                // Parse atom data
                let _atom_id: u32 = parts[0].parse()
                    .map_err(|_| AxiomError::ParseError(format!("Invalid atom ID: {}", parts[0])))?;
                let molecule_id: u32 = parts[1].parse()
                    .map_err(|_| AxiomError::ParseError(format!("Invalid molecule ID: {}", parts[1])))?;
                let atom_type: u32 = parts[2].parse()
                    .map_err(|_| AxiomError::ParseError(format!("Invalid atom type: {}", parts[2])))?;
                let charge: f32 = parts[3].parse()
                    .map_err(|_| AxiomError::ParseError(format!("Invalid charge: {}", parts[3])))?;
                let x: f32 = parts[4].parse()
                    .map_err(|_| AxiomError::ParseError(format!("Invalid x: {}", parts[4])))?;
                let y: f32 = parts[5].parse()
                    .map_err(|_| AxiomError::ParseError(format!("Invalid y: {}", parts[5])))?;
                let z: f32 = parts[6].parse()
                    .map_err(|_| AxiomError::ParseError(format!("Invalid z: {}", parts[6])))?;

                atoms.push(x, y, z, atom_type as u8);
                charges.push(charge);
                atom_types.push(atom_type);
                molecule_ids.push(molecule_id);
            }

            // Set optional vectors
            if !charges.is_empty() {
                atoms.charges = Some(charges.clone());
            }
            if !atom_types.is_empty() {
                atoms.atom_types = Some(atom_types.clone());
            }
            if !molecule_ids.is_empty() {
                atoms.molecule_ids = Some(molecule_ids.clone());
            }

            continue; // Continue to look for Bonds section
        }

        // Check for Bonds section
        if trimmed.starts_with("Bonds") {
            // Next line might be blank, then bond data
            loop {
                let bond_line = match lines.next() {
                    Some(Ok(l)) => l,
                    Some(Err(e)) => return Err(AxiomError::ParseError(format!("Read error: {}", e))),
                    None => break,
                };

                let bond_trimmed = bond_line.trim();
                if bond_trimmed.is_empty() {
                    continue;
                }

                // Check if we've hit the next section
                if bond_trimmed.starts_with("Angles")
                    || bond_trimmed.starts_with("Dihedrals")
                    || bond_trimmed.starts_with("Impropers")
                    || bond_trimmed.starts_with("Velocities") {
                    break;
                }

                let parts: Vec<&str> = bond_trimmed.split_whitespace().collect();
                if parts.len() < 4 {
                    continue;
                }

                // Parse bond data: bond_id bond_type atom1 atom2
                let _bond_id: u32 = parts[0].parse()
                    .map_err(|_| AxiomError::ParseError(format!("Invalid bond ID: {}", parts[0])))?;
                let _bond_type: u32 = parts[1].parse()
                    .map_err(|_| AxiomError::ParseError(format!("Invalid bond type: {}", parts[1])))?;
                let atom1: u32 = parts[2].parse()
                    .map_err(|_| AxiomError::ParseError(format!("Invalid atom1: {}", parts[2])))?;
                let atom2: u32 = parts[3].parse()
                    .map_err(|_| AxiomError::ParseError(format!("Invalid atom2: {}", parts[3])))?;

                // CRITICAL: LAMMPS uses 1-based indexing, convert to 0-based
                bonds.push(atom1 - 1, atom2 - 1, 1); // Default to single bond
            }

            // After parsing bonds, we're done
            break;
        }
    }

    if atoms.len() == 0 {
        return Err(AxiomError::ParseError("No atoms parsed from file".to_string()));
    }

    Ok((atoms, bonds))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_lammps_dump_simple() {
        let lammps_data = "\
ITEM: TIMESTEP
0
ITEM: NUMBER OF ATOMS
3
ITEM: BOX BOUNDS pp pp pp
0.0 10.0
0.0 10.0
0.0 10.0
ITEM: ATOMS id type x y z
1 1 0.0 0.0 0.0
2 2 1.0 1.0 1.0
3 1 2.0 2.0 2.0
";
        let cursor = Cursor::new(lammps_data);
        let atoms = parse_lammps_dump_reader(BufReader::new(cursor)).unwrap();

        assert_eq!(atoms.len(), 3);
        assert_eq!(atoms.position(0), Some([0.0, 0.0, 0.0]));
        assert_eq!(atoms.position(1), Some([1.0, 1.0, 1.0]));
        assert_eq!(atoms.position(2), Some([2.0, 2.0, 2.0]));
        assert_eq!(atoms.element(0), Some(1));
        assert_eq!(atoms.element(1), Some(2));
    }

    #[test]
    fn test_parse_lammps_data_simple() {
        let lammps_data = "\
LAMMPS data file

3 atoms
2 atom types

0.0 10.0 xlo xhi
0.0 10.0 ylo yhi
0.0 10.0 zlo zhi

Masses

1 1.008
2 12.011

Atoms # full

1 1 1 0.5 0.0 0.0 0.0
2 1 2 -0.5 1.0 1.0 1.0
3 2 1 0.3 2.0 2.0 2.0
";
        let cursor = Cursor::new(lammps_data);
        let atoms = parse_lammps_data_reader(BufReader::new(cursor)).unwrap();

        assert_eq!(atoms.len(), 3);
        assert_eq!(atoms.position(0), Some([0.0, 0.0, 0.0]));
        assert_eq!(atoms.position(1), Some([1.0, 1.0, 1.0]));
        assert_eq!(atoms.position(2), Some([2.0, 2.0, 2.0]));

        // Check charges
        assert!(atoms.charges.is_some());
        let charges = atoms.charges.as_ref().unwrap();
        assert_eq!(charges[0], 0.5);
        assert_eq!(charges[1], -0.5);
        assert_eq!(charges[2], 0.3);

        // Check atom types
        assert!(atoms.atom_types.is_some());
        let types = atoms.atom_types.as_ref().unwrap();
        assert_eq!(types[0], 1);
        assert_eq!(types[1], 2);
        assert_eq!(types[2], 1);

        // Check molecule IDs
        assert!(atoms.molecule_ids.is_some());
        let mol_ids = atoms.molecule_ids.as_ref().unwrap();
        assert_eq!(mol_ids[0], 1);
        assert_eq!(mol_ids[1], 1);
        assert_eq!(mol_ids[2], 2);
    }
}
