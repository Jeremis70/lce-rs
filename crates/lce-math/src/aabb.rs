//! `Aabb` and per-axis collision clipping.

use std::fmt;

use crate::facing::Facing;
use crate::vec3::Vec3;

/// An axis-aligned bounding box spanning `(x0, y0, z0)` to `(x1, y1, z1)`.
///
/// This is a plain `Copy` value: every operation returns a new box and never
/// aliases its inputs. Nothing enforces `x0 <= x1` (and so on); a box built
/// inside out simply intersects nothing.
///
/// The comparisons below are deliberately not simplified: rewriting
/// `!(a <= b)` as `a > b` changes the result when a coordinate is NaN, and the
/// NaN results are part of the specified behaviour.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Aabb {
    pub x0: f64,
    pub y0: f64,
    pub z0: f64,
    pub x1: f64,
    pub y1: f64,
    pub z1: f64,
}

/// Where a segment enters an [`Aabb`], as found by [`Aabb::clip`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClipHit {
    /// The entry point, on the surface of the box.
    pub pos: Vec3,
    /// The face the entry point lies on.
    pub face: Facing,
}

impl Aabb {
    #[must_use]
    pub const fn new(x0: f64, y0: f64, z0: f64, x1: f64, y1: f64, z1: f64) -> Self {
        Self {
            x0,
            y0,
            z0,
            x1,
            y1,
            z1,
        }
    }

    /// Stretches the box along the motion `(xa, ya, za)`: a negative
    /// component extends the low face, a positive one the high face.
    ///
    /// This is the swept volume used to gather collision candidates.
    #[must_use]
    pub fn expand(self, xa: f64, ya: f64, za: f64) -> Self {
        let (x0, x1) = sweep(self.x0, self.x1, xa);
        let (y0, y1) = sweep(self.y0, self.y1, ya);
        let (z0, z1) = sweep(self.z0, self.z1, za);
        Self::new(x0, y0, z0, x1, y1, z1)
    }

    /// Pushes every face outwards by the given amount on its axis.
    #[must_use]
    pub fn grow(self, xa: f64, ya: f64, za: f64) -> Self {
        Self::new(
            self.x0 - xa,
            self.y0 - ya,
            self.z0 - za,
            self.x1 + xa,
            self.y1 + ya,
            self.z1 + za,
        )
    }

    /// Pulls every face inwards by the given amount on its axis.
    #[must_use]
    pub fn shrink(self, xa: f64, ya: f64, za: f64) -> Self {
        Self::new(
            self.x0 + xa,
            self.y0 + ya,
            self.z0 + za,
            self.x1 - xa,
            self.y1 - ya,
            self.z1 - za,
        )
    }

    /// Translates the box by `(xa, ya, za)`.
    #[must_use]
    pub fn moved(self, xa: f64, ya: f64, za: f64) -> Self {
        Self::new(
            self.x0 + xa,
            self.y0 + ya,
            self.z0 + za,
            self.x1 + xa,
            self.y1 + ya,
            self.z1 + za,
        )
    }

    /// Limits the X motion `xa` of `mover` so that it stops flush against
    /// `self`, treating `self` as a fixed obstacle.
    ///
    /// The motion is returned unchanged unless the two boxes overlap on both
    /// Y and Z (touching faces do not count) and `mover` starts entirely on
    /// the side it is moving away from. Callers apply this per obstacle, and
    /// the order they clip the axes in is observable.
    #[must_use]
    pub fn clip_x_collide(self, mover: Self, xa: f64) -> f64 {
        if !overlaps(self.y0, self.y1, mover.y0, mover.y1)
            || !overlaps(self.z0, self.z1, mover.z0, mover.z1)
        {
            return xa;
        }
        clip_collide(self.x0, self.x1, mover.x0, mover.x1, xa)
    }

    /// Limits the Y motion `ya` of `mover`. See [`Aabb::clip_x_collide`].
    #[must_use]
    pub fn clip_y_collide(self, mover: Self, ya: f64) -> f64 {
        if !overlaps(self.x0, self.x1, mover.x0, mover.x1)
            || !overlaps(self.z0, self.z1, mover.z0, mover.z1)
        {
            return ya;
        }
        clip_collide(self.y0, self.y1, mover.y0, mover.y1, ya)
    }

    /// Limits the Z motion `za` of `mover`. See [`Aabb::clip_x_collide`].
    #[must_use]
    pub fn clip_z_collide(self, mover: Self, za: f64) -> f64 {
        if !overlaps(self.x0, self.x1, mover.x0, mover.x1)
            || !overlaps(self.y0, self.y1, mover.y0, mover.y1)
        {
            return za;
        }
        clip_collide(self.z0, self.z1, mover.z0, mover.z1, za)
    }

