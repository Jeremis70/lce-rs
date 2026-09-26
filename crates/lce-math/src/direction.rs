//! `Direction` (south = 0 .. east = 3) and its rotation tables.

use crate::facing::Facing;

/// One of the four horizontal directions used to orient a tile or entity.
/// The discriminants are the orientation ids stored in save data, so they
/// must not be reordered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    /// +Z
    South = 0,
    /// −X
    West = 1,
    /// −Z
    North = 2,
    /// +X
    East = 3,
}

/// For each [`Direction`], the [`Facing`] a world-space face rotates to when
/// re-expressed relative to that orientation, indexed by the face's id.
const RELATIVE_FACING: [[Facing; 6]; 4] = [
    // South
    [
        Facing::Up,
        Facing::Down,
        Facing::South,
        Facing::North,
        Facing::East,
        Facing::West,
    ],
    // West
    [
        Facing::Up,
        Facing::Down,
        Facing::East,
        Facing::West,
        Facing::North,
        Facing::South,
    ],
    // North
    [
        Facing::Up,
        Facing::Down,
        Facing::North,
        Facing::South,
        Facing::West,
        Facing::East,
    ],
    // East
    [
        Facing::Up,
        Facing::Down,
        Facing::West,
        Facing::East,
        Facing::South,
        Facing::North,
    ],
];

impl Direction {
    /// All four directions, indexed by id (south, west, north, east).
    pub const ALL: [Self; 4] = [Self::South, Self::West, Self::North, Self::East];

    /// This direction's id, `0` (south) through `3` (east).
    #[must_use]
    pub const fn id(self) -> u8 {
        self as u8
    }

    /// The direction with the given id, or `None` if `id` is not `0..=3`.
    #[must_use]
    pub const fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::South),
            1 => Some(Self::West),
            2 => Some(Self::North),
            3 => Some(Self::East),
            _ => None,
        }
    }

    /// The unit step along `x` when moving this way: `-1` for west, `1` for
    /// east, `0` for north/south.
    #[must_use]
    pub const fn step_x(self) -> i32 {
        match self {
            Self::West => -1,
            Self::East => 1,
            Self::South | Self::North => 0,
        }
    }

    /// The unit step along `z` when moving this way: `-1` for north, `1` for
    /// south, `0` for west/east.
    #[must_use]
    pub const fn step_z(self) -> i32 {
        match self {
            Self::South => 1,
            Self::North => -1,
            Self::West | Self::East => 0,
        }
    }

    /// The tile face this direction points toward.
    #[must_use]
    pub const fn facing(self) -> Facing {
        match self {
            Self::South => Facing::South,
            Self::West => Facing::West,
            Self::North => Facing::North,
            Self::East => Facing::East,
        }
    }

    /// The direction a face points toward, or `None` for [`Facing::Down`] and
    /// [`Facing::Up`], which have no horizontal direction.
    #[must_use]
    pub const fn from_facing(facing: Facing) -> Option<Self> {
        match facing {
            Facing::Down | Facing::Up => None,
            Facing::North => Some(Self::North),
            Facing::South => Some(Self::South),
            Facing::West => Some(Self::West),
            Facing::East => Some(Self::East),
        }
    }

    /// The direction pointing the opposite way.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::South => Self::North,
            Self::West => Self::East,
            Self::North => Self::South,
            Self::East => Self::West,
        }
    }

    /// This direction rotated 90 degrees clockwise (viewed from above).
    #[must_use]
    pub const fn clockwise(self) -> Self {
        match self {
            Self::South => Self::West,
            Self::West => Self::North,
            Self::North => Self::East,
            Self::East => Self::South,
        }
    }

    /// This direction rotated 90 degrees counter-clockwise (viewed from
    /// above).
    #[must_use]
    pub const fn counter_clockwise(self) -> Self {
        match self {
            Self::South => Self::East,
            Self::West => Self::South,
            Self::North => Self::West,
            Self::East => Self::North,
        }
    }

    /// Re-expresses a world-space face relative to an object oriented this
    /// way, e.g. turning a stair's "world north" side into its "left" side
    /// once the stair itself faces west.
    #[must_use]
    pub const fn relative_facing(self, world_facing: Facing) -> Facing {
        RELATIVE_FACING[self as usize][world_facing as usize]
    }
}

impl TryFrom<u8> for Direction {
    /// The rejected id.
    type Error = u8;

    fn try_from(id: u8) -> Result<Self, u8> {
        Self::from_id(id).ok_or(id)
    }
}
