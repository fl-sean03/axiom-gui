// XYZ file parser
//
// XYZ format:
// Line 1: Number of atoms
// Line 2: Comment line
// Line 3+: Element X Y Z

use crate::atoms::Atoms;
use crate::errors::{AxiomError, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Parse XYZ file
pub fn parse_xyz<P: AsRef<Path>>(path: P) -> Result<Atoms> {
    let file = File::open(path.as_ref())
        .map_err(|_| AxiomError::FileNotFound(path.as_ref().display().to_string()))?;
    let reader = BufReader::new(file);
    parse_xyz_reader(reader)
}

/// Parse XYZ from a buffered reader
pub fn parse_xyz_reader<R: BufRead>(reader: R) -> Result<Atoms> {
    let mut lines = reader.lines();

    // Line 1: Number of atoms
    let num_atoms: usize = lines
        .next()
        .ok_or_else(|| AxiomError::ParseError("Empty file".to_string()))??
        .trim()
        .parse()
        .map_err(|_| AxiomError::ParseError("Invalid atom count".to_string()))?;

    // Line 2: Comment (skip)
    lines.next();

    // Allocate capacity
    let mut atoms = Atoms::with_capacity(num_atoms);

    // Parse atoms
    for (line_num, line_result) in lines.enumerate() {
        let line = line_result?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 4 {
            return Err(AxiomError::ParseError(format!(
                "Invalid line {}: expected 'Element X Y Z'",
                line_num + 3
            )));
        }

        let element_symbol = parts[0];
        let x: f32 = parts[1]
            .parse()
            .map_err(|_| AxiomError::ParseError(format!("Invalid X coordinate on line {}", line_num + 3)))?;
        let y: f32 = parts[2]
            .parse()
            .map_err(|_| AxiomError::ParseError(format!("Invalid Y coordinate on line {}", line_num + 3)))?;
        let z: f32 = parts[3]
            .parse()
            .map_err(|_| AxiomError::ParseError(format!("Invalid Z coordinate on line {}", line_num + 3)))?;

        let element = symbol_to_atomic_number(element_symbol);
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

/// Convert element symbol to atomic number
fn symbol_to_atomic_number(symbol: &str) -> u8 {
    match symbol {
        "H" => 1,
        "He" => 2,
        "Li" => 3,
        "Be" => 4,
        "B" => 5,
        "C" => 6,
        "N" => 7,
        "O" => 8,
        "F" => 9,
        "Ne" => 10,
        "Na" => 11,
        "Mg" => 12,
        "Al" => 13,
        "Si" => 14,
        "P" => 15,
        "S" => 16,
        "Cl" => 17,
        "Ar" => 18,
        "K" => 19,
        "Ca" => 20,
        "Fe" => 26,
        "Cu" => 29,
        "Zn" => 30,
        "Br" => 35,
        "Ag" => 47,
        "I" => 53,
        "Au" => 79,
        // Add more as needed
        _ => 0, // Unknown element
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_xyz_simple() {
        let xyz_data = "3\nWater molecule\nO 0.0 0.0 0.0\nH 0.757 0.586 0.0\nH -0.757 0.586 0.0\n";
        let cursor = Cursor::new(xyz_data);
        let atoms = parse_xyz_reader(BufReader::new(cursor)).unwrap();

        assert_eq!(atoms.len(), 3);
        assert_eq!(atoms.element(0), Some(8)); // O
        assert_eq!(atoms.element(1), Some(1)); // H
        assert_eq!(atoms.element(2), Some(1)); // H
        assert_eq!(atoms.position(0), Some([0.0, 0.0, 0.0]));
    }

    #[test]
    fn test_parse_xyz_invalid_atom_count() {
        let xyz_data = "5\nComment\nO 0.0 0.0 0.0\nH 0.757 0.586 0.0\n";
        let cursor = Cursor::new(xyz_data);
        let result = parse_xyz_reader(BufReader::new(cursor));

        assert!(result.is_err());
    }

    #[test]
    fn test_symbol_to_atomic_number() {
        assert_eq!(symbol_to_atomic_number("H"), 1);
        assert_eq!(symbol_to_atomic_number("C"), 6);
        assert_eq!(symbol_to_atomic_number("O"), 8);
        assert_eq!(symbol_to_atomic_number("Au"), 79);
        assert_eq!(symbol_to_atomic_number("Unknown"), 0);
    }
}
