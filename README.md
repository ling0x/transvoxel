# Transvoxel Algorithm — Rust Implementation

A Rust implementation of [Eric Lengyel's Transvoxel Algorithm](https://transvoxel.org/), which provides seamless stitching of voxel terrain meshes at differing resolutions (LOD — Level of Detail).

## Overview

When generating triangle meshes from voxel data using Marching Cubes at multiple resolutions, cracks form along the boundaries between meshes of different resolutions. The Transvoxel Algorithm solves this by inserting special **transition cells** at the boundary between high-resolution and low-resolution voxel data.

Instead of handling ~1.2 million combinations, the algorithm considers only 9 samples of high-resolution data → **512 cases** → **73 equivalence classes**.

## Features

- Full Marching Cubes regular cell extraction
- Transition cell extraction for all 6 sides (LowX, HighX, LowY, HighY, LowZ, HighZ)
- Smooth vertex normal interpolation via central-difference gradients
- Density-field based surface extraction
- Configurable threshold value
- Zero external dependencies

## Usage

```rust
use transvoxel::prelude::*;

// Define a density function (e.g., a sphere)
fn sphere(x: f32, y: f32, z: f32) -> f32 {
    5.0 - (x * x + y * y + z * z).sqrt()
}

fn main() {
    let block = Block::new([-6.0, -6.0, -6.0], 12.0, 16);
    let threshold = 0.0f32;
    let sides = TransitionSides::empty();

    let mesh = extract_mesh(&sphere, &block, threshold, sides);
    println!("Triangles: {}", mesh.triangle_count());
    println!("Vertices:  {}", mesh.vertex_count());
}
```

## Running the Demo

```bash
cargo run --bin demo
```

## Running Tests

```bash
cargo test
```

## Library Structure

| Module | Description |
|---|---|
| `tables` | Lookup tables from Eric Lengyel's data tables |
| `mesh` | Mesh data structures (positions, normals, indices) |
| `block` | Block/grid definition |
| `extraction` | Core Marching Cubes + Transvoxel extraction |
| `transition_sides` | Enum for the 6 transition sides |
| `prelude` | Convenient re-exports |

## Algorithm Reference

- [transvoxel.org](https://transvoxel.org/) — Eric Lengyel's official page
- Lengyel, Eric. *"Voxel-Based Terrain for Real-Time Virtual Simulations"*. PhD diss., University of California at Davis, 2010.
- Lengyel, Eric. *"Transition Cells for Dynamic Multiresolution Marching Cubes"*. Journal of Graphics, GPU, and Game Tools. Vol. 15, No. 2 (2010).

## License

MIT
