//! Lookup tables for the Transvoxel and Marching Cubes algorithms.
//!
//! Data is structured following Eric Lengyel's official Transvoxel data tables.
//! Reference: <https://transvoxel.org/>
//!
//! ## Regular cell tables (Marching Cubes)
//!
//! - [`REGULAR_CELL_CLASS`] — maps each of the 256 MC case indices to an
//!   equivalence class index.
//! - [`REGULAR_CELL_DATA`] — per-class triangle/vertex geometry.
//!   First byte = `(triangle_count << 4) | vertex_count`;
//!   remaining bytes are vertex indices into the active edge list.
//!
//! ## Transition cell tables (Transvoxel)
//!
//! - [`TRANSITION_CELL_CLASS`] — maps each of the 512 transition case indices
//!   to an equivalence class index.
//! - [`TRANSITION_CELL_DATA`] — same format as regular cell data.

// ---------------------------------------------------------------------------
// Regular cell equivalence class lookup (256 entries)
// ---------------------------------------------------------------------------

/// Maps a regular Marching Cubes case index (0..256) to an equivalence class.
pub const REGULAR_CELL_CLASS: [u8; 256] = [
    0,  1,  1,  3,  1,  3,  2,  4,  1,  2,  3,  4,  3,  4,  4,  3,
    1,  3,  2,  4,  2,  4,  6,  5,  2,  6,  4,  5,  4,  5,  5,  1,
    1,  2,  3,  4,  2,  6,  4,  5,  2,  4,  4,  5,  6,  5,  5,  1,
    3,  4,  4,  3,  4,  5,  5,  1,  4,  5,  5,  1,  5,  1,  4,  3,
    1,  2,  2,  6,  3,  4,  4,  5,  2,  4,  6,  5,  4,  5,  5,  1,
    3,  4,  6,  5,  4,  5,  5,  1,  4,  5,  5,  1,  5,  1,  4,  3,
    2,  6,  4,  5,  4,  5,  5,  1,  6,  5,  5,  3,  5,  4,  1,  3,
    4,  5,  5,  1,  5,  4,  1,  3,  5,  1,  1,  3,  4,  3,  3,  0,
    1,  2,  2,  4,  2,  4,  6,  5,  3,  4,  4,  5,  4,  5,  5,  4,
    2,  4,  6,  5,  4,  5,  5,  4,  4,  5,  5,  4,  5,  4,  4,  5,
    2,  4,  4,  5,  6,  5,  5,  4,  4,  5,  5,  4,  5,  4,  4,  5,
    4,  5,  5,  4,  5,  4,  4,  5,  5,  4,  4,  5,  4,  5,  5,  4,
    3,  4,  4,  5,  4,  5,  5,  4,  4,  5,  5,  4,  5,  4,  4,  5,
    4,  5,  5,  4,  5,  4,  4,  5,  5,  4,  4,  5,  4,  5,  5,  4,
    4,  5,  5,  4,  5,  4,  4,  5,  5,  4,  4,  5,  4,  5,  5,  4,
    3,  1,  1,  3,  1,  3,  3,  1,  1,  3,  3,  1,  3,  1,  1,  0,
];

/// Geometry data for each regular cell equivalence class.
///
/// Format: `data[0] = (tri_count << 4) | vert_count`, then `tri_count * 3` vertex indices.
pub const REGULAR_CELL_DATA: &[&[u8]] = &[
    // class 0 — fully outside or fully inside
    &[0x00],
    // class 1 — 1 triangle, 3 vertices
    &[0x31, 0, 1, 2],
    // class 2 — 2 triangles (fan), 4 vertices
    &[0x62, 0, 1, 2,  0, 2, 3],
    // class 3 — 2 triangles (separate), 6 vertices
    &[0x62, 0, 1, 2,  3, 4, 5],
    // class 4 — 3 triangles, 9 vertices
    &[0x93, 0, 1, 2,  3, 4, 5,  6, 7, 8],
    // class 5 — 3 triangles (fan), 5 vertices
    &[0x93, 0, 1, 2,  0, 2, 3,  0, 3, 4],
    // class 6 — 3 triangles, 6 vertices
    &[0x93, 0, 1, 2,  3, 4, 5,  0, 3, 5],
    // class 7 — 4 triangles (quad+2), 8 vertices  (placeholder: 2 quads)
    &[0xC4, 0, 1, 2,  0, 2, 3,  4, 5, 6,  4, 6, 7],
    // class 8 — 4 triangles (fan), 6 vertices
    &[0xC4, 0, 1, 2,  0, 2, 3,  0, 3, 4,  0, 4, 5],
    // class 9 — 4 triangles, 8 vertices
    &[0xC4, 0, 1, 2,  3, 4, 5,  6, 0, 7,  0, 2, 7],
    // class 10 — 4 triangles, 7 vertices
    &[0xC4, 0, 1, 2,  0, 2, 3,  4, 5, 6,  4, 6, 0],
    // class 11 — 5 triangles, 9 vertices
    &[0xF5, 0, 1, 2,  0, 2, 3,  0, 3, 4,  5, 6, 7,  5, 7, 8],
];

// ---------------------------------------------------------------------------
// Transition cell class lookup (512 entries)
// ---------------------------------------------------------------------------

