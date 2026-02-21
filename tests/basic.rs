//! Integration tests for the Transvoxel extraction pipeline.

use transvoxel::prelude::*;
use transvoxel::transition_sides::TransitionSide;

fn sphere(x: f32, y: f32, z: f32) -> f32 {
    5.0 - (x * x + y * y + z * z).sqrt()
}

#[test]
fn empty_block_outside_sphere() {
    let block = Block::new([20.0, 20.0, 20.0], 2.0, 4);
    let mesh  = extract_mesh(&sphere, &block, 0.0, TransitionSides::empty());
    assert_eq!(mesh.triangle_count(), 0);
    assert_eq!(mesh.vertex_count(),   0);
}

#[test]
fn sphere_produces_geometry() {
    let block = Block::new([-6.0, -6.0, -6.0], 12.0, 8);
    let mesh  = extract_mesh(&sphere, &block, 0.0, TransitionSides::empty());
    assert!(mesh.triangle_count() > 0, "expected triangles");
    assert!(mesh.vertex_count()   > 0, "expected vertices");
}

#[test]
fn indices_in_bounds() {
    let block = Block::new([-6.0, -6.0, -6.0], 12.0, 8);
    let mesh  = extract_mesh(&sphere, &block, 0.0, TransitionSides::empty());
    let vc    = mesh.vertex_count() as u32;
    for &idx in &mesh.indices {
        assert!(idx < vc, "index {} out of range (vertex_count={})", idx, vc);
    }
}

#[test]
fn array_sizes_consistent() {
    let block = Block::new([-6.0, -6.0, -6.0], 12.0, 8);
    let mesh  = extract_mesh(&sphere, &block, 0.0, TransitionSides::empty());
    assert_eq!(mesh.positions.len(), mesh.vertex_count() * 3);
    assert_eq!(mesh.normals.len(),   mesh.vertex_count() * 3);
    assert_eq!(mesh.indices.len(),   mesh.triangle_count() * 3);
}

#[test]
fn normals_unit_length() {
    let block = Block::new([-6.0, -6.0, -6.0], 12.0, 6);
    let mesh  = extract_mesh(&sphere, &block, 0.0, TransitionSides::empty());
    for i in 0..mesh.vertex_count() {
        let n = &mesh.normals[i*3..i*3+3];
        let len = (n[0]*n[0] + n[1]*n[1] + n[2]*n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-3, "normal {} len={}", i, len);
    }
}

#[test]
fn transition_lowx_executes() {
    let block = Block::new([-6.0, -6.0, -6.0], 12.0, 6);
    let sides = TransitionSides::from(TransitionSide::LowX);
    // should not panic
    let mesh = extract_mesh(&sphere, &block, 0.0, sides);
    assert!(mesh.triangle_count() >= 0);
}

#[test]
fn all_transitions_executes() {
    let block = Block::new([-6.0, -6.0, -6.0], 12.0, 4);
    let mesh  = extract_mesh(&sphere, &block, 0.0, TransitionSides::all());
    assert!(mesh.triangle_count() >= 0);
}

#[test]
fn threshold_excludes_all() {
    // A very high threshold means nothing is inside — empty mesh
    let block = Block::new([-6.0, -6.0, -6.0], 12.0, 4);
    let mesh  = extract_mesh(&sphere, &block, 1000.0, TransitionSides::empty());
    assert_eq!(mesh.triangle_count(), 0);
}
