//! `Mth`: the 65536-entry sine lookup table, floor/clamp/angle-wrapping helpers,
//! and a ranged random integer.
//!
//! # Float-to-integer conversion
//!
//! Every function here that turns a float into an integer first truncates it
//! towards zero. When the truncated value does not fit the target type (NaN,
//! either infinity, or any finite value outside the range), the conversion
//! yields the target's minimum, `i32::MIN` or `i64::MIN`. Everything after
//! that step is ordinary wrapping integer arithmetic, so for example
//! `floor(-1e10)` is `i32::MAX`: the conversion gives `i32::MIN`, and stepping
//! down by one wraps. These results are part of the specified behaviour and
//! are covered by the bit-exact test vectors.

use crate::random::JavaRandom;

/// π as used throughout the simulation: `3.141592654` rounded to single
/// precision (bits `0x4049_0fdb`).
///
/// Wherever it meets `f64` arithmetic it widens to `3.1415927410125732`, not
/// to [`std::f64::consts::PI`].
#[expect(
    clippy::approx_constant,
    clippy::excessive_precision,
    reason = "the specified literal, not a rounded form of π"
)]
pub const PI: f32 = 3.141_592_654;

/// Degrees-to-radians factor, `PI / 180` in single precision.
pub const DEG_TO_RAD: f32 = PI / 180.0;

/// Radians-to-degrees factor, `180 / PI` in single precision.
pub const RAD_TO_DEG: f32 = 180.0 / PI;

/// Number of entries in the sine table; one full turn.
const SIN_TABLE_LEN: usize = 65536;

/// Table entries per radian, `65536 / (2 * PI)` in single precision.
const SIN_SCALE: f32 = 65536.0 / (PI * 2.0);

/// `SIN_TABLE[i]` is the sine of the single-precision angle
/// `i * PI * 2 / 65536` (evaluated left to right in `f32`), computed in
/// double precision and rounded to `f32`.
///
/// Built at compile time from IEEE-754 basic operations only, so the table is
/// identical on every target regardless of the platform's libm.
static SIN_TABLE: [f32; SIN_TABLE_LEN] = build_sin_table();

const fn build_sin_table() -> [f32; SIN_TABLE_LEN] {
    let mut table = [0.0; SIN_TABLE_LEN];
    let mut i = 0;
    while i < SIN_TABLE_LEN {
        let angle = i as f32 * PI * 2.0 / 65536.0;
        table[i] = sin_f64(angle as f64) as f32;
        i += 1;
    }
    table
}

/// Sine of `x` in double precision, for `0 <= x < 7`.
///
/// `x` is reduced to `r = x - k·π/2` with `|r| <= π/4`, using π/2 split
/// into three parts (Cody–Waite). The first two parts have their low bits
/// cleared, so `k` times either part is exact for the small `k` needed here
/// and the reduction loses nothing even when `x` lies near a multiple of π.
/// The quadrant `k mod 4` then selects `±sin r` or `±cos r`, each a Taylor
/// polynomial truncated well below one `f64` ulp on `|r| <= π/4`.
///
/// The result is within a couple of `f64` ulps of the true sine, far inside
/// the `2⁻²⁴` relative spacing that the final rounding to `f32` resolves.
const fn sin_f64(x: f64) -> f64 {
    /// First 33 bits of π/2.
    const PIO2_1: f64 = f64::from_bits(0x3ff9_21fb_5440_0000);
    /// Next 33 bits of π/2.
    const PIO2_2: f64 = f64::from_bits(0x3dd0_b461_1a60_0000);
    /// π/2 − `PIO2_1` − `PIO2_2`, rounded.
    const PIO2_3: f64 = f64::from_bits(0x3ba3_198a_2e03_7073);

    let k = (x * std::f64::consts::FRAC_2_PI + 0.5) as i32;
    let kf = k as f64;
    let r = ((x - kf * PIO2_1) - kf * PIO2_2) - kf * PIO2_3;
    match k & 3 {
        0 => sin_poly(r),
        1 => cos_poly(r),
        2 => -sin_poly(r),
        _ => -cos_poly(r),
    }
}

/// Taylor series of `sin r` through the `r¹⁷` term; the next term is below
/// `1e-19` on `|r| <= π/4`.
const fn sin_poly(r: f64) -> f64 {
    let s = r * r;
    // p(s) = 1/3! - s/5! + s²/7! - … - s⁷/17!, so sin r = r - r·s·p(s).
    let p = -1.0 / 355_687_428_096_000.0;
    let p = p * s + 1.0 / 1_307_674_368_000.0;
    let p = p * s - 1.0 / 6_227_020_800.0;
    let p = p * s + 1.0 / 39_916_800.0;
    let p = p * s - 1.0 / 362_880.0;
    let p = p * s + 1.0 / 5_040.0;
    let p = p * s - 1.0 / 120.0;
    let p = p * s + 1.0 / 6.0;
    r - r * s * p
}

