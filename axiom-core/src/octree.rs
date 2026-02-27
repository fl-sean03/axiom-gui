// Octree spatial indexing for efficient large structure handling
// Enables fast queries for frustum culling, LOD selection, and neighbor searches

use crate::atoms::Atoms;

/// Axis-aligned bounding box
#[derive(Clone, Debug)]
pub struct AABB {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl AABB {
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    /// Check if point is inside AABB
    pub fn contains_point(&self, point: [f32; 3]) -> bool {
        point[0] >= self.min[0] && point[0] <= self.max[0]
            && point[1] >= self.min[1] && point[1] <= self.max[1]
            && point[2] >= self.min[2] && point[2] <= self.max[2]
    }

    /// Check if sphere (atom) intersects AABB
    pub fn intersects_sphere(&self, center: [f32; 3], radius: f32) -> bool {
        // Find closest point in AABB to sphere center
        let closest_x = center[0].clamp(self.min[0], self.max[0]);
        let closest_y = center[1].clamp(self.min[1], self.max[1]);
        let closest_z = center[2].clamp(self.min[2], self.max[2]);

        // Check if distance to closest point is less than radius
        let dx = center[0] - closest_x;
        let dy = center[1] - closest_y;
        let dz = center[2] - closest_z;
        let dist_sq = dx * dx + dy * dy + dz * dz;

        dist_sq <= radius * radius
    }

    /// Check if frustum (6 planes) intersects AABB
    pub fn intersects_frustum(&self, frustum_planes: &[[f32; 4]; 6]) -> bool {
        // Test AABB against each frustum plane
        // Plane equation: ax + by + cz + d = 0
        for plane in frustum_planes {
            let (a, b, c, d) = (plane[0], plane[1], plane[2], plane[3]);

            // Find positive/negative vertices of AABB relative to plane normal
            let px = if a >= 0.0 { self.max[0] } else { self.min[0] };
            let py = if b >= 0.0 { self.max[1] } else { self.min[1] };
            let pz = if c >= 0.0 { self.max[2] } else { self.min[2] };

            // If positive vertex is outside (negative side), AABB is completely outside
            if a * px + b * py + c * pz + d < 0.0 {
                return false;
            }
        }
        true
    }

    /// Get center of AABB
    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) / 2.0,
            (self.min[1] + self.max[1]) / 2.0,
            (self.min[2] + self.max[2]) / 2.0,
        ]
    }

    /// Get size of AABB (max dimension)
    pub fn max_extent(&self) -> f32 {
        let dx = self.max[0] - self.min[0];
        let dy = self.max[1] - self.min[1];
        let dz = self.max[2] - self.min[2];
        dx.max(dy).max(dz)
    }
}

/// Octree node for spatial partitioning
pub struct OctreeNode {
    pub bounds: AABB,
    pub atom_indices: Vec<usize>,  // Atoms in this node (leaf nodes only)
    pub children: Option<Box<[OctreeNode; 8]>>,  // 8 octants
    pub depth: u32,
}

impl OctreeNode {
    /// Create new leaf node
    fn new_leaf(bounds: AABB, atom_indices: Vec<usize>, depth: u32) -> Self {
        Self {
            bounds,
            atom_indices,
            children: None,
            depth,
        }
    }

    /// Check if this is a leaf node
    pub fn is_leaf(&self) -> bool {
        self.children.is_none()
    }

    /// Subdivide node into 8 children
    fn subdivide(&mut self, atoms: &Atoms, max_depth: u32, max_atoms_per_node: usize) {
        if self.depth >= max_depth || self.atom_indices.len() <= max_atoms_per_node {
            return;  // Don't subdivide further
        }

        let center = self.bounds.center();
        let mut child_indices: [Vec<usize>; 8] = Default::default();

        // Distribute atoms to children based on position relative to center
        for &atom_idx in &self.atom_indices {
            let x = atoms.x[atom_idx];
            let y = atoms.y[atom_idx];
            let z = atoms.z[atom_idx];

            let child_idx =
                (if x > center[0] { 4 } else { 0 })
                | (if y > center[1] { 2 } else { 0 })
                | (if z > center[2] { 1 } else { 0 });

            child_indices[child_idx].push(atom_idx);
        }

        // Create 8 child nodes
        let mut children = Vec::with_capacity(8);
        for i in 0..8 {
            let min = [
                if i & 4 != 0 { center[0] } else { self.bounds.min[0] },
                if i & 2 != 0 { center[1] } else { self.bounds.min[1] },
                if i & 1 != 0 { center[2] } else { self.bounds.min[2] },
            ];
            let max = [
                if i & 4 != 0 { self.bounds.max[0] } else { center[0] },
                if i & 2 != 0 { self.bounds.max[1] } else { center[1] },
                if i & 1 != 0 { self.bounds.max[2] } else { center[2] },
            ];

            let mut child = OctreeNode::new_leaf(
                AABB::new(min, max),
                child_indices[i].clone(),
                self.depth + 1,
            );

            // Recursively subdivide children if needed
            child.subdivide(atoms, max_depth, max_atoms_per_node);
            children.push(child);
        }

        // Convert Vec to fixed-size array
        self.children = Some(Box::new([
            children.remove(0), children.remove(0), children.remove(0), children.remove(0),
            children.remove(0), children.remove(0), children.remove(0), children.remove(0),
        ]));

        // Clear atom indices from internal node (only leaves store atoms)
        self.atom_indices.clear();
    }

