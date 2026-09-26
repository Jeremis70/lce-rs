//! `Pos`, `TilePos`, `ChunkPos` and the `ChunkPos` 64-bit hash.
//!
//! All coordinate arithmetic in this module is two's-complement and wraps on
//! overflow. That is part of the specified behaviour: the hashes in particular
//! are expected to overflow for large coordinates, and the same inputs must
//! always produce the same bits.

use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::mth;
use crate::vec3::Vec3;

/// An integer position, used for things like village centres and door
/// positions.
///
/// North is −Z, south is +Z, west is −X and east is +X.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Pos {
    pub const ZERO: Self = Self::new(0, 0, 0);

    #[must_use]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// The 32-bit hash `x + (z << 8) + (y << 16)`, wrapping.
    ///
    /// Positions that differ by 256 in `x` and −1 in `z` (and similar)
    /// collide; the hash is only meant to spread nearby positions.
    #[must_use]
    pub const fn hash_code(self) -> i32 {
        self.x.wrapping_add(self.z << 8).wrapping_add(self.y << 16)
    }

    /// Orders by `y`, then `z`, then `x`, returning the wrapping difference
    /// of the first coordinate that differs (or `0` if all are equal).
    ///
    /// Only the sign of the result is meaningful, and even that flips when
    /// the difference overflows (for example `i32::MIN` against `1`), so this
    /// is deliberately not exposed as [`Ord`].
    #[must_use]
    pub const fn compare_to(self, other: Self) -> i32 {
        if self.y != other.y {
            self.y.wrapping_sub(other.y)
        } else if self.z != other.z {
            self.z.wrapping_sub(other.z)
        } else {
            self.x.wrapping_sub(other.x)
        }
    }

    /// Returns this position translated by `(dx, dy, dz)`.
    #[must_use]
    pub const fn offset(self, dx: i32, dy: i32, dz: i32) -> Self {
        Self::new(
            self.x.wrapping_add(dx),
            self.y.wrapping_add(dy),
            self.z.wrapping_add(dz),
        )
    }

    /// One step up (+Y).
    #[must_use]
    pub const fn above(self) -> Self {
        self.above_n(1)
    }

    /// `steps` up (+Y).
    #[must_use]
    pub const fn above_n(self, steps: i32) -> Self {
        self.offset(0, steps, 0)
    }

    /// One step down (−Y).
    #[must_use]
    pub const fn below(self) -> Self {
        self.below_n(1)
    }

    /// `steps` down (−Y).
    #[must_use]
    pub const fn below_n(self, steps: i32) -> Self {
        Self::new(self.x, self.y.wrapping_sub(steps), self.z)
    }

    /// One step north (−Z).
    #[must_use]
    pub const fn north(self) -> Self {
        self.north_n(1)
    }

    /// `steps` north (−Z).
    #[must_use]
    pub const fn north_n(self, steps: i32) -> Self {
        Self::new(self.x, self.y, self.z.wrapping_sub(steps))
    }

    /// One step south (+Z).
    #[must_use]
    pub const fn south(self) -> Self {
        self.south_n(1)
    }

    /// `steps` south (+Z).
    #[must_use]
    pub const fn south_n(self, steps: i32) -> Self {
        self.offset(0, 0, steps)
    }

    /// One step west (−X).
    #[must_use]
    pub const fn west(self) -> Self {
        self.west_n(1)
    }

    /// `steps` west (−X).
    #[must_use]
    pub const fn west_n(self, steps: i32) -> Self {
        Self::new(self.x.wrapping_sub(steps), self.y, self.z)
    }

    /// One step east (+X).
    #[must_use]
    pub const fn east(self) -> Self {
        self.east_n(1)
    }

    /// `steps` east (+X).
    #[must_use]
    pub const fn east_n(self, steps: i32) -> Self {
        self.offset(steps, 0, 0)
    }

    /// Euclidean distance to `(x, y, z)`.
    ///
    /// Each coordinate difference is taken as a wrapping `i32`. The `x` term
    /// is then squared in `f64`, but the `y` and `z` terms are squared as
    /// wrapping `i32` products before being widened, so they overflow once a
    /// difference exceeds 46340 in magnitude. Distances between positions
    /// inside the world are unaffected.
    #[must_use]
    pub fn dist(self, x: i32, y: i32, z: i32) -> f64 {
        let dx = f64::from(self.x.wrapping_sub(x));
        let dy = self.y.wrapping_sub(y);
        let dz = self.z.wrapping_sub(z);
        (dx * dx + f64::from(dy.wrapping_mul(dy)) + f64::from(dz.wrapping_mul(dz))).sqrt()
    }

    /// [`Pos::dist`] to another position.
    #[must_use]
    pub fn dist_to(self, other: Self) -> f64 {
        self.dist(other.x, other.y, other.z)
    }

    /// Squared distance to `(x, y, z)`, computed entirely in wrapping `i32`
    /// arithmetic and then rounded to the nearest `f32`.
    #[must_use]
    pub fn dist_sqr(self, x: i32, y: i32, z: i32) -> f32 {
        let dx = self.x.wrapping_sub(x);
        let dy = self.y.wrapping_sub(y);
        let dz = self.z.wrapping_sub(z);
        let sum = dx
            .wrapping_mul(dx)
            .wrapping_add(dy.wrapping_mul(dy))
            .wrapping_add(dz.wrapping_mul(dz));
        // Sums above 2^24 are rounded to the nearest representable `f32`.
        sum as f32
    }
}