/// Taylor series of `cos r` through the `r¹⁸` term; the next term is below
/// `1e-20` on `|r| <= π/4`.
const fn cos_poly(r: f64) -> f64 {
    let s = r * r;
    // q(s) = 1/4! - s/6! + s²/8! - … - s⁷/18!, so cos r = 1 - s/2 + s²·q(s).
    let q = -1.0 / 6_402_373_705_728_000.0;
    let q = q * s + 1.0 / 20_922_789_888_000.0;
    let q = q * s - 1.0 / 87_178_291_200.0;
    let q = q * s + 1.0 / 479_001_600.0;
    let q = q * s - 1.0 / 3_628_800.0;
    let q = q * s + 1.0 / 40_320.0;
    let q = q * s - 1.0 / 720.0;
    let q = q * s + 1.0 / 24.0;
    1.0 - 0.5 * s + s * s * q
}

/// Truncates towards zero; `i32::MIN` when the result does not fit.
const fn trunc_i32(v: f64) -> i32 {
    // Written as two comparisons so that NaN falls through to the fallback.
    if v > i32::MIN as f64 - 1.0 && v < -(i32::MIN as f64) {
        v as i32
    } else {
        i32::MIN
    }
}

/// Truncates towards zero; `i64::MIN` when the result does not fit.
const fn trunc_i64(v: f64) -> i64 {
    // No `f64` lies strictly between -2⁶³ - 1 and -2⁶³, so `>=` suffices.
    if v >= i64::MIN as f64 && v < -(i64::MIN as f64) {
        v as i64
    } else {
        i64::MIN
    }
}

/// Table index for a scaled angle.
const fn sin_index(scaled: f32) -> usize {
    (trunc_i32(scaled as f64) & 0xffff) as usize
}

/// The whole sine table: entry `i` is the sine of the single-precision angle
/// `i * PI * 2 / 65536`, rounded to `f32`.
#[must_use]
pub const fn sin_table() -> &'static [f32; SIN_TABLE_LEN] {
    &SIN_TABLE
}

/// Table sine of `angle` radians.
///
/// The angle is scaled by `65536 / (2 * PI)` in single precision, truncated
/// towards zero and masked to 16 bits, so the result is the table entry at or
/// below the angle (above it, for negative angles). NaN, infinite and very
/// large angles (beyond about ±205887 radians) convert to `i32::MIN`, which
/// masks to entry `0`, i.e. `0.0`.
#[must_use]
pub const fn sin(angle: f32) -> f32 {
    SIN_TABLE[sin_index(angle * SIN_SCALE)]
}

/// Table cosine of `angle` radians: the sine table read a quarter turn
/// (16384 entries) ahead.
///
/// The quarter turn is added after scaling, in single precision, before
/// truncating; out-of-range results read entry `0` exactly as in [`sin`].
#[must_use]
pub const fn cos(angle: f32) -> f32 {
    SIN_TABLE[sin_index(angle * SIN_SCALE + 16384.0)]
}

/// The largest integer not greater than `v`.
///
/// Computed by truncating towards zero and stepping down by one when the
/// truncation rounded a negative value up. Exact wherever the floor fits in
/// an `i32`. Otherwise, per the conversion rule in the module docs: NaN and
/// values of at least `2³¹` give `i32::MIN`, and values below `-2³¹` give
/// `i32::MAX`.
#[must_use]
pub const fn floor(v: f64) -> i32 {
    let i = trunc_i32(v);
    if v < i as f64 { i.wrapping_sub(1) } else { i }
}

/// [`floor`] for a single-precision value; the truncated integer is compared
/// against `v` in single precision.
///
/// Out-of-range results follow the same rule: NaN and values of at least
/// `2³¹` give `i32::MIN`, values below `-2³¹` give `i32::MAX`.
#[must_use]
pub const fn floor_f32(v: f32) -> i32 {
    let i = trunc_i32(v as f64);
    if v < i as f32 { i.wrapping_sub(1) } else { i }
}

/// [`floor`] into an `i64`.
///
/// Exact wherever the floor fits in an `i64`. NaN and values of at least
/// `2⁶³` give `i64::MIN`; values below `-2⁶³` give `i64::MAX`.
#[must_use]
pub const fn lfloor(v: f64) -> i64 {
    let i = trunc_i64(v);
    if v < i as f64 { i.wrapping_sub(1) } else { i }
}

