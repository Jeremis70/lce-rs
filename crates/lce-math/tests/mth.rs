//! Behavioural unit tests for [`lce_math::mth`].
//! Bit-exact test vectors live in `mth_golden.rs`.

use lce_math::mth::{
    DEG_TO_RAD, PI, RAD_TO_DEG, ceil, clamp, clamp_f32, cos, fast_floor, floor, floor_f32,
    int_floor_div, lfloor, sin, sin_table, wrap_degrees, wrap_degrees_f32,
};

#[test]
fn pi_is_the_single_precision_constant() {
    assert_eq!(PI.to_bits(), 0x4049_0fdb);
    // 3.141592654 lies above the midpoint of its two f32 neighbours, so it
    // rounds to the same f32 as π itself...
    assert_eq!(PI.to_bits(), std::f32::consts::PI.to_bits());
    // ...but in f64 arithmetic it widens to that f32, not to f64 π.
    assert_eq!(f64::from(PI), 3.141_592_741_012_573_2);
    assert_ne!(f64::from(PI), std::f64::consts::PI);
}

#[test]
fn angle_factors_are_single_precision_quotients() {
    assert_eq!(DEG_TO_RAD, PI / 180.0);
    assert_eq!(RAD_TO_DEG, 180.0 / PI);
    assert_eq!(180.0 * DEG_TO_RAD, PI);
}

#[test]
fn sin_table_hits_the_exact_quarter_turns() {
    let t = sin_table();
    assert_eq!(t.len(), 65536);
    assert_eq!(t[0].to_bits(), 0.0f32.to_bits());
    assert_eq!(t[16384], 1.0);
    assert_eq!(t[49152], -1.0);
    // The half-turn angle is PI in f32, which is slightly above π.
    assert!(t[32768] < 0.0 && t[32768] > -1e-7);
}

#[test]
fn sin_table_is_close_to_the_true_sine() {
    for (i, &v) in sin_table().iter().enumerate() {
        let angle = i as f64 * std::f64::consts::TAU / 65536.0;
        assert!((f64::from(v) - angle.sin()).abs() < 1e-6, "entry {i}");
    }
}

#[test]
fn sin_reads_the_entry_at_or_below_the_angle() {
    let step = 2.0 * PI / 65536.0;
    assert_eq!(sin(0.0), 0.0);
    assert_eq!(sin(PI / 2.0), 1.0);
    // Anything below one step truncates to entry 0, on either side of zero.
    assert_eq!(sin(step * 0.9), 0.0);
    assert_eq!(sin(-step * 0.9), 0.0);
    assert_eq!(sin(-PI / 2.0), -1.0);
    // Negative angles wrap round the table.
    assert_eq!(sin(-PI / 2.0), sin(3.0 * PI / 2.0));
}

#[test]
fn cos_is_the_table_a_quarter_turn_ahead() {
    assert_eq!(cos(0.0), 1.0);
    assert_eq!(cos(PI / 2.0), sin(PI));
    assert_eq!(cos(PI), -1.0);
    for i in 0..1000 {
        let x = i as f32 * 0.0137 - 7.0;
        assert!((cos(x) - x.cos()).abs() < 1e-3, "cos({x})");
        assert!((sin(x) - x.sin()).abs() < 1e-3, "sin({x})");
    }
}

#[test]
fn trig_of_unrepresentable_angles_reads_entry_zero() {
    for x in [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        1e10,
        -1e10,
        205_888.0,
    ] {
        assert_eq!(sin(x), 0.0, "sin({x})");
        assert_eq!(cos(x), 0.0, "cos({x})");
    }
    // Just inside the convertible range the lookup still works.
    assert_ne!(sin(205_887.0), 0.0);
}

#[test]
fn floor_rounds_towards_negative_infinity() {
    assert_eq!(floor(0.0), 0);
    assert_eq!(floor(-0.0), 0);
    assert_eq!(floor(0.999), 0);
    assert_eq!(floor(-0.5), -1);
    assert_eq!(floor(-1.0), -1);
    assert_eq!(floor(-1e-300), -1);
    assert_eq!(floor(2_147_483_647.9), i32::MAX);
    assert_eq!(floor(-2_147_483_648.0), i32::MIN);
}

#[test]
fn floor_of_unrepresentable_values_follows_the_conversion_rule() {
    assert_eq!(floor(f64::NAN), i32::MIN);
    assert_eq!(floor(f64::INFINITY), i32::MIN);
    assert_eq!(floor(2_147_483_648.0), i32::MIN);
    // Below the range the conversion gives i32::MIN, then the step down wraps.
    assert_eq!(floor(-2_147_483_648.5), i32::MAX);
    assert_eq!(floor(f64::NEG_INFINITY), i32::MAX);
}

#[test]
fn floor_f32_and_ceil_agree_with_float_rounding_in_range() {
    for i in -2000..2000 {
        let v = i as f32 * 0.37;
        assert_eq!(floor_f32(v), v.floor() as i32, "floor_f32({v})");
        assert_eq!(ceil(v), v.ceil() as i32, "ceil({v})");
    }
    assert_eq!(floor_f32(f32::NAN), i32::MIN);
    assert_eq!(floor_f32(f32::NEG_INFINITY), i32::MAX);
    assert_eq!(ceil(f32::NAN), i32::MIN);
    assert_eq!(ceil(f32::INFINITY), i32::MIN + 1);
    assert_eq!(ceil(f32::NEG_INFINITY), i32::MIN);
}

