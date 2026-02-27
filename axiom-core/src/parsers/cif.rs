// CIF (Crystallographic Information File) parser
//
// CIF format is a loop-based data structure used for crystallographic data.
// It contains:
// - Unit cell parameters (a, b, c, alpha, beta, gamma)
// - Atom sites with fractional coordinates
// - Optional: bond information via _geom_bond loops
// - Symmetry operations

use crate::atoms::{Atoms, Bonds};
use crate::errors::{AxiomError, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Parse CIF file
pub fn parse_cif<P: AsRef<Path>>(path: P) -> Result<Atoms> {
    let file = File::open(path.as_ref())
        .map_err(|_| AxiomError::FileNotFound(path.as_ref().display().to_string()))?;
    let reader = BufReader::new(file);
    let (atoms, _bonds) = parse_cif_with_bonds_reader(reader)?;
    Ok(atoms)
}

/// Parse CIF file with bonds
pub fn parse_cif_with_bonds<P: AsRef<Path>>(path: P) -> Result<(Atoms, Bonds)> {
    let file = File::open(path.as_ref())
        .map_err(|_| AxiomError::FileNotFound(path.as_ref().display().to_string()))?;
    let reader = BufReader::new(file);
    parse_cif_with_bonds_reader(reader)
}

/// Parse CIF from a buffered reader
fn parse_cif_with_bonds_reader<R: BufRead>(reader: R) -> Result<(Atoms, Bonds)> {
    let mut atoms = Atoms::new();
    let mut bonds = Bonds::new();

    // Unit cell parameters
    let mut cell_a = 1.0_f32;
    let mut cell_b = 1.0_f32;
    let mut cell_c = 1.0_f32;
    let mut cell_alpha = 90.0_f32;
    let mut cell_beta = 90.0_f32;
    let mut cell_gamma = 90.0_f32;

    // Atom data (fractional coordinates)
    let mut atom_labels: Vec<String> = Vec::new();
    let mut atom_symbols: Vec<String> = Vec::new();
    let mut fract_x: Vec<f32> = Vec::new();
    let mut fract_y: Vec<f32> = Vec::new();
    let mut fract_z: Vec<f32> = Vec::new();

    // Bond data (labels, not indices)
    let mut bond_labels: Vec<(String, String)> = Vec::new();

    // Parse state
    let mut in_atom_loop = false;
    let mut in_bond_loop = false;
    let mut atom_loop_cols: HashMap<String, usize> = HashMap::new();
    let mut bond_loop_cols: HashMap<String, usize> = HashMap::new();

    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parse unit cell parameters
        if trimmed.starts_with("_cell_length_a") {
            cell_a = parse_cell_param(&trimmed)?;
        } else if trimmed.starts_with("_cell_length_b") {
            cell_b = parse_cell_param(&trimmed)?;
        } else if trimmed.starts_with("_cell_length_c") {
            cell_c = parse_cell_param(&trimmed)?;
        } else if trimmed.starts_with("_cell_angle_alpha") {
            cell_alpha = parse_cell_param(&trimmed)?;
        } else if trimmed.starts_with("_cell_angle_beta") {
            cell_beta = parse_cell_param(&trimmed)?;
        } else if trimmed.starts_with("_cell_angle_gamma") {
            cell_gamma = parse_cell_param(&trimmed)?;
        }
        // Atom site loop
        else if trimmed.starts_with("loop_") {
            in_atom_loop = false;
            in_bond_loop = false;
            atom_loop_cols.clear();
            bond_loop_cols.clear();
        } else if trimmed.starts_with("_atom_site_") {
            in_atom_loop = true;
            let col_name = trimmed.to_string();
            atom_loop_cols.insert(col_name, atom_loop_cols.len());
        } else if trimmed.starts_with("_geom_bond_") {
            in_bond_loop = true;
            let col_name = trimmed.to_string();
            bond_loop_cols.insert(col_name, bond_loop_cols.len());
        }
        // Data lines (not starting with _ or loop_)
        else if !trimmed.starts_with('_') && !trimmed.starts_with("loop_") && !trimmed.starts_with("data_") {
            if in_atom_loop && !atom_loop_cols.is_empty() {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();

                if parts.len() < atom_loop_cols.len() {
                    continue; // Skip malformed lines
                }

                // Extract data based on column positions
                let label_col = atom_loop_cols.get("_atom_site_label");
                let symbol_col = atom_loop_cols.get("_atom_site_type_symbol");
                let x_col = atom_loop_cols.get("_atom_site_fract_x");
                let y_col = atom_loop_cols.get("_atom_site_fract_y");
                let z_col = atom_loop_cols.get("_atom_site_fract_z");

                if let (Some(&label_idx), Some(&x_idx), Some(&y_idx), Some(&z_idx)) = (label_col, x_col, y_col, z_col) {
                    if parts.len() > label_idx && parts.len() > x_idx && parts.len() > y_idx && parts.len() > z_idx {
                        atom_labels.push(parts[label_idx].to_string());

                        // Symbol might be in type_symbol column, or extract from label
                        let symbol = if let Some(&sym_idx) = symbol_col {
                            if parts.len() > sym_idx {
                                parts[sym_idx].to_string()
                            } else {
                                extract_symbol_from_label(parts[label_idx])
                            }
                        } else {
                            extract_symbol_from_label(parts[label_idx])
                        };
                        atom_symbols.push(symbol);

                        // Parse fractional coordinates (remove uncertainty if present, e.g. "0.5(2)")
                        fract_x.push(parse_coord_with_uncertainty(parts[x_idx])?);
                        fract_y.push(parse_coord_with_uncertainty(parts[y_idx])?);
                        fract_z.push(parse_coord_with_uncertainty(parts[z_idx])?);
                    }
                }
            } else if in_bond_loop && !bond_loop_cols.is_empty() {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();

                if parts.len() < bond_loop_cols.len() {
                    continue;
                }

                let atom1_col = bond_loop_cols.get("_geom_bond_atom_site_label_1");
                let atom2_col = bond_loop_cols.get("_geom_bond_atom_site_label_2");

                if let (Some(&a1_idx), Some(&a2_idx)) = (atom1_col, atom2_col) {
                    if parts.len() > a1_idx && parts.len() > a2_idx {
                        bond_labels.push((parts[a1_idx].to_string(), parts[a2_idx].to_string()));
                    }
                }
            }
        }
    }

    if atom_labels.is_empty() {
        return Err(AxiomError::ParseError("No atoms found in CIF file".to_string()));
    }

    // Convert fractional to Cartesian coordinates
    let (cart_x, cart_y, cart_z) = fractional_to_cartesian(
        &fract_x, &fract_y, &fract_z,
        cell_a, cell_b, cell_c,
        cell_alpha, cell_beta, cell_gamma
    );

    // Build atom structure
    for i in 0..atom_labels.len() {
        let element = symbol_to_atomic_number(&atom_symbols[i]);
        atoms.push(cart_x[i], cart_y[i], cart_z[i], element);
    }

    // Build bonds if present
    if !bond_labels.is_empty() {
        // Create label -> index map
        let mut label_to_index: HashMap<String, u32> = HashMap::new();
        for (i, label) in atom_labels.iter().enumerate() {
            label_to_index.insert(label.clone(), i as u32);
        }

        // Convert bond labels to indices
        for (label1, label2) in bond_labels {
            if let (Some(&idx1), Some(&idx2)) = (label_to_index.get(&label1), label_to_index.get(&label2)) {
                if idx1 < idx2 {
                    bonds.push(idx1, idx2, 1); // Default to single bond
                }
            }
        }
    }

    Ok((atoms, bonds))
}