impl Add for Pos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self.offset(rhs.x, rhs.y, rhs.z)
    }
}

impl AddAssign for Pos {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Pos {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self::new(
            self.x.wrapping_sub(rhs.x),
            self.y.wrapping_sub(rhs.y),
            self.z.wrapping_sub(rhs.z),
        )
    }
}

impl SubAssign for Pos {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

/// The integer coordinates of a single tile.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TilePos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl TilePos {
    #[must_use]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// The tile containing the point `p`: each coordinate is floored with
    /// [`mth::floor`].
    #[must_use]
    pub const fn containing(p: Vec3) -> Self {
        Self::new(mth::floor(p.x), mth::floor(p.y), mth::floor(p.z))
    }

    /// The 32-bit hash `x * 8976890 + y * 981131 + z`, wrapping.
    #[must_use]
    pub const fn hash_code(self) -> i32 {
        self.x
            .wrapping_mul(8_976_890)
            .wrapping_add(self.y.wrapping_mul(981_131))
            .wrapping_add(self.z)
    }
}

impl From<Vec3> for TilePos {
    fn from(p: Vec3) -> Self {
        Self::containing(p)
    }
}

/// The coordinates of a 16 × 16 column of tiles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    #[must_use]
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Packs a chunk coordinate pair into one 64-bit key: the low 32 bits are
    /// the two's-complement bits of `x`, the high 32 bits those of `z`.
    ///
    /// The mapping is a bijection, so distinct pairs never collide. The key
    /// is negative exactly when `z` is negative, and `x` is never
    /// sign-extended into the high word.
    #[must_use]
    pub const fn long_hash(x: i32, z: i32) -> i64 {
        // Reinterpreting through `u32` zero-extends; the final cast
        // reinterprets the packed bits as signed.
        let lo = x as u32 as u64;
        let hi = (z as u32 as u64) << 32;
        (lo | hi) as i64
    }

    /// [`ChunkPos::long_hash`] of this position.
    #[must_use]
    pub const fn to_long(self) -> i64 {
        Self::long_hash(self.x, self.z)
    }

    /// The 32-bit hash: the low and high words of [`ChunkPos::to_long`]
    /// XORed together, which comes to `x ^ z`.
    ///
    /// Unlike the 64-bit key this collides, e.g. for every `(n, n)`.
    #[must_use]
    pub const fn hash_code(self) -> i32 {
        let key = self.to_long();
        (key as i32) ^ ((key >> 32) as i32)
    }

    /// Squared horizontal distance from the centre of this chunk
    /// (`x * 16 + 8`, `z * 16 + 8`, wrapping) to `(px, pz)`.
    #[must_use]
    pub fn distance_to_sqr(self, px: f64, pz: f64) -> f64 {
        let xd = f64::from(self.middle_block_x()) - px;
        let zd = f64::from(self.middle_block_z()) - pz;
        xd * xd + zd * zd
    }

    /// The X coordinate of the tile at the centre of this chunk,
    /// `(x << 4) + 8`, wrapping.
    #[must_use]
    pub const fn middle_block_x(self) -> i32 {
        (self.x << 4).wrapping_add(8)
    }

    /// The Z coordinate of the tile at the centre of this chunk,
    /// `(z << 4) + 8`, wrapping.
    #[must_use]
    pub const fn middle_block_z(self) -> i32 {
        (self.z << 4).wrapping_add(8)
    }

    /// The centre tile of this chunk at height `y`.
    #[must_use]
    pub const fn middle_block_position(self, y: i32) -> TilePos {
        TilePos::new(self.middle_block_x(), y, self.middle_block_z())
    }
}

/// Formats as `[x, z]`.
impl fmt::Display for ChunkPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.z)
    }
}