#[test]
fn lfloor_covers_the_i64_range() {
    assert_eq!(lfloor(-0.5), -1);
    assert_eq!(lfloor(4_294_967_296.5), 4_294_967_296);
    assert_eq!(lfloor(-4_294_967_296.5), -4_294_967_297);
    assert_eq!(lfloor(f64::NAN), i64::MIN);
    assert_eq!(lfloor(9.3e18), i64::MIN);
    assert_eq!(lfloor(-9.3e18), i64::MAX);
}

#[test]
fn fast_floor_is_only_a_floor_down_to_minus_1024() {
    for i in -4096..4096 {
        let v = f64::from(i) * 0.25;
        assert_eq!(fast_floor(v), floor(v), "fast_floor({v})");
    }
    assert_eq!(fast_floor(-1024.5), -1024);
    assert_eq!(fast_floor(-1025.5), -1025);
    assert_eq!(fast_floor(-2048.75), -2048);
    assert_eq!(fast_floor(f64::NAN), 2_147_482_624);
}

#[test]
fn clamp_checks_the_lower_bound_first() {
    assert_eq!(clamp(5, 0, 10), 5);
    assert_eq!(clamp(-5, 0, 10), 0);
    assert_eq!(clamp(15, 0, 10), 10);
    // Inverted bounds never panic.
    assert_eq!(clamp(-5, 10, 0), 10);
    assert_eq!(clamp(5, 10, 0), 10);
    assert_eq!(clamp(15, 10, 0), 0);
    assert_eq!(clamp(i32::MIN, i32::MIN, i32::MAX), i32::MIN);
}

#[test]
fn clamp_f32_passes_nan_through() {
    assert_eq!(clamp_f32(0.5, 0.0, 1.0), 0.5);
    assert_eq!(clamp_f32(-90.5, -90.0, 90.0), -90.0);
    assert_eq!(clamp_f32(1e30, -90.0, 90.0), 90.0);
    assert!(clamp_f32(f32::NAN, -90.0, 90.0).is_nan());
    assert_eq!(clamp_f32(3.0, f32::NAN, f32::NAN), 3.0);
}

#[test]
fn int_floor_div_is_floor_division_for_positive_divisors() {
    for a in -100..100 {
        for b in 1..20 {
            assert_eq!(int_floor_div(a, b), a.div_euclid(b), "{a} / {b}");
        }
    }
    assert_eq!(int_floor_div(-1, 16), -1);
    assert_eq!(int_floor_div(-16, 16), -1);
    assert_eq!(int_floor_div(-17, 16), -2);
    assert_eq!(int_floor_div(i32::MIN, 1), i32::MIN);
    assert_eq!(int_floor_div(i32::MIN, 16), -134_217_728);
}

#[test]
fn int_floor_div_applies_the_same_formula_for_negative_divisors() {
    assert_eq!(int_floor_div(7, -2), -3);
    assert_eq!(int_floor_div(-1, -2), -1);
    assert_eq!(int_floor_div(-7, -2), 2);
    assert_eq!(int_floor_div(i32::MIN, -1), i32::MAX - 1);
}

#[test]
#[should_panic(expected = "divide by zero")]
fn int_floor_div_by_zero_panics() {
    let _ = int_floor_div(std::hint::black_box(1), std::hint::black_box(0));
}

#[test]
fn wrap_degrees_lands_in_the_half_open_range() {
    for i in -5000..5000 {
        let v = f64::from(i) * 0.731;
        let w = wrap_degrees(v);
        assert!((-180.0..180.0).contains(&w), "wrap_degrees({v}) = {w}");
        let turns = (v - w) / 360.0;
        assert_eq!(turns, turns.round(), "wrap_degrees({v}) = {w}");
    }
    assert_eq!(wrap_degrees(180.0), -180.0);
    assert_eq!(wrap_degrees(-180.0), -180.0);
    assert_eq!(wrap_degrees(539.5), 179.5);
    assert_eq!(wrap_degrees_f32(-180.5), 179.5);
    assert_eq!(wrap_degrees_f32(359.0), -1.0);
}

#[test]
fn wrap_degrees_signed_zeros() {
    assert_eq!(wrap_degrees(-0.0).to_bits(), (-0.0f64).to_bits());
    assert_eq!(wrap_degrees(-360.0).to_bits(), 0.0f64.to_bits());
    assert_eq!(wrap_degrees(720.0).to_bits(), 0.0f64.to_bits());
    assert_eq!(wrap_degrees_f32(-0.0).to_bits(), (-0.0f32).to_bits());
    assert_eq!(wrap_degrees_f32(-1080.0).to_bits(), 0.0f32.to_bits());
}

#[test]
fn wrap_degrees_of_huge_and_non_finite_input() {
    // The remainder is exact at any magnitude: 2^60 = 360·n + 136.
    assert_eq!(wrap_degrees(1_152_921_504_606_846_976.0), 136.0);
    assert!(wrap_degrees(f64::NAN).is_nan());
    assert!(wrap_degrees(f64::INFINITY).is_nan());
    assert!(wrap_degrees_f32(f32::NEG_INFINITY).is_nan());
}

#[test]
fn helpers_are_usable_in_const_context() {
    const FLOORED: i32 = floor(-0.5);
    const WRAPPED: f32 = wrap_degrees_f32(270.0);
    const DIVIDED: i32 = int_floor_div(-17, 16);
    const SINE: f32 = sin(PI / 2.0);
    assert_eq!((FLOORED, WRAPPED, DIVIDED, SINE), (-1, -90.0, -2, 1.0));
}
