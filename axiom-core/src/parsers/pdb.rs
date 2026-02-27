// PDB file parser
//
// PDB format specification:
// ATOM/HETATM records contain atomic coordinates
// Columns:
// 1-6:   Record name (ATOM/HETATM)
// 7-11:  Atom serial number
// 13-16: Atom name
// 17:    Alternate location indicator
// 18-20: Residue name
// 22:    Chain identifier
// 23-26: Residue sequence number
// 31-38: X coordinate (Angstroms)
// 39-46: Y coordinate (Angstroms)
// 47-54: Z coordinate (Angstroms)
// 77-78: Element symbol (right-justified)

use crate::atoms::{Atoms, Bonds};
use crate::errors::{AxiomError, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Parse PDB file
pub fn parse_pdb<P: AsRef<Path>>(path: P) -> Result<Atoms> {
    let file = File::open(path.as_ref())
        .map_err(|_| AxiomError::FileNotFound(path.as_ref().display().to_string()))?;
    let reader = BufReader::new(file);
    parse_pdb_reader(reader)
}

/// Parse PDB from a buffered reader
pub fn parse_pdb_reader<R: BufRead>(reader: R) -> Result<Atoms> {
    let mut atoms = Atoms::new();
    let mut residue_names = Vec::new();
    let mut chain_ids = Vec::new();
    let mut residue_indices = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;

        // Only process ATOM and HETATM records
        if !line.starts_with("ATOM") && !line.starts_with("HETATM") {
            continue;
        }

        // PDB files have fixed-width columns, so we need at least 54 characters
        if line.len() < 54 {
            return Err(AxiomError::ParseError(format!(
                "Line {}: ATOM/HETATM record too short (need at least 54 chars)",
                line_num + 1
            )));
        }

        // Extract residue name (columns 18-20)
        let resname = line.get(17..20)
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "UNK".to_string());

        // Extract chain ID (column 22)
        let chain = line.get(21..22)
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| " ".to_string());

        // Extract residue sequence number (columns 23-26)
        let resid_str = line.get(22..26).unwrap_or("    ");
        let resid: u32 = resid_str.trim().parse().unwrap_or(0);

        // Extract coordinates (columns 31-38, 39-46, 47-54)
        let x_str = line.get(30..38).ok_or_else(|| {
            AxiomError::ParseError(format!("Line {}: cannot extract X coordinate", line_num + 1))
        })?;
        let y_str = line.get(38..46).ok_or_else(|| {
            AxiomError::ParseError(format!("Line {}: cannot extract Y coordinate", line_num + 1))
        })?;
        let z_str = line.get(46..54).ok_or_else(|| {
            AxiomError::ParseError(format!("Line {}: cannot extract Z coordinate", line_num + 1))
        })?;

        let x: f32 = x_str.trim().parse().map_err(|_| {
            AxiomError::ParseError(format!("Line {}: invalid X coordinate '{}'", line_num + 1, x_str.trim()))
        })?;
        let y: f32 = y_str.trim().parse().map_err(|_| {
            AxiomError::ParseError(format!("Line {}: invalid Y coordinate '{}'", line_num + 1, y_str.trim()))
        })?;
        let z: f32 = z_str.trim().parse().map_err(|_| {
            AxiomError::ParseError(format!("Line {}: invalid Z coordinate '{}'", line_num + 1, z_str.trim()))
        })?;

        // Try to extract element symbol from columns 77-78 (PDB v3.0+)
        let element_symbol = if line.len() >= 78 {
            let elem = line.get(76..78).unwrap_or("").trim();
            if !elem.is_empty() {
                elem
            } else {
                // If element column is empty, fallback to atom name
                extract_element_from_atom_name(&line)
            }
        } else {
            // Fallback: extract from atom name (columns 13-16)
            extract_element_from_atom_name(&line)
        };

        let element = symbol_to_atomic_number(element_symbol);
        atoms.push(x, y, z, element);

        // Store metadata
        residue_names.push(resname);
        chain_ids.push(chain);
        residue_indices.push(resid);
    }

    if atoms.len() == 0 {
        return Err(AxiomError::ParseError(
            "No ATOM or HETATM records found in PDB file".to_string(),
        ));
    }

    // Attach metadata to atoms
    atoms.residue_names = Some(residue_names);
    atoms.chain_ids = Some(chain_ids);
    atoms.residue_indices = Some(residue_indices);

    Ok(atoms)
}

