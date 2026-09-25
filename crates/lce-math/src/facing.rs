//! `Facing` (down = 0 .. east = 5), opposite-face and per-axis step tables.

/// One of the six faces of a tile. The discriminants are the face ids stored
/// in hit results and save data, so they must not be reordered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Facing {
    /// −Y
    Down = 0,
    /// +Y
    Up = 1,
    /// −Z
    North = 2,
    /// +Z
    South = 3,
    /// −X
    West = 4,
    /// +X
    East = 5,
}
