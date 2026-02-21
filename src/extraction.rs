//! Core mesh extraction: Marching Cubes for regular cells + Transvoxel for transition cells.
//!
//! ## Regular cell extraction
//!
//! For each `n×n×n` cell in the block:
//! 1. Sample density at each of the 8 corners.
//! 2. Build an 8-bit case index (bit `i` = 1 if corner `i` is inside).
//! 3. Look up the equivalence class and triangle pattern from the tables.
//! 4. For each of the 12 possible edges, if it is crossed by the iso-surface,
//!    interpolate a vertex and store it in `edge_verts[edge_index]`.
//! 5. Emit triangles by indexing into `edge_verts` with the table's vertex indices.
//!
//! ## Transition cell extraction
//!
//! For each active transition face:
//! 1. Iterate over all `n×n` transition cells on that face.
//! 2. Sample density at each of the 9 high-resolution corners (3×3 grid).
//! 3. Build a 9-bit case index.
//! 4. Same edge-slot mapping as above, using the 12 TC edges.

use crate::block::Block;
use crate::mesh::Mesh;
use crate::tables::{REGULAR_CELL_CLASS, REGULAR_CELL_DATA, TRANSITION_CELL_CLASS, TRANSITION_CELL_DATA};
use crate::transition_sides::{TransitionSide, TransitionSides};

// ---------------------------------------------------------------------------
// Public extraction entry point
// ---------------------------------------------------------------------------

/// Extract a mesh for the given block and density field.
///
/// # Arguments
/// * `field`       - Density function `(x, y, z) -> f32`. Values ≥ `threshold` are *inside*.
/// * `block`       - The cubic region of space to triangulate.
/// * `threshold`   - Iso-surface density value.
/// * `transitions` - Which block faces should receive Transvoxel transition cells.
pub fn extract_mesh<F>(
    field: &F,
    block: &Block,
    threshold: f32,
    transitions: TransitionSides,
) -> Mesh
where
    F: Fn(f32, f32, f32) -> f32,
{
    let mut mesh = Mesh::new();
    let n = block.subdivisions;

    // --- Regular cells (Marching Cubes) ---
    for iz in 0..n {
        for iy in 0..n {
            for ix in 0..n {
                extract_regular_cell(field, block, threshold, &transitions, ix, iy, iz, &mut mesh);
            }
        }
    }

    // --- Transition cells (Transvoxel) ---
    for side in TransitionSide::ALL {
        if transitions.contains(side) {
            extract_transition_face(field, block, threshold, side, &mut mesh);
        }
    }

    mesh
}

// ---------------------------------------------------------------------------
// Regular cell (Marching Cubes)
// ---------------------------------------------------------------------------

// Standard Marching Cubes edge-to-corner mapping.
// Edge index → (corner_a, corner_b).
// Corners are numbered with bit0=+X, bit1=+Y, bit2=+Z:
//   0=(0,0,0)  1=(1,0,0)  2=(0,1,0)  3=(1,1,0)
//   4=(0,0,1)  5=(1,0,1)  6=(0,1,1)  7=(1,1,1)
const EDGE_CORNERS: [(usize, usize); 12] = [
    (0, 1), (1, 3), (2, 3), (0, 2),  // bottom face edges
    (4, 5), (5, 7), (6, 7), (4, 6),  // top face edges
    (0, 4), (1, 5), (3, 7), (2, 6),  // vertical edges
];