/// A floor that shifts `x` up by `1024`, truncates, and shifts back.
///
/// Only floors correctly for `x >= -1024`. Below that the shifted value is
/// negative and truncation rounds it up, so non-integers come out one too
/// high (`fast_floor(-1025.5)` is `-1025`). If the shifted value
/// does not fit in an `i32` the conversion gives `i32::MIN` and the shift
/// back wraps, so NaN and both infinities give `2147482624`.
#[must_use]
pub const fn fast_floor(x: f64) -> i32 {
    trunc_i32(x + 1024.0).wrapping_sub(1024)
}

/// The smallest integer not less than `v`, compared in single precision.
///
/// Exact wherever the ceiling fits in an `i32`. NaN and values below `-2³¹`
/// give `i32::MIN`; values of at least `2³¹` give `i32::MIN + 1`, since the
/// conversion yields `i32::MIN` and `v` then compares greater.
#[must_use]
pub const fn ceil(v: f32) -> i32 {
    let i = trunc_i32(v as f64);
    if v > i as f32 { i.wrapping_add(1) } else { i }
}

/// `value` limited to `min..=max`.
///
/// The lower bound is checked first, so if `min > max` any value below `min`
/// gives `min` and any other value above `max` gives `max`. Never panics.
#[must_use]
pub const fn clamp(value: i32, min: i32, max: i32) -> i32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// [`clamp`] for single-precision values, with the same bound order.
///
/// A NaN `value` fails both comparisons and is returned unchanged; a NaN
/// bound is never selected.
#[must_use]
pub const fn clamp_f32(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// `a / b` rounded towards negative infinity, for `b > 0`.
///
/// A negative `a` is divided as `-((-a - 1) / b) - 1` with truncating
/// division. For two's-complement integers `-a - 1` is `!a`, which is
/// non-negative for every negative `a` including `i32::MIN`, so no step can
/// overflow. For `b < 0` the formula is still applied and the result is not a
/// floor division (`int_floor_div(-1, -2)` is `-1`).
///
/// # Panics
///
/// If `b` is zero, like the `/` operator.
#[must_use]
pub const fn int_floor_div(a: i32, b: i32) -> i32 {
    if a < 0 { !(!a / b) } else { a / b }
}

/// A uniformly distributed integer in `min_inclusive..=max_inclusive`.
///
/// If `min_inclusive >= max_inclusive` this returns `min_inclusive` without
/// drawing. Otherwise it is `random.next_int(max_inclusive - min_inclusive + 1)
/// + min_inclusive`, all in wrapping `i32` arithmetic.
///
/// A range wider than `i32::MAX` values wraps the bound to zero or below, so
/// per [`JavaRandom::next_int`] the result is `min_inclusive` after one draw.
/// The full range `i32::MIN..=i32::MAX` is one such case.
pub const fn next_int(random: &mut JavaRandom, min_inclusive: i32, max_inclusive: i32) -> i32 {
    if min_inclusive >= max_inclusive {
        return min_inclusive;
    }
    let bound = max_inclusive.wrapping_sub(min_inclusive).wrapping_add(1);
    random.next_int(bound).wrapping_add(min_inclusive)
}

/// `input` degrees wrapped into `[-180, 180)`.
///
/// Takes the exact remainder of `input` by `360` (sign of `input`), then
/// moves it by one `±360` step if it is still outside the range; every step
/// is exact, so the result is exactly `input - 360·n` for the unique integer
/// `n` that lands in range. A non-zero multiple of `360` gives `+0.0`; only
/// `-0.0` itself gives `-0.0`. NaN and infinities give NaN.
#[must_use]
pub const fn wrap_degrees(input: f64) -> f64 {
    let r = input % 360.0;
    if r >= 180.0 {
        r - 360.0
    } else if r < -180.0 {
        r + 360.0
    } else if r == 0.0 && input != 0.0 {
        // The remainder of a negative multiple of 360 is -0.0.
        0.0
    } else {
        r
    }
}

/// [`wrap_degrees`] in single precision.
#[must_use]
pub const fn wrap_degrees_f32(input: f32) -> f32 {
    let r = input % 360.0;
    if r >= 180.0 {
        r - 360.0
    } else if r < -180.0 {
        r + 360.0
    } else if r == 0.0 && input != 0.0 {
        // The remainder of a negative multiple of 360 is -0.0.
        0.0
    } else {
        r
    }
}