    /// Query atoms within frustum (recursive)
    pub fn query_frustum(&self, frustum_planes: &[[f32; 4]; 6], result: &mut Vec<usize>) {
        // Check if node intersects frustum
        if !self.bounds.intersects_frustum(frustum_planes) {
            return;  // Early exit: entire subtree outside frustum
        }

        if self.is_leaf() {
            // Leaf node: add all atoms (further culling done per-atom)
            result.extend_from_slice(&self.atom_indices);
        } else if let Some(ref children) = self.children {
            // Internal node: recursively query children
            for child in children.iter() {
                child.query_frustum(frustum_planes, result);
            }
        }
    }

    /// Query atoms within sphere (for LOD selection)
    pub fn query_sphere(&self, center: [f32; 3], radius: f32, result: &mut Vec<usize>) {
        // Check if node intersects sphere
        if !self.bounds.intersects_sphere(center, radius) {
            return;
        }

        if self.is_leaf() {
            result.extend_from_slice(&self.atom_indices);
        } else if let Some(ref children) = self.children {
            for child in children.iter() {
                child.query_sphere(center, radius, result);
            }
        }
    }

    /// Get node count (for debugging/stats)
    pub fn count_nodes(&self) -> usize {
        if self.is_leaf() {
            1
        } else if let Some(ref children) = self.children {
            1 + children.iter().map(|c| c.count_nodes()).sum::<usize>()
        } else {
            1
        }
    }
}

/// Octree spatial index for efficient large structure queries
pub struct Octree {
    root: OctreeNode,
    pub atom_count: usize,
}

impl Octree {
    /// Build octree from atoms
    pub fn build(atoms: &Atoms, max_depth: u32, max_atoms_per_node: usize) -> Self {
        // Calculate bounding box with small margin
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];

        for i in 0..atoms.len() {
            min[0] = min[0].min(atoms.x[i]);
            min[1] = min[1].min(atoms.y[i]);
            min[2] = min[2].min(atoms.z[i]);
            max[0] = max[0].max(atoms.x[i]);
            max[1] = max[1].max(atoms.y[i]);
            max[2] = max[2].max(atoms.z[i]);
        }

        // Add small margin to avoid edge cases
        let margin = 0.1;
        min[0] -= margin;
        min[1] -= margin;
        min[2] -= margin;
        max[0] += margin;
        max[1] += margin;
        max[2] += margin;

        let bounds = AABB::new(min, max);
        let atom_indices: Vec<usize> = (0..atoms.len()).collect();

        let mut root = OctreeNode::new_leaf(bounds, atom_indices, 0);
        root.subdivide(atoms, max_depth, max_atoms_per_node);

        Self {
            root,
            atom_count: atoms.len(),
        }
    }

    /// Query visible atoms using frustum culling
    pub fn query_visible(&self, frustum_planes: &[[f32; 4]; 6]) -> Vec<usize> {
        let mut result = Vec::new();
        self.root.query_frustum(frustum_planes, &mut result);
        result
    }

    /// Query atoms near camera (for LOD selection)
    pub fn query_near_camera(&self, camera_pos: [f32; 3], radius: f32) -> Vec<usize> {
        let mut result = Vec::new();
        self.root.query_sphere(camera_pos, radius, &mut result);
        result
    }

    /// Get statistics (for debugging)
    pub fn stats(&self) -> OctreeStats {
        OctreeStats {
            total_nodes: self.root.count_nodes(),
            total_atoms: self.atom_count,
            max_depth: self.get_max_depth(&self.root),
        }
    }

    fn get_max_depth(&self, node: &OctreeNode) -> u32 {
        if node.is_leaf() {
            node.depth
        } else if let Some(ref children) = node.children {
            children.iter().map(|c| self.get_max_depth(c)).max().unwrap_or(node.depth)
        } else {
            node.depth
        }
    }
}

/// Octree statistics
#[derive(Debug)]
pub struct OctreeStats {
    pub total_nodes: usize,
    pub total_atoms: usize,
    pub max_depth: u32,
}