fn extract_regular_cell<F>(
    field: &F,
    block: &Block,
    threshold: f32,
    transitions: &TransitionSides,
    ix: usize, iy: usize, iz: usize,
    mesh: &mut Mesh,
)
where
    F: Fn(f32, f32, f32) -> f32,
{
    // Skip boundary cells that will be replaced by transition geometry
    if is_boundary_cell(ix, iy, iz, block.subdivisions, transitions) {
        return;
    }

    // 8 cell corners in voxel-grid coordinates
    let corners: [[usize; 3]; 8] = [
        [ix,   iy,   iz  ],
        [ix+1, iy,   iz  ],
        [ix,   iy+1, iz  ],
        [ix+1, iy+1, iz  ],
        [ix,   iy,   iz+1],
        [ix+1, iy,   iz+1],
        [ix,   iy+1, iz+1],
        [ix+1, iy+1, iz+1],
    ];

    // Sample density at each corner
    let densities: [f32; 8] = std::array::from_fn(|i| {
        let [cx, cy, cz] = corners[i];
        let p = block.voxel_position(cx, cy, cz);
        field(p[0], p[1], p[2])
    });

    // 8-bit case index: bit i set if corner i is inside
    let case_index = (0..8).fold(0usize, |acc, i| {
        if densities[i] >= threshold { acc | (1 << i) } else { acc }
    });

    if case_index == 0 || case_index == 255 {
        return; // fully outside or fully inside
    }

    let class_idx = REGULAR_CELL_CLASS[case_index] as usize;
    let cell_data = REGULAR_CELL_DATA[class_idx.min(REGULAR_CELL_DATA.len() - 1)];
    if cell_data.len() < 1 { return; }

    let tri_count  = ((cell_data[0] >> 4) & 0xF) as usize;
    let vert_count = (cell_data[0] & 0xF) as usize;
    if tri_count == 0 || cell_data.len() < 1 + tri_count * 3 { return; }

    // For each of the 12 edges, compute an interpolated vertex if the edge is
    // crossed by the iso-surface. Store the mesh vertex index in edge_verts[edge].
    // Uncrossed edges stay as u32::MAX (sentinel).
    let mut edge_verts: [u32; 12] = [u32::MAX; 12];

    for (edge_idx, &(ca, cb)) in EDGE_CORNERS.iter().enumerate() {
        let da = densities[ca];
        let db = densities[cb];
        if (da >= threshold) == (db >= threshold) {
            continue; // edge not crossed
        }
        let t  = ((threshold - da) / (db - da)).clamp(0.0, 1.0);
        let pa = block.voxel_position(corners[ca][0], corners[ca][1], corners[ca][2]);
        let pb = block.voxel_position(corners[cb][0], corners[cb][1], corners[cb][2]);
        let pos    = lerp3(pa, pb, t);
        let normal = gradient_normal(field, block.voxel_step(), pos);
        edge_verts[edge_idx] = mesh.push_vertex(pos, normal);
    }

    // Emit triangles: each vertex index in cell_data references an edge slot
    emit_triangles(cell_data, tri_count, vert_count, &edge_verts, mesh);
}

// ---------------------------------------------------------------------------
// Transition cell (Transvoxel Algorithm)
// ---------------------------------------------------------------------------

// Edges of the high-resolution 3×3 face (9 voxels, 12 edges).
// Row-major index: voxel at (row, col) = row*3 + col.
const TC_EDGE_CORNERS: [(usize, usize); 12] = [
    (0, 1), (1, 2),           // top row, horizontal
    (3, 4), (4, 5),           // middle row, horizontal
    (6, 7), (7, 8),           // bottom row, horizontal
    (0, 3), (1, 4), (2, 5),   // columns, upper half
    (3, 6), (4, 7), (5, 8),   // columns, lower half
];

