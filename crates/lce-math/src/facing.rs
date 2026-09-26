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

impl Facing {
    /// All six faces, indexed by id (down, up, north, south, west, east).
    pub const ALL: [Self; 6] = [
        Self::Down,
        Self::Up,
        Self::North,
        Self::South,
        Self::West,
        Self::East,
    ];

    /// This face's id, `0` (down) through `5` (east).
    #[must_use]
    pub const fn id(self) -> u8 {
        self as u8
    }

    /// The face with the given id, or `None` if `id` is not `0..=5`.
    #[must_use]
    pub const fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::Down),
            1 => Some(Self::Up),
            2 => Some(Self::North),
            3 => Some(Self::South),
            4 => Some(Self::West),
            5 => Some(Self::East),
            _ => None,
        }
    }

    /// The face on the other side of a tile from this one: down pairs with
    /// up, north with south, and west with east.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }

    /// The unit step along `x` when crossing this face: `-1` for west, `1`
    /// for east, `0` for every other face.
    #[must_use]
    pub const fn step_x(self) -> i32 {
        match self {
            Self::West => -1,
            Self::East => 1,
            Self::Down | Self::Up | Self::North | Self::South => 0,
        }
    }

    /// The unit step along `y` when crossing this face: `-1` for down, `1`
    /// for up, `0` for every other face.
    #[must_use]
    pub const fn step_y(self) -> i32 {
        match self {
            Self::Down => -1,
            Self::Up => 1,
            Self::North | Self::South | Self::West | Self::East => 0,
        }
    }

    /// The unit step along `z` when crossing this face: `-1` for north, `1`
    /// for south, `0` for every other face.
    #[must_use]
    pub const fn step_z(self) -> i32 {
        match self {
            Self::North => -1,
            Self::South => 1,
            Self::Down | Self::Up | Self::West | Self::East => 0,
        }
    }
}

impl TryFrom<u8> for Facing {
    /// The rejected id.
    type Error = u8;

    fn try_from(id: u8) -> Result<Self, u8> {
        Self::from_id(id).ok_or(id)
    }
}
