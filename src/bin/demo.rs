//! Demo binary showcasing the Transvoxel extraction.
//!
//! Run with:  `cargo run --bin demo`

use transvoxel::prelude::*;
use transvoxel::transition_sides::TransitionSide;

/// Signed distance to a sphere of radius 5 centred at the origin.
/// Positive = inside the sphere.
fn sphere(x: f32, y: f32, z: f32) -> f32 {
    5.0 - (x * x + y * y + z * z).sqrt()
}

/// A sphere with sinusoidal surface noise.
fn bumpy_sphere(x: f32, y: f32, z: f32) -> f32 {
    sphere(x, y, z) + 0.3 * (x * 3.0).sin() * (y * 3.0).cos() * (z * 3.0).sin()
}

fn stats(label: &str, mesh: &Mesh) {
    println!(
        "{:<42} {:>6} vertices  {:>6} triangles",
        label,
        mesh.vertex_count(),
        mesh.triangle_count()
    );
}

fn main() {
    println!("╔══════════════════════════════════════════════════╗");
    println!("║   Transvoxel Algorithm — Rust Demo               ║");
    println!("║   Reference: https://transvoxel.org/             ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();

    let block = Block::new([-6.0, -6.0, -6.0], 12.0, 16);
    let threshold = 0.0_f32;

    // 1. Pure Marching Cubes (no transitions)
    let m = extract_mesh(&sphere, &block, threshold, TransitionSides::empty());
    stats("Sphere — no transitions", &m);

    // 2. Single transition face
    let m = extract_mesh(
        &sphere,
        &block,
        threshold,
        TransitionSides::from(TransitionSide::LowX),
    );
    stats("Sphere — transition LowX", &m);

    // 3. Three transition faces
    let sides = TransitionSides::empty()
        | TransitionSide::LowX
        | TransitionSide::HighY
        | TransitionSide::LowZ;
    let m = extract_mesh(&sphere, &block, threshold, sides);
    stats("Sphere — transitions LowX+HighY+LowZ", &m);

    // 4. All six transition faces on a bumpy sphere
    let m = extract_mesh(&bumpy_sphere, &block, threshold, TransitionSides::all());
    stats("Bumpy sphere — all 6 transitions", &m);

    // 5. OBJ-style preview of first few vertices
    println!();
    println!("--- First 6 vertices (sphere, no transitions) ---");
    let m = extract_mesh(&sphere, &block, threshold, TransitionSides::empty());
    for i in 0..m.vertex_count().min(6) {
        println!(
            "v {:.4}  {:.4}  {:.4}",
            m.positions[i * 3],
            m.positions[i * 3 + 1],
            m.positions[i * 3 + 2]
        );
    }
    if m.indices.len() >= 3 {
        println!(
            "f {} {} {}",
            m.indices[0] + 1,
            m.indices[1] + 1,
            m.indices[2] + 1
        );
    }

    println!();
    println!("Done.");
}