    /// Whether the boxes share interior volume. Boxes that only touch along a
    /// face, edge or corner do not intersect.
    #[must_use]
    pub fn intersects(self, other: Self) -> bool {
        overlaps(self.x0, self.x1, other.x0, other.x1)
            && overlaps(self.y0, self.y1, other.y0, other.y1)
            && overlaps(self.z0, self.z1, other.z0, other.z1)
    }

    /// Like [`Aabb::intersects`], but boxes that merely touch also count.
    #[must_use]
    pub fn intersects_inner(self, other: Self) -> bool {
        touches(self.x0, self.x1, other.x0, other.x1)
            && touches(self.y0, self.y1, other.y0, other.y1)
            && touches(self.z0, self.z1, other.z0, other.z1)
    }

    /// Whether `p` lies strictly inside the box; points on any face are
    /// outside.
    #[must_use]
    pub fn contains(self, p: Vec3) -> bool {
        !(p.x <= self.x0 || p.x >= self.x1)
            && !(p.y <= self.y0 || p.y >= self.y1)
            && !(p.z <= self.z0 || p.z >= self.z1)
    }

    /// Whether `p` lies in the half-open box `[x0, x1) × [y0, y1) × [z0, z1)`:
    /// points on the low faces are inside, points on the high faces are not.
    #[must_use]
    pub fn contains_including_lower_bound(self, p: Vec3) -> bool {
        !(p.x < self.x0 || p.x >= self.x1)
            && !(p.y < self.y0 || p.y >= self.y1)
            && !(p.z < self.z0 || p.z >= self.z1)
    }

    /// Euclidean distance from `p` to the nearest point of the box, or `0.0`
    /// if `p` is inside it or on its surface.
    ///
    /// On an inside-out axis (`x0 > x1`) the low-side gap wins.
    #[must_use]
    pub fn distance(self, p: Vec3) -> f64 {
        let xd = gap(p.x, self.x0, self.x1);
        let yd = gap(p.y, self.y0, self.y1);
        let zd = gap(p.z, self.z0, self.z1);
        (xd * xd + yd * yd + zd * zd).sqrt()
    }

    /// The mean edge length.
    #[must_use]
    pub fn size(self) -> f64 {
        let xs = self.x1 - self.x0;
        let ys = self.y1 - self.y0;
        let zs = self.z1 - self.z0;
        (xs + ys + zs) / 3.0
    }

    /// Finds where the segment `from → to` first meets the surface of the
    /// box, or `None` if it misses.
    ///
    /// Each of the six face planes is intersected with the segment, and hits
    /// outside the face's rectangle (edges included) are discarded. The hit
    /// nearest `from` wins; on an exact tie the earlier face in the order
    /// west, east, down, up, north, south is kept.
    ///
    /// A segment that starts inside the box reports where it *leaves*, since
    /// only surface crossings are considered.
    #[must_use]
    pub fn clip(self, from: Vec3, to: Vec3) -> Option<ClipHit> {
        let candidates = [
            (
                from.clip_x(to, self.x0).filter(|&v| self.spans_yz(v)),
                Facing::West,
            ),
            (
                from.clip_x(to, self.x1).filter(|&v| self.spans_yz(v)),
                Facing::East,
            ),
            (
                from.clip_y(to, self.y0).filter(|&v| self.spans_xz(v)),
                Facing::Down,
            ),
            (
                from.clip_y(to, self.y1).filter(|&v| self.spans_xz(v)),
                Facing::Up,
            ),
            (
                from.clip_z(to, self.z0).filter(|&v| self.spans_xy(v)),
                Facing::North,
            ),
            (
                from.clip_z(to, self.z1).filter(|&v| self.spans_xy(v)),
                Facing::South,
            ),
        ];

        candidates
            .into_iter()
            .filter_map(|(pos, face)| pos.map(|pos| ClipHit { pos, face }))
            // Replace only on strictly closer, so ties (and NaN distances)
            // keep the earlier candidate.
            .reduce(|best, hit| {
                if from.distance_sqr(hit.pos) < from.distance_sqr(best.pos) {
                    hit
                } else {
                    best
                }
            })
    }

    /// Whether `v` lies within the box's closed Y and Z extents, i.e. on an
    /// X face's rectangle when it is on that face's plane.
    fn spans_yz(self, v: Vec3) -> bool {
        v.y >= self.y0 && v.y <= self.y1 && v.z >= self.z0 && v.z <= self.z1
    }

