// CPK (Corey-Pauling-Koltun) element coloring
// Standard atomic colors for molecular visualization

/// Get RGB color for an atomic number (CPK scheme)
pub fn element_to_cpk_color(atomic_number: u8) -> [f32; 3] {
    match atomic_number {
        1 => [1.0, 1.0, 1.0],      // H - white
        6 => [0.5, 0.5, 0.5],      // C - gray
        7 => [0.2, 0.2, 1.0],      // N - blue
        8 => [1.0, 0.0, 0.0],      // O - red
        9 => [0.7, 1.0, 1.0],      // F - light cyan
        15 => [1.0, 0.5, 0.0],     // P - orange
        16 => [1.0, 1.0, 0.0],     // S - yellow
        17 => [0.0, 1.0, 0.0],     // Cl - green
        35 => [0.6, 0.1, 0.1],     // Br - dark red
        53 => [0.5, 0.0, 0.5],     // I - purple

        // Metals
        11 => [0.0, 0.0, 1.0],     // Na - blue
        12 => [0.0, 0.5, 0.0],     // Mg - dark green
        19 => [0.5, 0.0, 0.5],     // K - purple
        20 => [0.5, 0.5, 0.5],     // Ca - gray
        26 => [0.9, 0.4, 0.0],     // Fe - orange
        29 => [0.8, 0.5, 0.2],     // Cu - copper
        30 => [0.5, 0.5, 0.7],     // Zn - light gray-blue

        // Default: light pink for unknown elements
        _ => [1.0, 0.7, 0.8],
    }
}

/// Get element radius for rendering (in Angstroms)
/// Uses van der Waals radii
pub fn element_to_vdw_radius(atomic_number: u8) -> f32 {
    match atomic_number {
        1 => 1.20,   // H
        6 => 1.70,   // C
        7 => 1.55,   // N
        8 => 1.52,   // O
        9 => 1.47,   // F
        15 => 1.80,  // P
        16 => 1.80,  // S
        17 => 1.75,  // Cl
        35 => 1.85,  // Br
        53 => 1.98,  // I

        11 => 2.27,  // Na
        12 => 1.73,  // Mg
        19 => 2.75,  // K
        20 => 2.31,  // Ca
        26 => 2.04,  // Fe
        29 => 1.40,  // Cu
        30 => 1.39,  // Zn

        // Default: 1.5 Å
        _ => 1.50,
    }
}

/// Get element radius scaled for ball-and-stick representation
pub fn element_to_ball_stick_radius(atomic_number: u8) -> f32 {
    element_to_vdw_radius(atomic_number) * 0.3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpk_colors() {
        // Hydrogen is white
        assert_eq!(element_to_cpk_color(1), [1.0, 1.0, 1.0]);

        // Carbon is gray
        assert_eq!(element_to_cpk_color(6), [0.5, 0.5, 0.5]);

        // Oxygen is red
        assert_eq!(element_to_cpk_color(8), [1.0, 0.0, 0.0]);

        // Unknown element gets default color
        let color = element_to_cpk_color(255);
        assert_eq!(color[0], 1.0); // Pink-ish
    }

    #[test]
    fn test_vdw_radii() {
        // Hydrogen
        assert_eq!(element_to_vdw_radius(1), 1.20);

        // Carbon
        assert_eq!(element_to_vdw_radius(6), 1.70);

        // Unknown element gets default
        assert_eq!(element_to_vdw_radius(255), 1.50);
    }

    #[test]
    fn test_ball_stick_radius() {
        // Should be 30% of vdW radius
        let vdw = element_to_vdw_radius(6);
        let ball_stick = element_to_ball_stick_radius(6);
        assert!((ball_stick - vdw * 0.3).abs() < 0.001);
    }
}
