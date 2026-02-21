//! Integration tests for the Transvoxel extraction pipeline.

use transvoxel::prelude::*;
use transvoxel::transition_sides::TransitionSide;

fn sphere(x: f32, y: f32, z: f32) -> f32 {
    5.0 - (x * x + y * y + z * z).sqrt()
}

/// Bumpy sphere SDF matching the demo (sphere + sinusoidal noise).
fn bumpy_sphere(x: f32, y: f32, z: f32) -> f32 {
    sphere(x, y, z) + 0.3 * (x * 3.0).sin() * (y * 3.0).cos() * (z * 3.0).sin()
}

#[test]
fn empty_block_outside_sphere() {
    let block = Block::new([20.0, 20.0, 20.0], 2.0, 4);
    let mesh = extract_mesh(&sphere, &block, 0.0, TransitionSides::empty());
    assert_eq!(mesh.triangle_count(), 0);
    assert_eq!(mesh.vertex_count(), 0);
}

#[test]
fn sphere_produces_geometry() {
    // Use 16 subdivisions so the sphere surface is well sampled (matches demo).
    let block = Block::new([-6.0, -6.0, -6.0], 12.0, 16);
    let mesh = extract_mesh(&sphere, &block, 0.0, TransitionSides::empty());
    assert!(mesh.triangle_count() > 0, "expected triangles");
    assert!(mesh.vertex_count() > 0, "expected vertices");
}

#[test]
fn indices_in_bounds() {
    let block = Block::new([-6.0, -6.0, -6.0], 12.0, 16);
    let mesh = extract_mesh(&sphere, &block, 0.0, TransitionSides::empty());
    let vc = mesh.vertex_count() as u32;
    for &idx in &mesh.indices {
        assert!(idx < vc, "index {} out of range (vertex_count={})", idx, vc);
    }
}

#[test]
fn array_sizes_consistent() {
    let block = Block::new([-6.0, -6.0, -6.0], 12.0, 16);
    let mesh = extract_mesh(&sphere, &block, 0.0, TransitionSides::empty());
    assert_eq!(mesh.positions.len(), mesh.vertex_count() * 3);
    assert_eq!(mesh.normals.len(), mesh.vertex_count() * 3);
    assert_eq!(mesh.indices.len(), mesh.triangle_count() * 3);
}

#[test]
fn normals_unit_length() {
    let block = Block::new([-6.0, -6.0, -6.0], 12.0, 16);
    let mesh = extract_mesh(&sphere, &block, 0.0, TransitionSides::empty());
    for i in 0..mesh.vertex_count() {
        let n = &mesh.normals[i * 3..i * 3 + 3];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-3, "normal {} len={}", i, len);
    }
}

#[test]
fn transition_lowx_executes() {
    let block = Block::new([-6.0, -6.0, -6.0], 12.0, 16);
    let sides = TransitionSides::from(TransitionSide::LowX);
    let mesh = extract_mesh(&sphere, &block, 0.0, sides);
    assert!(mesh.triangle_count() > 0);
}

#[test]
fn all_transitions_executes() {
    let block = Block::new([-6.0, -6.0, -6.0], 12.0, 16);
    let mesh = extract_mesh(&sphere, &block, 0.0, TransitionSides::all());
    assert!(mesh.triangle_count() > 0);
}

#[test]
fn threshold_excludes_all() {
    // A very high threshold means nothing is inside — empty mesh
    let block = Block::new([-6.0, -6.0, -6.0], 12.0, 4);
    let mesh = extract_mesh(&sphere, &block, 1000.0, TransitionSides::empty());
    assert_eq!(mesh.triangle_count(), 0);
}

/// Asserts the demo produces the exact vertex/triangle counts and first vertices/face
/// (matches output of `cargo run --bin demo`).
#[test]
fn demo_output_values() {
    let block = Block::new([-6.0, -6.0, -6.0], 12.0, 16);
    let threshold = 0.0_f32;
    const EPS: f32 = 1e-4;

    // 1. Sphere — no transitions
    let m = extract_mesh(&sphere, &block, threshold, TransitionSides::empty());
    assert_eq!(m.vertex_count(), 822, "sphere no transitions vertices");
    assert_eq!(m.triangle_count(), 1640, "sphere no transitions triangles");

    // 2. Sphere — transition LowX
    let m = extract_mesh(
        &sphere,
        &block,
        threshold,
        TransitionSides::from(TransitionSide::LowX),
    );
    assert_eq!(m.vertex_count(), 822, "sphere LowX vertices");
    assert_eq!(m.triangle_count(), 1640, "sphere LowX triangles");

    // 3. Sphere — transitions LowX+HighY+LowZ
    let sides = TransitionSides::empty()
        | TransitionSide::LowX
        | TransitionSide::HighY
        | TransitionSide::LowZ;
    let m = extract_mesh(&sphere, &block, threshold, sides);
    assert_eq!(m.vertex_count(), 822, "sphere 3 transitions vertices");
    assert_eq!(m.triangle_count(), 1640, "sphere 3 transitions triangles");

    // 4. Bumpy sphere — all 6 transitions
    let m = extract_mesh(&bumpy_sphere, &block, threshold, TransitionSides::all());
    assert_eq!(m.vertex_count(), 862, "bumpy sphere all transitions vertices");
    assert_eq!(m.triangle_count(), 1720, "bumpy sphere all transitions triangles");

    // 5. First 6 vertices and first face (sphere, no transitions)
    let m = extract_mesh(&sphere, &block, threshold, TransitionSides::empty());
    let expected_verts: [(f32, f32, f32); 6] = [
        (-1.5000, -1.5000, -4.5273),
        (-1.5000, -1.5683, -4.5000),
        (-1.5683, -1.5000, -4.5000),
        (-0.7500, -1.5000, -4.7091),
        (-0.7500, -2.0212, -4.5000),
        (0.0000, -1.5000, -4.7685),
    ];
    for (i, (ex, ey, ez)) in expected_verts.iter().enumerate() {
        assert!(
            (m.positions[i * 3] - ex).abs() < EPS
                && (m.positions[i * 3 + 1] - ey).abs() < EPS
                && (m.positions[i * 3 + 2] - ez).abs() < EPS,
            "vertex {}: expected ({}, {}, {}), got ({}, {}, {})",
            i,
            ex,
            ey,
            ez,
            m.positions[i * 3],
            m.positions[i * 3 + 1],
            m.positions[i * 3 + 2]
        );
    }
    assert!(m.indices.len() >= 3, "mesh has at least one triangle");
    assert_eq!(
        [m.indices[0], m.indices[1], m.indices[2]],
        [0, 1, 2],
        "first face is f 1 2 3 (OBJ 1-based)"
    );
}
