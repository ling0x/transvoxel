//! Core mesh extraction: Marching Cubes for regular cells + Transvoxel for transition cells.

use crate::block::Block;
use crate::mesh::Mesh;
use crate::tables::{
    REGULAR_CELL_CLASS, REGULAR_CELL_DATA, TRANSITION_CELL_CLASS, TRANSITION_CELL_DATA,
};
use crate::transition_sides::{TransitionSide, TransitionSides};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Public extraction entry point
// ---------------------------------------------------------------------------

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

    // Hash map to reuse vertices dynamically generated on grid edges.
    // The key is the exact edge coordinate integer tuple ((x1, y1, z1), (x2, y2, z2)).
    // A point `(cx, cy, cz)` is mapped to `cx * 2` to accommodate half-voxels from transition cells.
    let mut cache: HashMap<([i32; 3], [i32; 3]), u32> = HashMap::new();

    // --- Regular cells (Marching Cubes) ---
    for iz in 0..n {
        for iy in 0..n {
            for ix in 0..n {
                extract_regular_cell(
                    field,
                    block,
                    threshold,
                    &transitions,
                    ix,
                    iy,
                    iz,
                    &mut mesh,
                    &mut cache,
                );
            }
        }
    }

    // --- Transition cells (Transvoxel) ---
    for side in TransitionSide::ALL {
        if transitions.contains(side) {
            extract_transition_face(field, block, threshold, side, &mut mesh, &mut cache);
        }
    }

    mesh
}

// ---------------------------------------------------------------------------
// Regular cell (Marching Cubes)
// ---------------------------------------------------------------------------

// Standard Transvoxel edge-to-corner mapping (following Lengyel ordering).
const EDGE_CORNERS: [(usize, usize); 12] = [
    (0, 1),
    (1, 3),
    (3, 2),
    (2, 0), // bottom edges (z=0)
    (4, 5),
    (5, 7),
    (7, 6),
    (6, 4), // top edges (z=1)
    (0, 4),
    (1, 5),
    (3, 7),
    (2, 6), // vertical edges
];

fn extract_regular_cell<F>(
    field: &F,
    block: &Block,
    threshold: f32,
    transitions: &TransitionSides,
    ix: usize,
    iy: usize,
    iz: usize,
    mesh: &mut Mesh,
    cache: &mut HashMap<([i32; 3], [i32; 3]), u32>,
) where
    F: Fn(f32, f32, f32) -> f32,
{
    if is_boundary_cell(ix, iy, iz, block.subdivisions, transitions) {
        return;
    }

    // Lengyel's Corner index mapping: x | (y << 1) | (z << 2)
    let corners: [[usize; 3]; 8] = [
        [ix, iy, iz],             // 0: 000
        [ix + 1, iy, iz],         // 1: 100
        [ix, iy + 1, iz],         // 2: 010
        [ix + 1, iy + 1, iz],     // 3: 110
        [ix, iy, iz + 1],         // 4: 001
        [ix + 1, iy, iz + 1],     // 5: 101
        [ix, iy + 1, iz + 1],     // 6: 011
        [ix + 1, iy + 1, iz + 1], // 7: 111
    ];

    let densities: [f32; 8] = std::array::from_fn(|i| {
        let [cx, cy, cz] = corners[i];
        let p = block.voxel_position(cx, cy, cz);
        field(p[0], p[1], p[2])
    });

    // Transvoxel corner ordering (z-major, then y, then x):
    // bit 0: (0,0,0), bit 1: (1,0,0), bit 2: (0,1,0), bit 3: (1,1,0)
    // bit 4: (0,0,1), bit 5: (1,0,1), bit 6: (0,1,1), bit 7: (1,1,1)
    let case_index = (0..8).fold(0usize, |acc, i| {
        // Since corners array maps index exactly to this binary mapping:
        // Ex: idx 5 = [1,0,1] => bit 5.
        if densities[i] < threshold {
            acc | (1 << i)
        } else {
            acc
        }
    });

    if case_index == 0 || case_index == 255 {
        return;
    }

    let class_idx = REGULAR_CELL_CLASS[case_index] as usize;
    let cell_data = REGULAR_CELL_DATA[class_idx.min(REGULAR_CELL_DATA.len() - 1)];
    if cell_data.len() < 1 {
        return;
    }

    let tri_count = (cell_data[0] & 0x0F) as usize; // Low nibble is triangle count in Transvoxel.cpp
    if tri_count == 0 || cell_data.len() < 1 + tri_count * 3 {
        return;
    }

    let mut edge_verts: [u32; 12] = [u32::MAX; 12];
    for (edge_idx, &(ca, cb)) in EDGE_CORNERS.iter().enumerate() {
        let da = densities[ca];
        let db = densities[cb];
        if (da >= threshold) == (db >= threshold) {
            continue;
        }

        let mut p_a_int = [
            corners[ca][0] as i32 * 2,
            corners[ca][1] as i32 * 2,
            corners[ca][2] as i32 * 2,
        ];
        let mut p_b_int = [
            corners[cb][0] as i32 * 2,
            corners[cb][1] as i32 * 2,
            corners[cb][2] as i32 * 2,
        ];
        if p_a_int > p_b_int {
            std::mem::swap(&mut p_a_int, &mut p_b_int);
        }
        let edge_key = (p_a_int, p_b_int);

        if let Some(&v_idx) = cache.get(&edge_key) {
            edge_verts[edge_idx] = v_idx;
        } else {
            let t = ((threshold - da) / (db - da)).clamp(0.0, 1.0);
            let pa = block.voxel_position(corners[ca][0], corners[ca][1], corners[ca][2]);
            let pb = block.voxel_position(corners[cb][0], corners[cb][1], corners[cb][2]);
            let pos = lerp3(pa, pb, t);
            let normal = gradient_normal(field, block.voxel_step(), pos);
            let v_idx = mesh.push_vertex(pos, normal);
            cache.insert(edge_key, v_idx);
            edge_verts[edge_idx] = v_idx;
        }
    }

    emit_triangles(cell_data, tri_count, &edge_verts, mesh, false);
}

