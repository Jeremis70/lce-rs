//! Behavioural unit tests for [`Vec3`]. Bit-exact test vectors live in
//! `vec3_golden.rs`.

use lce_math::vec3::Vec3;

/// The parallel-segment threshold is `1e-7` rounded to f32, widened to f64
/// (`1.0000000116860974e-7`). A squared delta between the two must be
/// rejected; an exact `1e-7` threshold would accept it.
#[test]
fn clip_epsilon_is_widened_f32() {
    let d = 1.000_000_005e-7_f64.sqrt();
    let sq = d * d;
    assert!(sq > 1e-7 && sq < f64::from(1e-7_f32));

    let a = Vec3::ZERO;
    assert_eq!(a.clip_x(Vec3::new(d, 0.0, 0.0), d * 0.5), None);
}

#[test]
fn canonical_unsigns_zero_but_new_does_not() {
    let raw = Vec3::new(-0.0, -0.0, -0.0);
    assert!(raw.x.is_sign_negative());

    let c = Vec3::canonical(-0.0, -0.0, -1.5);
    assert!(c.x.is_sign_positive());
    assert!(c.y.is_sign_positive());
    assert_eq!(c.z, -1.5);
}

#[test]
fn arithmetic() {
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, -5.0, 6.0);
    assert_eq!(a + b, Vec3::new(5.0, -3.0, 9.0));
    assert_eq!(b - a, Vec3::new(3.0, -7.0, 3.0));
    assert_eq!(a * 2.0, Vec3::new(2.0, 4.0, 6.0));
    assert_eq!(a.dot(b), 12.0);
    assert_eq!(a.cross(b), Vec3::new(27.0, 6.0, -13.0));
    assert_eq!(Vec3::new(3.0, 4.0, 0.0).length(), 5.0);
    assert_eq!(a.distance_sqr(b), 67.0);
    assert_eq!(Vec3::ZERO.distance(Vec3::new(0.0, 3.0, 4.0)), 5.0);
}

#[test]
fn lerp_endpoints() {
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(5.0, 6.0, 7.0);
    assert_eq!(a.lerp(b, 0.0), a);
    assert_eq!(a.lerp(b, 1.0), b);
    assert_eq!(a.lerp(b, 0.5), Vec3::new(3.0, 4.0, 5.0));
}

#[test]
fn normalize_threshold() {
    assert_eq!(Vec3::new(0.0, 0.0, 0.00009).normalize(), Vec3::ZERO);
    assert_eq!(
        Vec3::new(0.0, 0.0, 0.0001).normalize(),
        Vec3::new(0.0, 0.0, 1.0)
    );
    // The zero result is positive zero even for negative input.
    assert!(
        Vec3::new(-0.00001, 0.0, 0.0)
            .normalize()
            .x
            .is_sign_positive()
    );
}

#[test]
fn clip_hits_plane() {
    let a = Vec3::new(0.0, 0.0, 0.0);
    let b = Vec3::new(2.0, 4.0, 6.0);
    assert_eq!(a.clip_x(b, 1.0), Some(Vec3::new(1.0, 2.0, 3.0)));
    assert_eq!(a.clip_y(b, 1.0), Some(Vec3::new(0.5, 1.0, 1.5)));
    assert_eq!(a.clip_z(b, 3.0), Some(Vec3::new(1.0, 2.0, 3.0)));
}

#[test]
fn clip_endpoints_are_inclusive() {
    let a = Vec3::new(0.0, 0.0, 0.0);
    let b = Vec3::new(1.0, 1.0, 1.0);
    assert_eq!(a.clip_x(b, 0.0), Some(a));
    assert_eq!(a.clip_x(b, 1.0), Some(b));
    assert_eq!(a.clip_x(b, 1.0000001), None);
    assert_eq!(a.clip_x(b, -0.0000001), None);
}

#[test]
fn clip_rejects_parallel_segment() {
    let a = Vec3::new(0.0, 0.0, 0.0);
    // Squared delta 1e-7 is below the widened-f32 epsilon.
    let b = Vec3::new(1.0e-7_f64.sqrt(), 5.0, 5.0);
    assert_eq!(a.clip_x(b, 0.0), None);
    assert!(a.clip_y(b, 1.0).is_some());
}

#[test]
fn clip_nan_passes_through() {
    let a = Vec3::new(0.0, 0.0, 0.0);
    let b = Vec3::new(1.0, 1.0, 1.0);
    let hit = a
        .clip_x(b, f64::NAN)
        .expect("NaN parameter is not rejected");
    assert!(hit.x.is_nan());
}

#[test]
fn rotations_quarter_turn() {
    let half_pi = std::f32::consts::FRAC_PI_2;
    let close = |a: Vec3, b: Vec3| a.distance(b) < 1e-6;

    assert!(close(
        Vec3::new(0.0, 1.0, 0.0).rotate_x(half_pi),
        Vec3::new(0.0, 0.0, -1.0)
    ));
    assert!(close(
        Vec3::new(1.0, 0.0, 0.0).rotate_y(half_pi),
        Vec3::new(0.0, 0.0, -1.0)
    ));
    assert!(close(
        Vec3::new(1.0, 0.0, 0.0).rotate_z(half_pi),
        Vec3::new(0.0, -1.0, 0.0)
    ));
}

#[test]
fn rotation_uses_single_precision_trig() {
    let angle = 0.3_f32;
    let v = Vec3::new(1.0, 0.0, 0.0).rotate_y(angle);
    assert_eq!(v.x.to_bits(), f64::from(angle.cos()).to_bits());
}

#[test]
fn display_matches_printf() {
    assert_eq!(
        Vec3::new(1.0, -2.5, 0.125).to_string(),
        "(1.000000,-2.500000,0.125000)"
    );
}
