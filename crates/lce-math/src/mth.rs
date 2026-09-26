//! `Mth`: the 65536-entry sine lookup table, floor/clamp/angle-wrapping helpers.

/// The largest integer not greater than `v`.
///
/// Computed by truncating towards zero and stepping down by one when the
/// truncation rounded a negative value up. Only inputs whose floor fits in
/// an `i32` are covered by the specified behaviour; NaN and out-of-range
/// inputs are not yet pinned down.
#[must_use]
pub fn floor(v: f64) -> i32 {
    let i = v as i32;
    if v < f64::from(i) {
        i.wrapping_sub(1)
    } else {
        i
    }
}