// ---------------------------------------------------------------------------
// Transition cell (Transvoxel Algorithm)
// ---------------------------------------------------------------------------

// Edges of the high-resolution 3×3 face (9 voxels, 12 edges).
// Numbered according to Figure 4.16 in Transvoxel paper.
const TC_EDGE_CORNERS: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 5),
    (5, 8),
    (8, 7),
    (7, 6),
    (6, 3),
    (3, 0),
    (1, 4),
    (4, 7),
    (3, 4),
    (4, 5),
];

fn extract_transition_face<F>(
    field: &F,
    block: &Block,
    threshold: f32,
    side: TransitionSide,
    mesh: &mut Mesh,
    cache: &mut HashMap<([i32; 3], [i32; 3]), u32>,
) where
    F: Fn(f32, f32, f32) -> f32,
{
    let n = block.subdivisions;
    let step = block.voxel_step();
    let (ax, ay) = side.face_axes();
    let (norm_axis, sign) = side.normal_axis_sign();
    let fixed_idx = if sign < 0.0 { 0usize } else { n };

    for cell_v in 0..n {
        for cell_u in 0..n {
            let mut densities = [0.0f32; 9];
            let mut positions = [[0.0f32; 3]; 9];
            let mut int_coords = [[0i32; 3]; 9];

            for row in 0..3usize {
                for col in 0..3usize {
                    let sub_u = cell_u * 2 + col;
                    let sub_v = cell_v * 2 + row;
                    let u_int = sub_u / 2;
                    let v_int = sub_v / 2;
                    let u_frac = if sub_u % 2 == 1 { 0.5 } else { 0.0 };
                    let v_frac = if sub_v % 2 == 1 { 0.5 } else { 0.0 };

                    let mut idx3 = [0usize; 3];
                    idx3[ax] = u_int.min(n);
                    idx3[ay] = v_int.min(n);
                    idx3[norm_axis] = fixed_idx;

                    let base = block.voxel_position(idx3[0], idx3[1], idx3[2]);
                    let mut pos = base;
                    pos[ax] += u_frac * step;
                    pos[ay] += v_frac * step;

                    let si = row * 3 + col;
                    positions[si] = pos;
                    densities[si] = field(pos[0], pos[1], pos[2]);

                    let mut int_pos = [0i32; 3];
                    int_pos[ax] = sub_u as i32;
                    int_pos[ay] = sub_v as i32;
                    int_pos[norm_axis] = (fixed_idx * 2) as i32;
                    int_coords[si] = int_pos;
                }
            }

            let case_index = (0..9).fold(0usize, |acc, i| {
                if densities[i] < threshold {
                    acc | (1 << i)
                } else {
                    acc
                }
            });

            if case_index == 0 || case_index == 511 {
                continue;
            }

            let class_val = TRANSITION_CELL_CLASS[case_index];
            let invert = (class_val & 0x80) != 0;
            let class_idx = (class_val & 0x7F) as usize;

            let cell_data = TRANSITION_CELL_DATA[class_idx.min(TRANSITION_CELL_DATA.len() - 1)];
            if cell_data.len() < 1 {
                continue;
            }

            let tri_count = (cell_data[0] & 0x0F) as usize;
            if tri_count == 0 || cell_data.len() < 1 + tri_count * 3 {
                continue;
            }

            let mut edge_verts: [u32; 12] = [u32::MAX; 12];
            for (edge_idx, &(ca, cb)) in TC_EDGE_CORNERS.iter().enumerate() {
                let da = densities[ca];
                let db = densities[cb];
                if (da >= threshold) == (db >= threshold) {
                    continue;
                }

                let mut p_a_int = int_coords[ca];
                let mut p_b_int = int_coords[cb];
                if p_a_int > p_b_int {
                    std::mem::swap(&mut p_a_int, &mut p_b_int);
                }
                let edge_key = (p_a_int, p_b_int);

                if let Some(&v_idx) = cache.get(&edge_key) {
                    edge_verts[edge_idx] = v_idx;
                } else {
                    let t = ((threshold - da) / (db - da)).clamp(0.0, 1.0);
                    let pos = lerp3(positions[ca], positions[cb], t);
                    let normal = gradient_normal(field, step, pos);
                    let v_idx = mesh.push_vertex(pos, normal);
                    cache.insert(edge_key, v_idx);
                    edge_verts[edge_idx] = v_idx;
                }
            }

            emit_triangles(cell_data, tri_count, &edge_verts, mesh, invert);
        }
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

#[inline]
fn emit_triangles(
    cell_data: &[u8],
    tri_count: usize,
    edge_verts: &[u32; 12],
    mesh: &mut Mesh,
    invert: bool,
) {
    for tri in 0..tri_count {
        let base = 1 + tri * 3;
        if base + 2 >= cell_data.len() {
            break;
        }
        let a = cell_data[base] as usize;
        let mut b = cell_data[base + 1] as usize;
        let mut c = cell_data[base + 2] as usize;

        if invert {
            std::mem::swap(&mut b, &mut c);
        }

        if a < 12
            && b < 12
            && c < 12
            && edge_verts[a] != u32::MAX
            && edge_verts[b] != u32::MAX
            && edge_verts[c] != u32::MAX
        {
            mesh.push_triangle(edge_verts[a], edge_verts[b], edge_verts[c]);
        }
    }
}

#[inline]
fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

#[inline]
fn gradient_normal<F>(field: &F, step: f32, pos: [f32; 3]) -> [f32; 3]
where
    F: Fn(f32, f32, f32) -> f32,
{
    let e = step * 0.1;
    let dx = field(pos[0] + e, pos[1], pos[2]) - field(pos[0] - e, pos[1], pos[2]);
    let dy = field(pos[0], pos[1] + e, pos[2]) - field(pos[0], pos[1] - e, pos[2]);
    let dz = field(pos[0], pos[1], pos[2] + e) - field(pos[0], pos[1], pos[2] - e);
    normalize3([dx, dy, dz])
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len_sq = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if len_sq < 1e-20 {
        [0.0, 1.0, 0.0]
    } else {
        let inv = 1.0 / len_sq.sqrt();
        [v[0] * inv, v[1] * inv, v[2] * inv]
    }
}

#[inline]
fn is_boundary_cell(ix: usize, iy: usize, iz: usize, n: usize, ts: &TransitionSides) -> bool {
    (ts.contains(TransitionSide::LowX) && ix == 0)
        || (ts.contains(TransitionSide::HighX) && ix == n - 1)
        || (ts.contains(TransitionSide::LowY) && iy == 0)
        || (ts.contains(TransitionSide::HighY) && iy == n - 1)
        || (ts.contains(TransitionSide::LowZ) && iz == 0)
        || (ts.contains(TransitionSide::HighZ) && iz == n - 1)
}
