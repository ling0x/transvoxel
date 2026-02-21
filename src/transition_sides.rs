//! Defines the 6 sides of a block that can be transition (double-resolution) faces.
//!
//! In the Transvoxel Algorithm, a "transition face" is a face of a block that
//! borders a neighbouring block rendered at higher resolution. The algorithm
//! inserts special transition cells on that face to seamlessly stitch the two
//! meshes together without cracks.

/// The 6 possible sides of a voxel block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransitionSide {
    /// Face at minimum X
    LowX = 0,
    /// Face at maximum X
    HighX = 1,
    /// Face at minimum Y
    LowY = 2,
    /// Face at maximum Y
    HighY = 3,
    /// Face at minimum Z
    LowZ = 4,
    /// Face at maximum Z
    HighZ = 5,
}

impl TransitionSide {
    /// All 6 sides in a fixed order.
    pub const ALL: [TransitionSide; 6] = [
        TransitionSide::LowX,
        TransitionSide::HighX,
        TransitionSide::LowY,
        TransitionSide::HighY,
        TransitionSide::LowZ,
        TransitionSide::HighZ,
    ];

    /// Returns the (axis_index, sign) for the outward normal of this side.
    /// axis: 0=X, 1=Y, 2=Z; sign: -1.0 (low) or +1.0 (high).
    pub fn normal_axis_sign(&self) -> (usize, f32) {
        match self {
            TransitionSide::LowX => (0, -1.0),
            TransitionSide::HighX => (0, 1.0),
            TransitionSide::LowY => (1, -1.0),
            TransitionSide::HighY => (1, 1.0),
            TransitionSide::LowZ => (2, -1.0),
            TransitionSide::HighZ => (2, 1.0),
        }
    }

    /// The two tangent axes (for 2-D indexing within the face).
    /// Returns (u_axis, v_axis) as indices into [x, y, z].
    pub fn face_axes(&self) -> (usize, usize) {
        match self {
            TransitionSide::LowX | TransitionSide::HighX => (1, 2),
            TransitionSide::LowY | TransitionSide::HighY => (0, 2),
            TransitionSide::LowZ | TransitionSide::HighZ => (0, 1),
        }
    }
}

/// A bitflag set of [`TransitionSide`] values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TransitionSides(pub u8);

impl TransitionSides {
    /// No transition sides active.
    pub fn empty() -> Self {
        TransitionSides(0)
    }

    /// All 6 sides active.
    pub fn all() -> Self {
        TransitionSides(0b0011_1111)
    }

    /// Returns `true` if the given side is active.
    pub fn contains(&self, side: TransitionSide) -> bool {
        self.0 & (1 << side as u8) != 0
    }

    /// Add a side to the set.
    pub fn insert(&mut self, side: TransitionSide) {
        self.0 |= 1 << side as u8;
    }

    /// Iterate over all active sides.
    pub fn iter(&self) -> impl Iterator<Item = TransitionSide> + '_ {
        TransitionSide::ALL
            .iter()
            .copied()
            .filter(move |s| self.contains(*s))
    }
}

impl From<TransitionSide> for TransitionSides {
    fn from(side: TransitionSide) -> Self {
        let mut s = TransitionSides::empty();
        s.insert(side);
        s
    }
}

impl std::ops::BitOrAssign<TransitionSide> for TransitionSides {
    fn bitor_assign(&mut self, rhs: TransitionSide) {
        self.insert(rhs);
    }
}

impl std::ops::BitOr<TransitionSide> for TransitionSides {
    type Output = TransitionSides;
    fn bitor(mut self, rhs: TransitionSide) -> TransitionSides {
        self.insert(rhs);
        self
    }
}
