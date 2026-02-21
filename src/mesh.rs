//! Mesh output data structure.
//!
//! Vertex data is stored as flat `Vec<f32>` arrays (3 components per element),
//! and triangles are stored as a `Vec<u32>` index list (3 indices per triangle).

/// A triangle mesh produced by the Transvoxel / Marching Cubes extraction.
#[derive(Debug, Clone, Default)]
pub struct Mesh {
    /// Flat vertex positions: `[x0, y0, z0,  x1, y1, z1, …]`.
    pub positions: Vec<f32>,
    /// Flat vertex normals (unit vectors): `[nx0, ny0, nz0, …]`.
    pub normals: Vec<f32>,
    /// Triangle index list: every three entries form one triangle.
    pub indices: Vec<u32>,
}

impl Mesh {
    /// Creates an empty mesh.
    pub fn new() -> Self { Mesh::default() }

    /// Returns the number of vertices.
    #[inline]
    pub fn vertex_count(&self) -> usize { self.positions.len() / 3 }

    /// Returns the number of triangles.
    #[inline]
    pub fn triangle_count(&self) -> usize { self.indices.len() / 3 }

    /// Add a vertex and return its index.
    pub(crate) fn push_vertex(&mut self, pos: [f32; 3], normal: [f32; 3]) -> u32 {
        let idx = self.vertex_count() as u32;
        self.positions.extend_from_slice(&pos);
        self.normals.extend_from_slice(&normal);
        idx
    }

    /// Append a triangle by vertex indices.
    pub(crate) fn push_triangle(&mut self, a: u32, b: u32, c: u32) {
        self.indices.push(a);
        self.indices.push(b);
        self.indices.push(c);
    }
}
