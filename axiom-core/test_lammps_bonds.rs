// Quick test for LAMMPS bond parsing
use std::path::Path;

fn main() {
    // Add axiom-core to path
    let data_file = "/home/agent/messages/downloads/1771984928_438159_equil_nvt_dry.data";

    println!("Parsing {}...", data_file);

    match axiom_core::parsers::parse_lammps_data_with_bonds(Path::new(data_file)) {
        Ok((atoms, bonds)) => {
            println!("\n✓ Atoms: {} atoms", atoms.len());
            println!("  - X range: [{:.2}, {:.2}]",
                atoms.x.iter().fold(f32::INFINITY, |a, &b| a.min(b)),
                atoms.x.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b)));
            println!("  - Y range: [{:.2}, {:.2}]",
                atoms.y.iter().fold(f32::INFINITY, |a, &b| a.min(b)),
                atoms.y.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b)));
            println!("  - Z range: [{:.2}, {:.2}]",
                atoms.z.iter().fold(f32::INFINITY, |a, &b| a.min(b)),
                atoms.z.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b)));

            if let Some(ref charges) = atoms.charges {
                println!("  - Charges: [{:.3}, {:.3}]",
                    charges.iter().fold(f32::INFINITY, |a, &b| a.min(b)),
                    charges.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b)));
            }

            if let Some(ref types) = atoms.atom_types {
                let unique_types: std::collections::HashSet<_> = types.iter().collect();
                println!("  - Atom types: {} unique types", unique_types.len());
            }

            println!("\n✓ Bonds: {} bonds", bonds.len());
            if bonds.len() > 0 {
                println!("  - First 10 bonds:");
                for i in 0..std::cmp::min(10, bonds.len()) {
                    if let Some((a1, a2, order)) = bonds.get(i) {
                        println!("    {}: {} - {} (order {})", i, a1, a2, order);
                    }
                }
            } else {
                println!("  - WARNING: No bonds found!");
            }

            println!("\n✓ Test complete!");
        }
        Err(e) => {
            eprintln!("ERROR: {:?}", e);
            std::process::exit(1);
        }
    }
}