/// Parse cell parameter (extract number from line like "_cell_length_a 10.0")
fn parse_cell_param(line: &str) -> Result<f32> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(AxiomError::ParseError(format!("Invalid cell parameter: {}", line)));
    }

    // Remove uncertainty if present (e.g. "10.0(5)" -> "10.0")
    parse_coord_with_uncertainty(parts[1])
}

/// Parse coordinate that may have uncertainty in parentheses
fn parse_coord_with_uncertainty(s: &str) -> Result<f32> {
    // Remove uncertainty: "0.5(2)" -> "0.5"
    let cleaned = if let Some(paren_pos) = s.find('(') {
        &s[..paren_pos]
    } else {
        s
    };

    cleaned.parse::<f32>()
        .map_err(|_| AxiomError::ParseError(format!("Invalid number: {}", s)))
}

/// Extract element symbol from atom label (e.g. "C1" -> "C", "Fe2" -> "Fe")
fn extract_symbol_from_label(label: &str) -> String {
    let mut symbol = String::new();
    for c in label.chars() {
        if c.is_alphabetic() {
            symbol.push(c);
        } else {
            break;
        }
    }
    symbol
}

/// Convert fractional coordinates to Cartesian
fn fractional_to_cartesian(
    fract_x: &[f32], fract_y: &[f32], fract_z: &[f32],
    a: f32, b: f32, c: f32,
    alpha: f32, beta: f32, gamma: f32
) -> (Vec<f32>, Vec<f32>, Vec<f32>) {

    // Convert angles to radians
    let alpha_rad = alpha.to_radians();
    let beta_rad = beta.to_radians();
    let gamma_rad = gamma.to_radians();

    // Build transformation matrix
    // This is the standard crystallographic transformation
    let cos_alpha = alpha_rad.cos();
    let cos_beta = beta_rad.cos();
    let cos_gamma = gamma_rad.cos();
    let sin_gamma = gamma_rad.sin();

    let volume = a * b * c * (1.0 - cos_alpha.powi(2) - cos_beta.powi(2) - cos_gamma.powi(2)
                             + 2.0 * cos_alpha * cos_beta * cos_gamma).sqrt();

    // Orthogonalization matrix (converts fractional to Cartesian)
    let m11 = a;
    let m12 = b * cos_gamma;
    let m13 = c * cos_beta;
    let m21 = 0.0;
    let m22 = b * sin_gamma;
    let m23 = c * (cos_alpha - cos_beta * cos_gamma) / sin_gamma;
    let m31 = 0.0;
    let m32 = 0.0;
    let m33 = volume / (a * b * sin_gamma);

    let mut cart_x = Vec::with_capacity(fract_x.len());
    let mut cart_y = Vec::with_capacity(fract_y.len());
    let mut cart_z = Vec::with_capacity(fract_z.len());

    for i in 0..fract_x.len() {
        let fx = fract_x[i];
        let fy = fract_y[i];
        let fz = fract_z[i];

        cart_x.push(m11 * fx + m12 * fy + m13 * fz);
        cart_y.push(m21 * fx + m22 * fy + m23 * fz);
        cart_z.push(m31 * fx + m32 * fy + m33 * fz);
    }

    (cart_x, cart_y, cart_z)
}