/// Extract element symbol from atom name (columns 13-16)
/// PDB atom naming conventions:
/// - First character is often the element (e.g., "C   ", "N   ", "O   ")
/// - Two-letter elements are left-aligned (e.g., "CA  ", "CB  ", "FE  ")
/// - Special cases: "CA" (alpha carbon), "CB" (beta carbon) are carbon, not calcium
fn extract_element_from_atom_name(line: &str) -> &str {
    let atom_name = line.get(12..16).unwrap_or("").trim();

    if atom_name.is_empty() {
        return "";
    }

    // Handle common protein atom names that are carbon
    if atom_name.starts_with('C') {
        // CA, CB, CG, CD, CE, CZ are all carbon atoms in proteins
        return "C";
    }

    // Handle common nitrogen atoms
    if atom_name.starts_with('N') {
        return "N";
    }

    // Handle oxygen
    if atom_name.starts_with('O') {
        return "O";
    }

    // Handle sulfur
    if atom_name.starts_with('S') {
        return "S";
    }

    // For other cases, take first character if it's alphabetic
    let first_char = atom_name.chars().next().unwrap();
    if first_char.is_alphabetic() {
        // Return first character as a static str (we need a way to return &str)
        // For now, return the whole atom_name and let the caller handle it
        &atom_name[0..1]
    } else {
        ""
    }
}

/// Convert element symbol to atomic number
fn symbol_to_atomic_number(symbol: &str) -> u8 {
    // Handle both single and double character symbols
    let symbol_upper = symbol.to_uppercase();
    match symbol_upper.as_str() {
        "H" => 1,
        "HE" => 2,
        "LI" => 3,
        "BE" => 4,
        "B" => 5,
        "C" => 6,
        "N" => 7,
        "O" => 8,
        "F" => 9,
        "NE" => 10,
        "NA" => 11,
        "MG" => 12,
        "AL" => 13,
        "SI" => 14,
        "P" => 15,
        "S" => 16,
        "CL" => 17,
        "AR" => 18,
        "K" => 19,
        "CA" => 20,
        "SC" => 21,
        "TI" => 22,
        "V" => 23,
        "CR" => 24,
        "MN" => 25,
        "FE" => 26,
        "CO" => 27,
        "NI" => 28,
        "CU" => 29,
        "ZN" => 30,
        "BR" => 35,
        "AG" => 47,
        "I" => 53,
        "AU" => 79,
        // Add more as needed
        _ => 0, // Unknown element
    }
}

/// Parse PDB file with CONECT records for bonds
pub fn parse_pdb_with_bonds<P: AsRef<Path>>(path: P) -> Result<(Atoms, Bonds)> {
    let file = File::open(path.as_ref())
        .map_err(|_| AxiomError::FileNotFound(path.as_ref().display().to_string()))?;
    let reader = BufReader::new(file);
    parse_pdb_with_bonds_reader(reader)
}

