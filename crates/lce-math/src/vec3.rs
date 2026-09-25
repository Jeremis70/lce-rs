//! `Vec3`, a double-precision vector.

use std::fmt;
use std::ops::{Add, Mul, Sub};

/// Squared-length threshold below which a segment is treated as parallel to a
/// clipping plane.
///
/// The threshold is `1e-7` rounded to single precision and widened to `f64`,
/// so its value is `1.0000000116860974e-7` rather than exactly `1e-7`. This matters at the
/// boundary and must not be "cleaned up".
const CLIP_EPSILON: f64 = 1.0e-7_f32 as f64;

/// Length below which [`Vec3::normalize`] yields the zero vector.
const NORMALIZE_EPSILON: f64 = 0.0001;

/// A three-component double-precision vector.
///
/// This is a plain `Copy` value: every operation returns a new vector and
/// never aliases its inputs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    /// Creates a vector from raw components, preserving the sign of zero.
    ///
    /// This is the constructor used for every intermediate result.
    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Creates a vector with any negative-zero component replaced by
    /// positive zero.
    ///
    /// Only long-lived vectors (cached positions, stored offsets) are built
    /// this way; transient arithmetic results keep their signed zeros, see
    /// [`Vec3::new`].
    #[must_use]
    pub const fn canonical(x: f64, y: f64, z: f64) -> Self {
        const fn unsign_zero(v: f64) -> f64 {
            if v == 0.0 { 0.0 } else { v }
        }
        Self::new(unsign_zero(x), unsign_zero(y), unsign_zero(z))
    }

    #[must_use]
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[must_use]
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    #[must_use]
    pub fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Returns the unit vector in this direction, or [`Vec3::ZERO`] if the
    /// length is below `0.0001`.
    #[must_use]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len < NORMALIZE_EPSILON {
            return Self::ZERO;
        }
        Self::new(self.x / len, self.y / len, self.z / len)
    }

    #[must_use]
    pub fn distance(self, other: Self) -> f64 {
        self.distance_sqr(other).sqrt()
    }

    #[must_use]
    pub fn distance_sqr(self, other: Self) -> f64 {
        let d = other - self;
        d.x * d.x + d.y * d.y + d.z * d.z
    }

    /// Linear interpolation: `self + (to - self) * t`, evaluated per
    /// component in that order.
    #[must_use]
    pub fn lerp(self, to: Self, t: f64) -> Self {
        Self::new(
            self.x + (to.x - self.x) * t,
            self.y + (to.y - self.y) * t,
            self.z + (to.z - self.z) * t,
        )
    }

    /// Point where the segment `self → end` crosses the plane `x = plane`,
    /// or `None` if the segment is (nearly) parallel to it or does not
    /// reach it.
    #[must_use]
    pub fn clip_x(self, end: Self, plane: f64) -> Option<Self> {
        self.clip_axis(end, plane, |v| v.x)
    }

    /// Point where the segment `self → end` crosses the plane `y = plane`.
    /// See [`Vec3::clip_x`].
    #[must_use]
    pub fn clip_y(self, end: Self, plane: f64) -> Option<Self> {
        self.clip_axis(end, plane, |v| v.y)
    }

    /// Point where the segment `self → end` crosses the plane `z = plane`.
    /// See [`Vec3::clip_x`].
    #[must_use]
    pub fn clip_z(self, end: Self, plane: f64) -> Option<Self> {
        self.clip_axis(end, plane, |v| v.z)
    }

    fn clip_axis(self, end: Self, plane: f64, axis: impl Fn(Self) -> f64) -> Option<Self> {
        let delta = end - self;
        let axis_delta = axis(delta);
        if axis_delta * axis_delta < CLIP_EPSILON {
            return None;
        }

        let t = (plane - axis(self)) / axis_delta;
        // Written as two comparisons so that a NaN `t` is *accepted*; a range
        // `contains` would reject it.
        if t < 0.0 || t > 1.0 {
            return None;
        }
        Some(Self::new(
            self.x + delta.x * t,
            self.y + delta.y * t,
            self.z + delta.z * t,
        ))
    }

    /// Rotates about the X axis by `angle` radians.
    ///
    /// The sine and cosine are evaluated in single precision, since the angle
    /// is an `f32`, then widened. They come from the platform libm, not the
    /// [`crate::mth`] lookup table, so results are only reproducible to within libm
    /// accuracy. Only rendering code uses these.
    #[must_use]
    pub fn rotate_x(self, angle: f32) -> Self {
        let (sin, cos) = sin_cos(angle);
        Self::new(
            self.x,
            self.y * cos + self.z * sin,
            self.z * cos - self.y * sin,
        )
    }

    /// Rotates about the Y axis by `angle` radians. See [`Vec3::rotate_x`].
    #[must_use]
    pub fn rotate_y(self, angle: f32) -> Self {
        let (sin, cos) = sin_cos(angle);
        Self::new(
            self.x * cos + self.z * sin,
            self.y,
            self.z * cos - self.x * sin,
        )
    }

    /// Rotates about the Z axis by `angle` radians. See [`Vec3::rotate_x`].
    #[must_use]
    pub fn rotate_z(self, angle: f32) -> Self {
        let (sin, cos) = sin_cos(angle);
        Self::new(
            self.x * cos + self.y * sin,
            self.y * cos - self.x * sin,
            self.z,
        )
    }
}

fn sin_cos(angle: f32) -> (f64, f64) {
    (f64::from(angle.sin()), f64::from(angle.cos()))
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.6},{:.6},{:.6})", self.x, self.y, self.z)
    }
}

impl From<[f64; 3]> for Vec3 {
    fn from([x, y, z]: [f64; 3]) -> Self {
        Self::new(x, y, z)
    }
}

impl From<Vec3> for [f64; 3] {
    fn from(v: Vec3) -> Self {
        [v.x, v.y, v.z]
    }
}

/// For handing positions to rendering code. Simulation logic stays on [`Vec3`].
impl From<Vec3> for glam::DVec3 {
    fn from(v: Vec3) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}
