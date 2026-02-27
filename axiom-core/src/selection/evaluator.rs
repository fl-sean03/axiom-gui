// Selection evaluator - converts AST to atom indices

use crate::atoms::Atoms;
use crate::errors::{AxiomError, Result};
use crate::selection::parser::SelectionAST;
use std::collections::HashSet;

/// Evaluate a selection AST and return matching atom indices
pub fn evaluate_selection(atoms: &Atoms, ast: &SelectionAST) -> Result<Vec<usize>> {
    let indices = evaluate_ast(atoms, ast)?;
    let mut result: Vec<usize> = indices.into_iter().collect();
    result.sort_unstable();
    Ok(result)
}

fn evaluate_ast(atoms: &Atoms, ast: &SelectionAST) -> Result<HashSet<usize>> {
    match ast {
        SelectionAST::All => {
            Ok((0..atoms.len()).collect())
        }

        SelectionAST::Element(element_symbol) => {
            let element_num = symbol_to_atomic_number(element_symbol)?;
            Ok(atoms
                .elements
                .iter()
                .enumerate()
                .filter(|(_, &e)| e == element_num)
                .map(|(i, _)| i)
                .collect())
        }

        SelectionAST::Resname(resname) => {
            if let Some(residue_names) = &atoms.residue_names {
                Ok(residue_names
                    .iter()
                    .enumerate()
                    .filter(|(_, r)| r.eq_ignore_ascii_case(resname))
                    .map(|(i, _)| i)
                    .collect())
            } else {
                Err(AxiomError::SelectionError(
                    "No residue names available in structure".to_string(),
                ))
            }
        }

        SelectionAST::Chain(chain) => {
            if let Some(chain_ids) = &atoms.chain_ids {
                Ok(chain_ids
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.eq_ignore_ascii_case(chain))
                    .map(|(i, _)| i)
                    .collect())
            } else {
                Err(AxiomError::SelectionError(
                    "No chain IDs available in structure".to_string(),
                ))
            }
        }

        SelectionAST::Resid(resid) => {
            if let Some(residue_indices) = &atoms.residue_indices {
                Ok(residue_indices
                    .iter()
                    .enumerate()
                    .filter(|(_, &r)| r == *resid)
                    .map(|(i, _)| i)
                    .collect())
            } else {
                Err(AxiomError::SelectionError(
                    "No residue indices available in structure".to_string(),
                ))
            }
        }

        SelectionAST::ResidRange(start, end) => {
            if let Some(residue_indices) = &atoms.residue_indices {
                Ok(residue_indices
                    .iter()
                    .enumerate()
                    .filter(|(_, &r)| r >= *start && r <= *end)
                    .map(|(i, _)| i)
                    .collect())
            } else {
                Err(AxiomError::SelectionError(
                    "No residue indices available in structure".to_string(),
                ))
            }
        }

        SelectionAST::Protein => {
            // Standard protein residues
            let protein_residues = vec![
                "ALA", "ARG", "ASN", "ASP", "CYS", "GLN", "GLU", "GLY", "HIS", "ILE",
                "LEU", "LYS", "MET", "PHE", "PRO", "SER", "THR", "TRP", "TYR", "VAL",
                // Alternative names
                "HSD", "HSE", "HSP", // Histidine protonation states
                "CYX", // Cysteine in disulfide bridge
            ];

            if let Some(residue_names) = &atoms.residue_names {
                Ok(residue_names
                    .iter()
                    .enumerate()
                    .filter(|(_, r)| {
                        protein_residues.iter().any(|&p| r.eq_ignore_ascii_case(p))
                    })
                    .map(|(i, _)| i)
                    .collect())
            } else {
                Err(AxiomError::SelectionError(
                    "No residue names available in structure".to_string(),
                ))
            }
        }

        SelectionAST::Water => {
            let result: HashSet<usize> = evaluate_ast(atoms, &SelectionAST::Resname("WAT".to_string()))?
                .into_iter()
                .chain(evaluate_ast(atoms, &SelectionAST::Resname("HOH".to_string()))?)
                .chain(evaluate_ast(atoms, &SelectionAST::Resname("TIP".to_string()))?)
                .chain(evaluate_ast(atoms, &SelectionAST::Resname("TIP3".to_string()))?)
                .collect();
            Ok(result)
        }

        SelectionAST::Backbone => {
            // Backbone atoms: N, CA, C, O
            // This is a simplified version - ideally we'd check atom names
            // For now, just return an error indicating we need atom names
            Err(AxiomError::SelectionError(
                "Backbone selection requires atom names (not yet implemented)".to_string(),
            ))
        }

        SelectionAST::Sidechain => {
            // Sidechain = protein - backbone
            Err(AxiomError::SelectionError(
                "Sidechain selection requires atom names (not yet implemented)".to_string(),
            ))
        }

        SelectionAST::Within(dist_cutoff, selection) => {
            let reference_indices = evaluate_ast(atoms, selection)?;

            // Calculate distances from each atom to nearest reference atom
            let mut result = HashSet::new();

            for i in 0..atoms.len() {
                let pos_i = [atoms.x[i], atoms.y[i], atoms.z[i]];

                for &ref_idx in &reference_indices {
                    let pos_ref = [atoms.x[ref_idx], atoms.y[ref_idx], atoms.z[ref_idx]];
                    let dist = distance(&pos_i, &pos_ref);

                    if dist <= *dist_cutoff {
                        result.insert(i);
                        break;
                    }
                }
            }

            Ok(result)
        }

        SelectionAST::And(left, right) => {
            let left_indices = evaluate_ast(atoms, left)?;
            let right_indices = evaluate_ast(atoms, right)?;
            Ok(left_indices.intersection(&right_indices).copied().collect())
        }

        SelectionAST::Or(left, right) => {
            let left_indices = evaluate_ast(atoms, left)?;
            let right_indices = evaluate_ast(atoms, right)?;
            Ok(left_indices.union(&right_indices).copied().collect())
        }

        SelectionAST::Not(selection) => {
            let selected = evaluate_ast(atoms, selection)?;
            Ok((0..atoms.len())
                .filter(|i| !selected.contains(i))
                .collect())
        }
    }
}

