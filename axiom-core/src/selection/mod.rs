// Semantic selection parser for Axiom
// Supports queries like "element O", "resname WAT", "within 5 of resname LIG"

pub mod parser;
pub mod evaluator;

pub use parser::{parse_selection, SelectionAST, SelectionToken};
pub use evaluator::evaluate_selection;

use crate::atoms::Atoms;
use crate::errors::Result;

/// Main entry point for selection queries
///
/// # Examples
/// ```
/// use axiom_core::selection::select;
/// use axiom_core::atoms::Atoms;
///
/// let mut atoms = Atoms::new();
/// atoms.push(0.0, 0.0, 0.0, 8);  // oxygen
/// atoms.residue_names = Some(vec!["WAT".to_string()]);
///
/// let indices = select(&atoms, "element O").unwrap();
/// assert_eq!(indices, vec![0]);
/// ```
pub fn select(atoms: &Atoms, query: &str) -> Result<Vec<usize>> {
    let ast = parse_selection(query)?;
    evaluate_selection(atoms, &ast)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::Atoms;

    fn create_test_atoms() -> Atoms {
        let mut atoms = Atoms::new();

        // Add water molecule (O-H-H)
        atoms.push(0.0, 0.0, 0.0, 8);  // O
        atoms.push(0.96, 0.0, 0.0, 1); // H
        atoms.push(-0.24, 0.93, 0.0, 1); // H

        // Add metadata
        atoms.residue_names = Some(vec!["WAT".to_string(), "WAT".to_string(), "WAT".to_string()]);
        atoms.chain_ids = Some(vec!["A".to_string(), "A".to_string(), "A".to_string()]);
        atoms.residue_indices = Some(vec![1, 1, 1]);

        atoms
    }

    #[test]
    fn test_select_all() {
        let atoms = create_test_atoms();
        let indices = select(&atoms, "all").unwrap();
        assert_eq!(indices, vec![0, 1, 2]);
    }

    #[test]
    fn test_select_element() {
        let atoms = create_test_atoms();
        let indices = select(&atoms, "element O").unwrap();
        assert_eq!(indices, vec![0]);

        let indices = select(&atoms, "element H").unwrap();
        assert_eq!(indices, vec![1, 2]);
    }

    #[test]
    fn test_select_resname() {
        let atoms = create_test_atoms();
        let indices = select(&atoms, "resname WAT").unwrap();
        assert_eq!(indices, vec![0, 1, 2]);
    }
}
