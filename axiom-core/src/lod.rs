// Level of Detail (LOD) system for rendering performance optimization
// Reduces detail for distant atoms to maintain interactive framerates

/// LOD level for rendering
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LODLevel {
    /// Full detail: sphere rendering with lighting
    High,
    /// Medium detail: simplified sphere with reduced tessellation
    Medium,
    /// Low detail: point sprite or very simple geometry
    Low,
    /// Minimal detail: single pixel point
    Minimal,
}

impl LODLevel {
    /// Get quality factor (0.0-1.0) for this LOD level
    pub fn quality_factor(&self) -> f32 {
        match self {
            LODLevel::High => 1.0,
            LODLevel::Medium => 0.6,
            LODLevel::Low => 0.3,
            LODLevel::Minimal => 0.1,
        }
    }

    /// Get radius multiplier for this LOD level
    pub fn radius_multiplier(&self) -> f32 {
        match self {
            LODLevel::High => 1.0,
            LODLevel::Medium => 0.85,
            LODLevel::Low => 0.6,
            LODLevel::Minimal => 0.3,
        }
    }
}

/// LOD configuration
#[derive(Clone, Debug)]
pub struct LODConfig {
    /// Enable LOD system
    pub enabled: bool,
    /// Distance thresholds for LOD levels (in world space units)
    pub high_threshold: f32,    // < this = High
    pub medium_threshold: f32,  // < this = Medium
    pub low_threshold: f32,     // < this = Low
    // >= low_threshold = Minimal
}

impl Default for LODConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            high_threshold: 30.0,    // High detail within 30 units
            medium_threshold: 60.0,   // Medium detail 30-60 units
            low_threshold: 100.0,     // Low detail 60-100 units
            // Minimal beyond 100 units
        }
    }
}

impl LODConfig {
    /// Determine LOD level based on distance from camera
    pub fn get_lod_level(&self, distance: f32) -> LODLevel {
        if !self.enabled {
            return LODLevel::High;
        }

        if distance < self.high_threshold {
            LODLevel::High
        } else if distance < self.medium_threshold {
            LODLevel::Medium
        } else if distance < self.low_threshold {
            LODLevel::Low
        } else {
            LODLevel::Minimal
        }
    }

    /// Calculate distance from camera to atom
    pub fn calculate_distance(camera_pos: [f32; 3], atom_pos: [f32; 3]) -> f32 {
        let dx = atom_pos[0] - camera_pos[0];
        let dy = atom_pos[1] - camera_pos[1];
        let dz = atom_pos[2] - camera_pos[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// LOD statistics for performance monitoring
#[derive(Default, Debug, Clone)]
pub struct LODStats {
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub minimal_count: usize,
    pub culled_count: usize,  // Atoms culled by frustum
}

impl LODStats {
    pub fn total_rendered(&self) -> usize {
        self.high_count + self.medium_count + self.low_count + self.minimal_count
    }

    pub fn total_atoms(&self) -> usize {
        self.total_rendered() + self.culled_count
    }

    pub fn record_lod(&mut self, level: LODLevel) {
        match level {
            LODLevel::High => self.high_count += 1,
            LODLevel::Medium => self.medium_count += 1,
            LODLevel::Low => self.low_count += 1,
            LODLevel::Minimal => self.minimal_count += 1,
        }
    }

    pub fn reset(&mut self) {
        self.high_count = 0;
        self.medium_count = 0;
        self.low_count = 0;
        self.minimal_count = 0;
        self.culled_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_selection() {
        let config = LODConfig::default();

        assert_eq!(config.get_lod_level(10.0), LODLevel::High);
        assert_eq!(config.get_lod_level(45.0), LODLevel::Medium);
        assert_eq!(config.get_lod_level(80.0), LODLevel::Low);
        assert_eq!(config.get_lod_level(150.0), LODLevel::Minimal);
    }

    #[test]
    fn test_distance_calculation() {
        let distance = LODConfig::calculate_distance(
            [0.0, 0.0, 0.0],
            [3.0, 4.0, 0.0],
        );
        assert_eq!(distance, 5.0);
    }

    #[test]
    fn test_lod_disabled() {
        let config = LODConfig {
            enabled: false,
            ..Default::default()
        };

        assert_eq!(config.get_lod_level(10.0), LODLevel::High);
        assert_eq!(config.get_lod_level(1000.0), LODLevel::High);
    }
}