/// Convert element symbol to atomic number
fn symbol_to_atomic_number(symbol: &str) -> Result<u8> {
    let num = match symbol.to_uppercase().as_str() {
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
        "GA" => 31,
        "GE" => 32,
        "AS" => 33,
        "SE" => 34,
        "BR" => 35,
        "KR" => 36,
        // Add more as needed...
        _ => {
            return Err(AxiomError::SelectionError(format!(
                "Unknown element symbol: {}",
                symbol
            )))
        }
    };
    Ok(num)
}

/// Calculate Euclidean distance between two 3D points
fn distance(p1: &[f32; 3], p2: &[f32; 3]) -> f32 {
    let dx = p1[0] - p2[0];
    let dy = p1[1] - p2[1];
    let dz = p1[2] - p2[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::Atoms;

    fn create_test_structure() -> Atoms {
        let mut atoms = Atoms::new();

        // Water molecule
        atoms.push(0.0, 0.0, 0.0, 8);  // O
        atoms.push(0.96, 0.0, 0.0, 1); // H
        atoms.push(-0.24, 0.93, 0.0, 1); // H

        // Another atom far away (carbon)
        atoms.push(10.0, 10.0, 10.0, 6); // C

        atoms.residue_names = Some(vec![
            "WAT".to_string(),
            "WAT".to_string(),
            "WAT".to_string(),
            "LIG".to_string(),
        ]);

        atoms.chain_ids = Some(vec![
            "A".to_string(),
            "A".to_string(),
            "A".to_string(),
            "A".to_string(),
        ]);

        atoms.residue_indices = Some(vec![1, 1, 1, 2]);

        atoms
    }

    #[test]
    fn test_evaluate_all() {
        let atoms = create_test_structure();
        let ast = SelectionAST::All;
        let indices = evaluate_selection(&atoms, &ast).unwrap();
        assert_eq!(indices, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_evaluate_element() {
        let atoms = create_test_structure();
        let ast = SelectionAST::Element("O".to_string());
        let indices = evaluate_selection(&atoms, &ast).unwrap();
        assert_eq!(indices, vec![0]);
    }

    #[test]
    fn test_evaluate_resname() {
        let atoms = create_test_structure();
        let ast = SelectionAST::Resname("WAT".to_string());
        let indices = evaluate_selection(&atoms, &ast).unwrap();
        assert_eq!(indices, vec![0, 1, 2]);
    }

    #[test]
    fn test_evaluate_within() {
        let atoms = create_test_structure();
        // Select atoms within 2Å of carbon at (10,10,10)
        let ast = SelectionAST::Within(
            2.0,
            Box::new(SelectionAST::Resname("LIG".to_string())),
        );
        let indices = evaluate_selection(&atoms, &ast).unwrap();
        // Only the carbon itself should be selected (within 0Å of itself)
        assert_eq!(indices, vec![3]);
    }

    #[test]
    fn test_evaluate_and() {
        let atoms = create_test_structure();
        let ast = SelectionAST::And(
            Box::new(SelectionAST::Element("O".to_string())),
            Box::new(SelectionAST::Resname("WAT".to_string())),
        );
        let indices = evaluate_selection(&atoms, &ast).unwrap();
        assert_eq!(indices, vec![0]);
    }

    #[test]
    fn test_evaluate_not() {
        let atoms = create_test_structure();
        let ast = SelectionAST::Not(Box::new(SelectionAST::Resname("WAT".to_string())));
        let indices = evaluate_selection(&atoms, &ast).unwrap();
        assert_eq!(indices, vec![3]); // Only the carbon (LIG)
    }
}