    /// Whether `v` lies within the box's closed X and Z extents.
    fn spans_xz(self, v: Vec3) -> bool {
        v.x >= self.x0 && v.x <= self.x1 && v.z >= self.z0 && v.z <= self.z1
    }

    /// Whether `v` lies within the box's closed X and Y extents.
    fn spans_xy(self, v: Vec3) -> bool {
        v.x >= self.x0 && v.x <= self.x1 && v.y >= self.y0 && v.y <= self.y1
    }
}

/// Extends `[lo, hi]` towards the sign of `delta`. Zero and NaN leave both
/// ends untouched.
fn sweep(lo: f64, hi: f64, delta: f64) -> (f64, f64) {
    if delta < 0.0 {
        (lo + delta, hi)
    } else if delta > 0.0 {
        (lo, hi + delta)
    } else {
        (lo, hi)
    }
}

/// How far `v` lies outside `[lo, hi]`; zero inside, and for NaN.
fn gap(v: f64, lo: f64, hi: f64) -> f64 {
    if v < lo {
        lo - v
    } else if v > hi {
        v - hi
    } else {
        0.0
    }
}

/// Whether `[a0, a1]` and `[b0, b1]` overlap with non-zero length.
fn overlaps(a0: f64, a1: f64, b0: f64, b1: f64) -> bool {
    !(b1 <= a0 || b0 >= a1)
}

/// Whether `[a0, a1]` and `[b0, b1]` overlap or share an endpoint.
fn touches(a0: f64, a1: f64, b0: f64, b1: f64) -> bool {
    !(b1 < a0 || b0 > a1)
}

/// Clamps the motion `delta` of the span `[mover0, mover1]` so that it stops
/// at the fixed span `[fixed0, fixed1]`, if it starts clear of it on the side
/// it is moving from.
///
/// A NaN `delta` is returned unchanged, since it enters neither branch. A NaN
/// gap (from `inf - inf`) also leaves `delta` unchanged, which is what
/// `f64::min`/`max` do with a NaN operand. The branches are exclusive because
/// the gap in the first is never negative.
fn clip_collide(fixed0: f64, fixed1: f64, mover0: f64, mover1: f64, delta: f64) -> f64 {
    if delta > 0.0 && mover1 <= fixed0 {
        delta.min(fixed0 - mover1)
    } else if delta < 0.0 && mover0 >= fixed1 {
        delta.max(fixed1 - mover0)
    } else {
        delta
    }
}

/// `box[x0, y0, z0 -> x1, y1, z1]`, each coordinate in `%g` notation (six
/// significant digits, trailing zeros stripped).
impl fmt::Display for Aabb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("box[")?;
        write_g(f, self.x0)?;
        f.write_str(", ")?;
        write_g(f, self.y0)?;
        f.write_str(", ")?;
        write_g(f, self.z0)?;
        f.write_str(" -> ")?;
        write_g(f, self.x1)?;
        f.write_str(", ")?;
        write_g(f, self.y1)?;
        f.write_str(", ")?;
        write_g(f, self.z1)?;
        f.write_str("]")
    }
}

/// Significant digits in `%g` output at default precision.
const G_PRECISION: i32 = 6;

/// Writes `v` as `printf("%g", v)` does.
///
/// `%g` picks fixed or scientific notation from the decimal exponent of the
/// value *after* rounding to six significant digits, then strips trailing
/// zeros. Rounding once in scientific notation gives that exponent; the fixed
/// form is then rounded at the same digit, so both agree.
fn write_g(f: &mut fmt::Formatter<'_>, v: f64) -> fmt::Result {
    if !v.is_finite() {
        let sign = if v.is_sign_negative() { "-" } else { "" };
        let body = if v.is_nan() { "nan" } else { "inf" };
        return write!(f, "{sign}{body}");
    }

    let sci = format!("{:.*e}", (G_PRECISION - 1) as usize, v);
    let (mantissa, exp) = sci.split_once('e').ok_or(fmt::Error)?;
    let exp: i32 = exp.parse().map_err(|_| fmt::Error)?;

    if (-4..G_PRECISION).contains(&exp) {
        let decimals = (G_PRECISION - 1 - exp) as usize;
        f.write_str(trim_fraction(&format!("{v:.decimals$}")))
    } else {
        let sign = if exp < 0 { '-' } else { '+' };
        write!(
            f,
            "{}e{sign}{:02}",
            trim_fraction(mantissa),
            exp.unsigned_abs()
        )
    }
}

/// Drops trailing fractional zeros, and the point if nothing follows it.
fn trim_fraction(s: &str) -> &str {
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.')
    } else {
        s
    }
}