/// Convert element symbol to atomic number
fn symbol_to_atomic_number(symbol: &str) -> u8 {
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
        "TI" => 22,
        "FE" => 26,
        "CU" => 29,
        "ZN" => 30,
        _ => 0, // Unknown element
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_coord_with_uncertainty() {
        assert_eq!(parse_coord_with_uncertainty("0.5").unwrap(), 0.5);
        assert_eq!(parse_coord_with_uncertainty("0.5(2)").unwrap(), 0.5);
        assert_eq!(parse_coord_with_uncertainty("10.234(15)").unwrap(), 10.234);
    }

    #[test]
    fn test_extract_symbol_from_label() {
        assert_eq!(extract_symbol_from_label("C1"), "C");
        assert_eq!(extract_symbol_from_label("Fe2"), "Fe");
        assert_eq!(extract_symbol_from_label("O"), "O");
        assert_eq!(extract_symbol_from_label("H3A"), "H");
    }

    #[test]
    fn test_fractional_to_cartesian_cubic() {
        // Test with cubic cell (a=b=c=10, alpha=beta=gamma=90)
        let fract_x = vec![0.0, 0.5, 1.0];
        let fract_y = vec![0.0, 0.5, 1.0];
        let fract_z = vec![0.0, 0.5, 1.0];

        let (cart_x, cart_y, cart_z) = fractional_to_cartesian(
            &fract_x, &fract_y, &fract_z,
            10.0, 10.0, 10.0,
            90.0, 90.0, 90.0
        );

        assert!((cart_x[0] - 0.0).abs() < 0.001);
        assert!((cart_x[1] - 5.0).abs() < 0.001);
        assert!((cart_x[2] - 10.0).abs() < 0.001);
    }
}