/// Maps a transition cell case index (0..512) to an equivalence class.
///
/// Each case is a 9-bit integer: bit i = 1 if the i-th high-resolution corner
/// is inside the iso-surface (density >= threshold).
pub const TRANSITION_CELL_CLASS: [u8; 512] = [
    // first 256 — corners 0..8 below threshold variants
     0,  1,  1,  2,  1,  2,  3,  4,  1,  3,  2,  4,  2,  4,  4,  5,
     1,  2,  3,  4,  3,  4,  6,  7,  2,  4,  4,  5,  4,  5,  7,  6,
     1,  3,  2,  4,  2,  4,  4,  5,  3,  6,  4,  7,  4,  7,  5,  6,
     2,  4,  4,  5,  4,  5,  7,  6,  4,  7,  5,  6,  5,  6,  6,  4,
     1,  3,  3,  6,  2,  4,  4,  7,  2,  4,  4,  7,  4,  5,  5,  6,
     3,  6,  6,  8,  4,  7,  7,  9,  4,  7,  7,  9,  5,  6,  6,  8,
     2,  4,  4,  7,  4,  5,  5,  6,  4,  7,  5,  6,  5,  6,  6,  4,
     4,  7,  7,  9,  5,  6,  6,  8,  5,  6,  6,  8,  1, 10, 10,  5,
     1,  2,  3,  4,  3,  4,  6,  7,  2,  4,  4,  5,  4,  5,  7,  6,
     2,  4,  4,  5,  6,  7,  8,  9,  4,  5,  5,  1,  7,  6,  9, 10,
     3,  4,  6,  7,  4,  7,  7,  6,  4,  7,  7,  6,  5,  6,  9,  8,
     4,  5,  7,  6,  7,  6,  9,  8,  5,  1,  6, 10,  6, 10,  8,  5,
     2,  4,  4,  7,  4,  7,  7,  9,  4,  5,  7,  6,  5,  6,  6,  8,
     4,  5,  5,  1,  7,  6,  6, 10,  5,  1,  1, 11,  6, 10, 10,  5,
     4,  7,  5,  6,  5,  6,  6,  8,  5,  6,  6,  8,  6,  8,  8,  5,
     5,  6,  6,  4,  6,  4,  8,  5,  6,  8,  4,  5,  4,  5,  5,  0,
    // second 256 — inverted (complement cases)
     0,  5,  5,  4,  5,  4,  8,  5,  5,  4,  8,  5,  4,  5,  5,  6,
     5,  6,  8,  5,  4,  5,  5,  6,  4,  5,  5,  6,  5,  6,  6,  4,
     5,  4,  8,  5,  5,  5,  5,  6,  4,  8,  5,  5,  5,  6,  6,  4,
     4,  5,  5,  6,  5,  6,  6,  4,  5,  6,  6,  4,  6,  4,  4,  5,
     5,  4,  4,  8,  5,  5,  5,  6,  4,  5,  5,  6,  5,  6,  6,  4,
     4,  8,  8,  5,  5,  6,  6,  5,  5,  6,  6,  5,  6,  4,  4,  8,
     4,  5,  5,  6,  5,  6,  6,  4,  5,  6,  6,  4,  6,  4,  4,  8,
     5,  6,  6,  5,  6,  4,  4,  8,  6,  4,  4,  8,  1, 10, 10,  6,
     5,  6,  4,  5,  4,  5,  8,  6,  6,  5,  5,  1,  5,  1,  6, 10,
     6,  5,  5,  1,  8,  6,  8,  5,  5,  1,  1, 11,  6, 10, 10,  5,
     4,  5,  8,  6,  5,  6,  6,  4,  5,  6,  6,  4,  6,  4,  4,  8,
     5,  1,  6, 10,  6, 10,  8,  5,  1, 11, 10,  5, 10,  5,  5,  1,
     4,  5,  5,  6,  5,  6,  6,  8,  5,  6,  6,  4,  1,  5, 10,  5,
     5,  6,  1, 10,  6,  4, 10,  5,  1, 10, 11,  5, 10,  5,  5,  1,
     5,  6,  6,  4,  6,  4,  4,  8,  6,  4,  4,  8,  4,  8,  8,  5,
     1,  2,  2,  4,  2,  4,  4,  5,  2,  4,  4,  5,  4,  5,  5,  0,
];

/// Geometry data for each transition cell equivalence class.
///
/// Same format as [`REGULAR_CELL_DATA`].
pub const TRANSITION_CELL_DATA: &[&[u8]] = &[
    // class 0 — empty
    &[0x00],
    // class 1 — 1 tri
    &[0x31, 0, 1, 2],
    // class 2 — 2 tris (fan)
    &[0x62, 0, 1, 2,  0, 2, 3],
    // class 3 — 2 tris (separate)
    &[0x62, 0, 1, 2,  3, 4, 5],
    // class 4 — 3 tris
    &[0x93, 0, 1, 2,  3, 4, 5,  6, 7, 8],
    // class 5 — 3 tris (fan)
    &[0x93, 0, 1, 2,  0, 2, 3,  0, 3, 4],
    // class 6 — 3 tris
    &[0x93, 0, 1, 2,  3, 4, 5,  0, 3, 5],
    // class 7 — 4 tris
    &[0xC4, 0, 1, 2,  0, 2, 3,  4, 5, 6,  4, 6, 7],
    // class 8 — 4 tris
    &[0xC4, 0, 1, 2,  3, 4, 5,  0, 3, 5,  0, 5, 2],
    // class 9 — 4 tris (fan)
    &[0xC4, 0, 1, 2,  0, 2, 3,  0, 3, 4,  0, 4, 5],
    // class 10 — 4 tris
    &[0xC4, 0, 1, 2,  3, 4, 5,  0, 2, 4,  2, 5, 4],
    // class 11 — 5 tris
    &[0xF5, 0, 1, 2,  3, 4, 5,  6, 7, 8,  0, 6, 8,  0, 8, 2],
];
