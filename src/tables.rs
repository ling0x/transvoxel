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
//!   remaining bytes are vertex indices into the 12-edge slot array.
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
/// Format: `data[0] = (tri_count << 4) | vert_count`,
/// then `tri_count * 3` vertex indices (each references one of the 12 edge slots).
///
/// Edge slot indices 0..12 correspond to EDGE_CORNERS in extraction.rs.
pub const REGULAR_CELL_DATA: &[&[u8]] = &[
    // class 0 — empty
    &[0x00],
    // class 1 — 1 triangle, 3 unique vertices
    // edges used: 0,1,2  (bottom-face triangle)
    &[0x13,  0,1,2],
    // class 2 — 2 triangles (quad), 4 vertices
    // edges: 0,1,2,3
    &[0x24,  0,1,2,  0,2,3],
    // class 3 — 2 triangles, 6 vertices (two separate tris)
    // edges: 0,1,2 + 3,4,5
    &[0x26,  0,1,2,  3,4,5],
    // class 4 — 3 triangles, 6 vertices
    &[0x36,  0,1,2,  3,4,5,  0,3,5],
    // class 5 — 3 triangles, 5 vertices (fan)
    &[0x35,  0,1,2,  0,2,3,  0,3,4],
    // class 6 — 3 triangles, 6 vertices
    &[0x36,  0,1,5,  1,4,5,  1,2,4],
    // class 7 — 4 triangles, 6 vertices
    &[0x46,  0,1,2,  0,2,5,  0,5,4,  2,3,5],
    // class 8 — 4 triangles, 6 vertices (fan)
    &[0x46,  0,1,4,  1,3,4,  1,2,3,  0,4,5],
    // class 9 — 4 triangles, 6 vertices
    &[0x46,  0,1,2,  0,2,3,  0,3,4,  0,4,5],
    // class 10 — 4 triangles, 7 vertices
    &[0x47,  0,1,2,  0,2,3,  4,5,6,  4,6,0],
    // class 11 — 5 triangles, 9 vertices
    &[0x59,  0,1,2,  0,2,3,  0,3,4,  5,6,7,  5,7,8],
];

// ---------------------------------------------------------------------------
// Transition cell class lookup (512 entries)
// ---------------------------------------------------------------------------

/// Maps a transition cell case index (0..512) to an equivalence class.
///
/// Each case is a 9-bit integer: bit i = 1 if the i-th high-resolution corner
/// is inside the iso-surface (density >= threshold).
pub const TRANSITION_CELL_CLASS: [u8; 512] = [
    // first 256
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
    // second 256 (complement/inverted)
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
/// Same format as [`REGULAR_CELL_DATA`].
/// Vertex indices reference the 12 TC_EDGE_CORNERS slots in extraction.rs.
pub const TRANSITION_CELL_DATA: &[&[u8]] = &[
    // class 0 — empty
    &[0x00],
    // class 1 — 1 tri, 3 verts
    &[0x13,  0,1,2],
    // class 2 — 2 tris (fan), 4 verts
    &[0x24,  0,1,2,  0,2,3],
    // class 3 — 2 tris (separate), 6 verts
    &[0x26,  0,1,2,  3,4,5],
    // class 4 — 3 tris, 6 verts
    &[0x36,  0,1,2,  3,4,5,  0,3,5],
    // class 5 — 3 tris (fan), 5 verts
    &[0x35,  0,1,2,  0,2,3,  0,3,4],
    // class 6 — 3 tris, 6 verts
    &[0x36,  0,1,5,  1,4,5,  1,2,4],
    // class 7 — 4 tris, 6 verts
    &[0x46,  0,1,2,  0,2,5,  0,5,4,  2,3,5],
    // class 8 — 4 tris, 6 verts
    &[0x46,  0,1,4,  1,3,4,  1,2,3,  0,4,5],
    // class 9 — 4 tris (fan), 6 verts
    &[0x46,  0,1,2,  0,2,3,  0,3,4,  0,4,5],
    // class 10 — 4 tris, 7 verts
    &[0x47,  0,1,2,  3,4,5,  0,2,4,  2,5,4],
    // class 11 — 5 tris, 9 verts
    &[0x59,  0,1,2,  3,4,5,  6,7,8,  0,6,8,  0,8,2],
];
