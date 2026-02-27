// Integration tests for Axiom core

use axiom_core::parsers::parse_xyz;

#[test]
fn test_parse_water_xyz() {
    let atoms = parse_xyz("tests/data/water.xyz").unwrap();

    assert_eq!(atoms.len(), 3);
    assert_eq!(atoms.element(0), Some(8)); // O
    assert_eq!(atoms.element(1), Some(1)); // H
    assert_eq!(atoms.element(2), Some(1)); // H

    // Check positions
    let pos0 = atoms.position(0).unwrap();
    assert!((pos0[0] - 0.0).abs() < 1e-6);
    assert!((pos0[1] - 0.0).abs() < 1e-6);
    assert!((pos0[2] - 0.0).abs() < 1e-6);
}

#[test]
fn test_parse_methane_xyz() {
    let atoms = parse_xyz("tests/data/methane.xyz").unwrap();

    assert_eq!(atoms.len(), 5);
    assert_eq!(atoms.element(0), Some(6)); // C
    assert_eq!(atoms.element(1), Some(1)); // H
    assert_eq!(atoms.element(2), Some(1)); // H
    assert_eq!(atoms.element(3), Some(1)); // H
    assert_eq!(atoms.element(4), Some(1)); // H
}