fn extract_transition_face<F>(
    field: &F,
    block: &Block,
    threshold: f32,
    side: TransitionSide,
    mesh: &mut Mesh,
)
where
    F: Fn(f32, f32, f32) -> f32,
{
    let n    = block.subdivisions;
    let step = block.voxel_step();
    let (ax, ay)        = side.face_axes();
    let (norm_axis, sign) = side.normal_axis_sign();
    let fixed_idx       = if sign < 0.0 { 0usize } else { n };

    for cell_v in 0..n {
        for cell_u in 0..n {
            // Build the 3×3 grid of high-resolution samples for this transition cell.
            let mut densities = [0.0f32; 9];
            let mut positions = [[0.0f32; 3]; 9];

            for row in 0..3usize {
                for col in 0..3usize {
                    // sub-grid coordinates (0..2n range)
                    let sub_u = cell_u * 2 + col;
                    let sub_v = cell_v * 2 + row;

                    // Integer voxel index and fractional offset
                    let u_int  = sub_u / 2;
                    let v_int  = sub_v / 2;
                    let u_frac = if sub_u % 2 == 1 { 0.5 } else { 0.0 };
                    let v_frac = if sub_v % 2 == 1 { 0.5 } else { 0.0 };

                    let mut idx3 = [0usize; 3];
                    idx3[ax]        = u_int.min(n);
                    idx3[ay]        = v_int.min(n);
                    idx3[norm_axis] = fixed_idx;

                    let base = block.voxel_position(idx3[0], idx3[1], idx3[2]);
                    let mut pos = base;
                    pos[ax] += u_frac * step;
                    pos[ay] += v_frac * step;

                    let si = row * 3 + col;
                    positions[si] = pos;
                    densities[si] = field(pos[0], pos[1], pos[2]);
                }
            }

            // 9-bit case index
            let case_index = (0..9).fold(0usize, |acc, i| {
                if densities[i] >= threshold { acc | (1 << i) } else { acc }
            });

            if case_index == 0 || case_index == 511 {
                continue;
            }

            let class_idx = TRANSITION_CELL_CLASS[case_index] as usize;
            let cell_data = TRANSITION_CELL_DATA[class_idx.min(TRANSITION_CELL_DATA.len() - 1)];
            if cell_data.len() < 1 { continue; }

            let tri_count  = ((cell_data[0] >> 4) & 0xF) as usize;
            let vert_count = (cell_data[0] & 0xF) as usize;
            if tri_count == 0 || cell_data.len() < 1 + tri_count * 3 { continue; }

            // Compute interpolated vertices for each crossed TC edge
            let mut edge_verts: [u32; 12] = [u32::MAX; 12];

            for (edge_idx, &(ca, cb)) in TC_EDGE_CORNERS.iter().enumerate() {
                let da = densities[ca];
                let db = densities[cb];
                if (da >= threshold) == (db >= threshold) {
                    continue;
                }
                let t   = ((threshold - da) / (db - da)).clamp(0.0, 1.0);
                let pos = lerp3(positions[ca], positions[cb], t);
                let normal = gradient_normal(field, step, pos);
                edge_verts[edge_idx] = mesh.push_vertex(pos, normal);
            }

            emit_triangles(cell_data, tri_count, vert_count, &edge_verts, mesh);
        }
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Emit triangles from a cell data table entry.
/// `verts[i]` = mesh vertex index for edge slot `i`.
/// Vertex indices in `cell_data` directly reference edge slots.
#[inline]
fn emit_triangles(
    cell_data:  &[u8],
    tri_count:  usize,
    vert_count: usize,
    verts:      &[u32; 12],
    mesh:       &mut Mesh,
) {
    for tri in 0..tri_count {
        let base = 1 + tri * 3;
        if base + 2 >= cell_data.len() { break; }
        let a = cell_data[base    ] as usize;
        let b = cell_data[base + 1] as usize;
        let c = cell_data[base + 2] as usize;
        // Guard: indices must be within vert_count and the edge slot must be filled
        if a < vert_count && b < vert_count && c < vert_count
            && verts[a] != u32::MAX
            && verts[b] != u32::MAX
            && verts[c] != u32::MAX
        {
            mesh.push_triangle(verts[a], verts[b], verts[c]);
        }
    }
}

/// Linear interpolation between two 3-D points.
#[inline]
fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Estimate surface normal at `pos` using central-difference gradients.
#[inline]
fn gradient_normal<F>(field: &F, step: f32, pos: [f32; 3]) -> [f32; 3]
where
    F: Fn(f32, f32, f32) -> f32,
{
    let e  = step * 0.1;
    let dx = field(pos[0]+e, pos[1],   pos[2]  ) - field(pos[0]-e, pos[1],   pos[2]  );
    let dy = field(pos[0],   pos[1]+e, pos[2]  ) - field(pos[0],   pos[1]-e, pos[2]  );
    let dz = field(pos[0],   pos[1],   pos[2]+e) - field(pos[0],   pos[1],   pos[2]-e);
    normalize3([dx, dy, dz])
}

/// Normalize a 3-D vector; returns `[0,1,0]` for near-zero input.
#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len_sq = v[0]*v[0] + v[1]*v[1] + v[2]*v[2];
    if len_sq < 1e-20 {
        [0.0, 1.0, 0.0]
    } else {
        let inv = 1.0 / len_sq.sqrt();
        [v[0]*inv, v[1]*inv, v[2]*inv]
    }
}

/// Returns `true` if the cell at `(ix, iy, iz)` lies on an active transition face.
/// Such cells are skipped in regular extraction and handled by the transition pass.
#[inline]
fn is_boundary_cell(
    ix: usize, iy: usize, iz: usize,
    n:  usize,
    ts: &TransitionSides,
) -> bool {
    (ts.contains(TransitionSide::LowX)  && ix == 0    ) ||
    (ts.contains(TransitionSide::HighX) && ix == n - 1) ||
    (ts.contains(TransitionSide::LowY)  && iy == 0    ) ||
    (ts.contains(TransitionSide::HighY) && iy == n - 1) ||
    (ts.contains(TransitionSide::LowZ)  && iz == 0    ) ||
    (ts.contains(TransitionSide::HighZ) && iz == n - 1)
}
