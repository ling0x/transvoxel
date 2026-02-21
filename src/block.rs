//! Defines a [`Block`] — a cubic region of space to be triangulated.
//!
//! A block is characterised by:
//! - an **origin** (world-space minimum corner),
//! - a **size** (world-space edge length),
//! - a **subdivision count** (number of cells per axis).
//!
//! The voxel grid has `subdivisions + 1` sample points along each axis.

/// A cubic region of world space for mesh extraction.
#[derive(Debug, Clone)]
pub struct Block {
    /// World-space minimum corner.
    pub origin: [f32; 3],
    /// World-space edge length.
    pub size: f32,
    /// Number of cells along each axis (`n` → `n³` cells, `(n+1)³` voxels).
    pub subdivisions: usize,
}

impl Block {
    /// Create a new block.
    ///
    /// # Panics
    /// Panics if `subdivisions == 0`.
    pub fn new(origin: [f32; 3], size: f32, subdivisions: usize) -> Self {
        assert!(subdivisions >= 1, "subdivisions must be >= 1");
        Block { origin, size, subdivisions }
    }

    /// Number of voxel sample points per axis (`subdivisions + 1`).
    #[inline]
    pub fn voxels_per_axis(&self) -> usize {
        self.subdivisions + 1
    }

    /// World-space distance between adjacent voxel samples.
    #[inline]
    pub fn voxel_step(&self) -> f32 {
        self.size / self.subdivisions as f32
    }

    /// Convert a voxel grid index `(ix, iy, iz)` to a world-space position.
    #[inline]
    pub fn voxel_position(&self, ix: usize, iy: usize, iz: usize) -> [f32; 3] {
        let s = self.voxel_step();
        [
            self.origin[0] + ix as f32 * s,
            self.origin[1] + iy as f32 * s,
            self.origin[2] + iz as f32 * s,
        ]
    }
}
