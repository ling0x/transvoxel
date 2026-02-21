//! # Transvoxel Algorithm — Rust Implementation
//!
//! A Rust implementation of Eric Lengyel's Transvoxel Algorithm.
//! See <https://transvoxel.org/> for the full description.
//!
//! The algorithm generates seamless triangle meshes from voxel data at
//! multiple resolutions (LOD), eliminating cracks at resolution boundaries
//! by inserting "transition cells" on boundary faces.
//!
//! ## Quick Start
//!
//! ```rust
//! use transvoxel::prelude::*;
//!
//! fn sphere(x: f32, y: f32, z: f32) -> f32 {
//!     5.0 - (x * x + y * y + z * z).sqrt()
//! }
//!
//! let block = Block::new([-6.0, -6.0, -6.0], 12.0, 10);
//! let mesh = extract_mesh(&sphere, &block, 0.0, TransitionSides::empty());
//! println!("Triangles: {}", mesh.triangle_count());
//! ```

pub mod tables;
pub mod mesh;
pub mod block;
pub mod transition_sides;
pub mod extraction;
pub mod prelude;
