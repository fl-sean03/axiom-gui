// GROMACS GRO file parser
//
// GRO format (fixed-width columns):
// Line 1: Title/comment
// Line 2: Number of atoms
// Lines 3+: Atom data (fixed format)
//   Columns 1-5: Residue number
//   Columns 6-10: Residue name
//   Columns 11-15: Atom name
//   Columns 16-20: Atom number
//   Columns 21-28: X coordinate (nm)
//   Columns 29-36: Y coordinate (nm)
//   Columns 37-44: Z coordinate (nm)
//   Columns 45-52: X velocity (optional, nm/ps)
//   Columns 53-60: Y velocity (optional, nm/ps)
//   Columns 61-68: Z velocity (optional, nm/ps)
// Last line: Box vectors (3 or 9 values)

use crate::atoms::Atoms;
use crate::errors::{AxiomError, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Parse GROMACS GRO file
///
/// Coordinates are converted from nm to Angstroms (multiply by 10)
pub fn parse_gro<P: AsRef<Path>>(path: P) -> Result<Atoms> {
    let file = File::open(path.as_ref())
        .map_err(|_| AxiomError::FileNotFound(path.as_ref().display().to_string()))?;
    let reader = BufReader::new(file);
    parse_gro_reader(reader)
}

/// Parse GRO from a buffered reader
pub fn parse_gro_reader<R: BufRead>(reader: R) -> Result<Atoms> {
    let mut lines = reader.lines();

    // Line 1: Title (skip)
    let _title = lines.next().ok_or_else(|| AxiomError::ParseError("Empty file".to_string()))??;

    // Line 2: Number of atoms
    let num_atoms: usize = lines.next()
        .ok_or_else(|| AxiomError::ParseError("Missing atom count".to_string()))??
        .trim()
        .parse()
        .map_err(|_| AxiomError::ParseError("Invalid atom count".to_string()))?;

    let mut atoms = Atoms::with_capacity(num_atoms);

    // Parse atom lines
    for (line_num, line_result) in lines.enumerate().take(num_atoms) {
        let line = line_result?;

        // GRO format uses fixed-width columns
        if line.len() < 44 {
            return Err(AxiomError::ParseError(format!(
                "Line {}: too short (need at least 44 chars for coordinates)",
                line_num + 3
            )));
        }

        // Extract atom name (columns 11-15)
        let atom_name = line.get(10..15).unwrap_or("").trim();

        // Extract coordinates (columns 21-28, 29-36, 37-44)
        // GRO uses nm, convert to Angstroms (*10)
        let x_str = line.get(20..28).ok_or_else(|| {
            AxiomError::ParseError(format!("Line {}: cannot extract X coordinate", line_num + 3))
        })?;
        let y_str = line.get(28..36).ok_or_else(|| {
            AxiomError::ParseError(format!("Line {}: cannot extract Y coordinate", line_num + 3))
        })?;
        let z_str = line.get(36..44).ok_or_else(|| {
            AxiomError::ParseError(format!("Line {}: cannot extract Z coordinate", line_num + 3))
        })?;

        let x: f32 = x_str.trim().parse::<f32>().map_err(|_| {
            AxiomError::ParseError(format!("Line {}: invalid X coordinate '{}'", line_num + 3, x_str.trim()))
        })? * 10.0; // nm to Angstroms

        let y: f32 = y_str.trim().parse::<f32>().map_err(|_| {
            AxiomError::ParseError(format!("Line {}: invalid Y coordinate '{}'", line_num + 3, y_str.trim()))
        })? * 10.0;

        let z: f32 = z_str.trim().parse::<f32>().map_err(|_| {
            AxiomError::ParseError(format!("Line {}: invalid Z coordinate '{}'", line_num + 3, z_str.trim()))
        })? * 10.0;

        // Infer element from atom name
        let element = atom_name_to_element(atom_name);

        atoms.push(x, y, z, element);
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

/// Infer atomic number from GROMACS atom name
///
/// Common GROMACS atom names:
/// - C, CA, CB, CG, CD, CE, CZ -> Carbon
/// - N, NA, NB, NH, NZ -> Nitrogen
/// - O, OA, OW, OH -> Oxygen
/// - H, H1, H2, HW, HA -> Hydrogen
/// - S, SH -> Sulfur
fn atom_name_to_element(name: &str) -> u8 {
    if name.is_empty() {
        return 0;
    }

    // Check for multi-character patterns first (ions and special atoms)
    match name {
        n if n.starts_with("CL") => return 17, // Chlorine
        n if n.starts_with("FE") => return 26, // Iron
        n if n.starts_with("ZN") => return 30, // Zinc
        n if n.starts_with("MG") => return 12, // Magnesium
        n if n.starts_with("MN") => return 25, // Manganese
        _ => {}
    }

    // Fall back to first character
    let first = name.chars().next().unwrap();

    match first {
        'H' => 1,  // Hydrogen
        'C' => 6,  // Carbon
        'N' => 7,  // Nitrogen
        'O' => 8,  // Oxygen
        'S' => 16, // Sulfur
        'P' => 15, // Phosphorus
        'F' => 9,  // Fluorine
        'K' => 19, // Potassium
        'Z' => 30, // Zinc (default)
        _ => 0, // Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_gro_simple() {
        // Water box with 3 atoms
        let gro_data = "\
Water molecule
    3
    1WAT     OW    1   0.126   0.126   0.126
    1WAT    HW1    2   0.190   0.126   0.126
    1WAT    HW2    3   0.062   0.126   0.126
   1.0   1.0   1.0
";
        let cursor = Cursor::new(gro_data);
        let atoms = parse_gro_reader(BufReader::new(cursor)).unwrap();

        assert_eq!(atoms.len(), 3);

        // Coordinates should be converted from nm to Angstroms (×10)
        assert_eq!(atoms.position(0), Some([1.26, 1.26, 1.26]));
        assert_eq!(atoms.position(1), Some([1.90, 1.26, 1.26]));
        assert_eq!(atoms.position(2), Some([0.62, 1.26, 1.26]));

        // Elements
        assert_eq!(atoms.element(0), Some(8));  // O
        assert_eq!(atoms.element(1), Some(1));  // H
        assert_eq!(atoms.element(2), Some(1));  // H
    }

    #[test]
    fn test_parse_gro_protein() {
        // Protein residue
        let gro_data = "\
Protein fragment
    5
    1ALA      N    1   1.000   2.000   3.000
    1ALA     CA    2   1.100   2.100   3.100
    1ALA      C    3   1.200   2.200   3.200
    1ALA      O    4   1.300   2.300   3.300
    2GLY      N    5   1.400   2.400   3.400
   5.0   5.0   5.0
";
        let cursor = Cursor::new(gro_data);
        let atoms = parse_gro_reader(BufReader::new(cursor)).unwrap();

        assert_eq!(atoms.len(), 5);
        assert_eq!(atoms.position(0), Some([10.0, 20.0, 30.0])); // nm to Å
        assert_eq!(atoms.element(0), Some(7));  // N
        assert_eq!(atoms.element(1), Some(6));  // C (CA)
        assert_eq!(atoms.element(2), Some(6));  // C
        assert_eq!(atoms.element(3), Some(8));  // O
    }

    #[test]
    fn test_parse_gro_invalid_atom_count() {
        let gro_data = "\
Title
10
    1WAT     OW    1   0.126   0.126   0.126
   1.0   1.0   1.0
";
        let cursor = Cursor::new(gro_data);
        let result = parse_gro_reader(BufReader::new(cursor));

        assert!(result.is_err());
    }

    #[test]
    fn test_atom_name_to_element() {
        assert_eq!(atom_name_to_element("H"), 1);
        assert_eq!(atom_name_to_element("HW1"), 1);
        assert_eq!(atom_name_to_element("C"), 6);
        assert_eq!(atom_name_to_element("CA"), 6);  // Alpha carbon
        assert_eq!(atom_name_to_element("N"), 7);
        assert_eq!(atom_name_to_element("O"), 8);
        assert_eq!(atom_name_to_element("OW"), 8);
        assert_eq!(atom_name_to_element("S"), 16);
        assert_eq!(atom_name_to_element("MG"), 12);
        assert_eq!(atom_name_to_element("CL"), 17);
    }
}