/// Parse PDB from a buffered reader with CONECT records
pub fn parse_pdb_with_bonds_reader<R: BufRead>(reader: R) -> Result<(Atoms, Bonds)> {
    let mut atoms = Atoms::new();
    let mut bonds = Bonds::new();
    let mut residue_names = Vec::new();
    let mut chain_ids = Vec::new();
    let mut residue_indices = Vec::new();

    // Map PDB serial numbers to atom indices (0-based)
    let mut serial_to_index: HashMap<u32, u32> = HashMap::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;

        // Process ATOM and HETATM records
        if line.starts_with("ATOM") || line.starts_with("HETATM") {
            if line.len() < 54 {
                return Err(AxiomError::ParseError(format!(
                    "Line {}: ATOM/HETATM record too short (need at least 54 chars)",
                    line_num + 1
                )));
            }

            // Extract atom serial number (columns 7-11)
            let serial_str = line.get(6..11).ok_or_else(|| {
                AxiomError::ParseError(format!("Line {}: cannot extract serial number", line_num + 1))
            })?;
            let serial: u32 = serial_str.trim().parse().map_err(|_| {
                AxiomError::ParseError(format!("Line {}: invalid serial number '{}'", line_num + 1, serial_str.trim()))
            })?;

            // Store mapping from serial to index
            serial_to_index.insert(serial, atoms.len() as u32);

            // Extract residue name (columns 18-20)
            let resname = line.get(17..20)
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "UNK".to_string());

            // Extract chain ID (column 22)
            let chain = line.get(21..22)
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| " ".to_string());

            // Extract residue sequence number (columns 23-26)
            let resid_str = line.get(22..26).unwrap_or("    ");
            let resid: u32 = resid_str.trim().parse().unwrap_or(0);

            // Extract coordinates (columns 31-38, 39-46, 47-54)
            let x_str = line.get(30..38).ok_or_else(|| {
                AxiomError::ParseError(format!("Line {}: cannot extract X coordinate", line_num + 1))
            })?;
            let y_str = line.get(38..46).ok_or_else(|| {
                AxiomError::ParseError(format!("Line {}: cannot extract Y coordinate", line_num + 1))
            })?;
            let z_str = line.get(46..54).ok_or_else(|| {
                AxiomError::ParseError(format!("Line {}: cannot extract Z coordinate", line_num + 1))
            })?;

            let x: f32 = x_str.trim().parse().map_err(|_| {
                AxiomError::ParseError(format!("Line {}: invalid X coordinate '{}'", line_num + 1, x_str.trim()))
            })?;
            let y: f32 = y_str.trim().parse().map_err(|_| {
                AxiomError::ParseError(format!("Line {}: invalid Y coordinate '{}'", line_num + 1, y_str.trim()))
            })?;
            let z: f32 = z_str.trim().parse().map_err(|_| {
                AxiomError::ParseError(format!("Line {}: invalid Z coordinate '{}'", line_num + 1, z_str.trim()))
            })?;

            // Extract element symbol
            let element_symbol = if line.len() >= 78 {
                let elem = line.get(76..78).unwrap_or("").trim();
                if !elem.is_empty() {
                    elem
                } else {
                    extract_element_from_atom_name(&line)
                }
            } else {
                extract_element_from_atom_name(&line)
            };

            let element = symbol_to_atomic_number(element_symbol);
            atoms.push(x, y, z, element);

            // Store metadata
            residue_names.push(resname);
            chain_ids.push(chain);
            residue_indices.push(resid);
        }
        // Process CONECT records
        else if line.starts_with("CONECT") {
            // CONECT format: CONECT serial1 serial2 serial3 serial4 ...
            // Columns 7-11: first atom serial
            // Columns 12-16, 17-21, 22-26, 27-31: bonded atom serials

            // Parse the line - serials are in fixed columns
            let parts: Vec<&str> = line[6..].split_whitespace().collect();

            if parts.is_empty() {
                continue;
            }

            // First number is the atom
            let atom1_serial: u32 = parts[0].parse().unwrap_or(0);
            if atom1_serial == 0 {
                continue;
            }

            // Get 0-based index for atom1
            let atom1_index = match serial_to_index.get(&atom1_serial) {
                Some(&idx) => idx,
                None => continue, // Skip if atom not found
            };

            // Rest are bonded atoms
            for &bonded_serial_str in &parts[1..] {
                let bonded_serial: u32 = match bonded_serial_str.parse() {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                // Get 0-based index for bonded atom
                let atom2_index = match serial_to_index.get(&bonded_serial) {
                    Some(&idx) => idx,
                    None => continue,
                };

                // Add bond only if atom1 < atom2 (avoid duplicates)
                if atom1_index < atom2_index {
                    bonds.push(atom1_index, atom2_index, 1); // Default to single bond
                }
            }
        }
    }

    if atoms.len() == 0 {
        return Err(AxiomError::ParseError(
            "No ATOM or HETATM records found in PDB file".to_string(),
        ));
    }

    // Attach metadata to atoms
    atoms.residue_names = Some(residue_names);
    atoms.chain_ids = Some(chain_ids);
    atoms.residue_indices = Some(residue_indices);

    Ok((atoms, bonds))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_pdb_simple() {
        // Simple PDB with 3 atoms (water molecule)
        let pdb_data = "\
ATOM      1  O   WAT A   1       0.000   0.000   0.000  1.00  0.00           O
ATOM      2  H1  WAT A   1       0.757   0.586   0.000  1.00  0.00           H
ATOM      3  H2  WAT A   1      -0.757   0.586   0.000  1.00  0.00           H
END
";
        let cursor = Cursor::new(pdb_data);
        let atoms = parse_pdb_reader(BufReader::new(cursor)).unwrap();

        assert_eq!(atoms.len(), 3);
        assert_eq!(atoms.element(0), Some(8));  // O
        assert_eq!(atoms.element(1), Some(1));  // H
        assert_eq!(atoms.element(2), Some(1));  // H
        assert_eq!(atoms.position(0), Some([0.0, 0.0, 0.0]));
        assert_eq!(atoms.position(1), Some([0.757, 0.586, 0.0]));
    }

    #[test]
    fn test_parse_pdb_without_element_column() {
        // PDB without element symbol in columns 77-78 (old format)
        let pdb_data = "\
ATOM      1  CA  ALA A   1      10.000  20.000  30.000  1.00  0.00
ATOM      2  C   ALA A   1      11.000  21.000  31.000  1.00  0.00
";
        let cursor = Cursor::new(pdb_data);
        let atoms = parse_pdb_reader(BufReader::new(cursor)).unwrap();

        assert_eq!(atoms.len(), 2);
        assert_eq!(atoms.element(0), Some(6));  // C (from CA)
        assert_eq!(atoms.element(1), Some(6));  // C
        assert_eq!(atoms.position(0), Some([10.0, 20.0, 30.0]));
    }

    #[test]
    fn test_parse_pdb_empty() {
        let pdb_data = "HEADER    TEST\nEND\n";
        let cursor = Cursor::new(pdb_data);
        let result = parse_pdb_reader(BufReader::new(cursor));

        assert!(result.is_err());
    }

    #[test]
    fn test_symbol_to_atomic_number() {
        assert_eq!(symbol_to_atomic_number("H"), 1);
        assert_eq!(symbol_to_atomic_number("C"), 6);
        assert_eq!(symbol_to_atomic_number("O"), 8);
        assert_eq!(symbol_to_atomic_number("Fe"), 26);
        assert_eq!(symbol_to_atomic_number("AU"), 79);
        assert_eq!(symbol_to_atomic_number("ca"), 20);  // Test case insensitivity
        assert_eq!(symbol_to_atomic_number("Unknown"), 0);
    }
}
