//! Behavioural unit tests for [`Pos`], [`TilePos`] and [`ChunkPos`].
//! Bit-exact test vectors live in `pos_golden.rs`.

use std::collections::HashSet;

use lce_math::pos::{ChunkPos, Pos, TilePos};
use lce_math::vec3::Vec3;

#[test]
fn directions_follow_axis_convention() {
    let p = Pos::new(10, 64, -3);
    assert_eq!(p.above(), Pos::new(10, 65, -3));
    assert_eq!(p.below(), Pos::new(10, 63, -3));
    assert_eq!(p.north(), Pos::new(10, 64, -4));
    assert_eq!(p.south(), Pos::new(10, 64, -2));
    assert_eq!(p.west(), Pos::new(9, 64, -3));
    assert_eq!(p.east(), Pos::new(11, 64, -3));
}

#[test]
fn multi_steps_apply_the_count() {
    let p = Pos::new(0, 0, 0);
    assert_eq!(p.above_n(5), Pos::new(0, 5, 0));
    assert_eq!(p.below_n(5), Pos::new(0, -5, 0));
    assert_eq!(p.north_n(5), Pos::new(0, 0, -5));
    assert_eq!(p.south_n(5), Pos::new(0, 0, 5));
    assert_eq!(p.west_n(5), Pos::new(-5, 0, 0));
    assert_eq!(p.east_n(5), Pos::new(5, 0, 0));
    // A negative count walks the other way.
    assert_eq!(p.west_n(-2), p.east_n(2));
}

#[test]
fn steps_wrap_at_the_integer_limits() {
    assert_eq!(Pos::new(i32::MAX, 0, 0).east(), Pos::new(i32::MIN, 0, 0));
    assert_eq!(Pos::new(0, i32::MIN, 0).below(), Pos::new(0, i32::MAX, 0));
    assert_eq!(
        Pos::new(1, 2, 3).offset(i32::MAX, 0, i32::MIN),
        Pos::new(i32::MIN, 2, i32::MIN + 3)
    );
}

#[test]
fn add_and_sub_are_componentwise() {
    let mut p = Pos::new(1, 2, 3);
    p += Pos::new(10, 20, 30);
    assert_eq!(p, Pos::new(11, 22, 33));
    p -= Pos::new(1, 2, 3);
    assert_eq!(p, Pos::new(10, 20, 30));
    assert_eq!(Pos::new(1, 1, 1) - Pos::new(2, 2, 2), Pos::new(-1, -1, -1));
}

#[test]
fn pos_hash_mixes_coordinates_with_shifts() {
    assert_eq!(Pos::ZERO.hash_code(), 0);
    assert_eq!(Pos::new(1, 2, 3).hash_code(), 1 + (3 << 8) + (2 << 16));
    // The hash only spreads nearby positions: these two collide.
    assert_eq!(Pos::new(256, 0, -1).hash_code(), Pos::ZERO.hash_code());
    // Large coordinates wrap instead of overflowing.
    assert_eq!(Pos::new(0, 0x8000, 0).hash_code(), i32::MIN);
}

#[test]
fn compare_orders_by_y_then_z_then_x() {
    let base = Pos::new(5, 5, 5);
    assert_eq!(base.compare_to(base), 0);
    assert!(base.compare_to(Pos::new(0, 6, 0)) < 0);
    assert!(base.compare_to(Pos::new(9, 5, 4)) > 0);
    assert!(base.compare_to(Pos::new(4, 5, 5)) > 0);
    // The result is the difference itself, not just its sign.
    assert_eq!(base.compare_to(Pos::new(0, 2, 0)), 3);
}

#[test]
fn compare_sign_flips_when_the_difference_overflows() {
    let low = Pos::new(0, 0, i32::MIN);
    let high = Pos::new(0, 0, 1);
    // `low.z` is the smaller coordinate, yet `low` compares as greater.
    assert_eq!(low.compare_to(high), i32::MAX);
}

#[test]
fn dist_is_euclidean() {
    let p = Pos::new(1, 2, 3);
    assert_eq!(p.dist(4, 6, 15), 13.0);
    assert_eq!(p.dist_to(Pos::new(-2, -2, -9)), 13.0);
    assert_eq!(Pos::ZERO.dist(46341, 0, 0), 46341.0);
}

#[test]
fn dist_y_and_z_terms_overflow_before_widening() {
    // 46341² exceeds i32::MAX. The x term is squared in f64, but the y and z
    // terms are squared as i32 and wrap negative, so the root is NaN.
    assert!(Pos::ZERO.dist(0, 46341, 0).is_nan());
    assert!(Pos::ZERO.dist(0, 0, -46341).is_nan());
    assert_eq!(Pos::ZERO.dist(0, 46340, 0), 46340.0);
}

#[test]
fn dist_sqr_is_integer_then_rounded_to_f32() {
    assert_eq!(Pos::ZERO.dist_sqr(1, 2, 2), 9.0);
    assert_eq!(Pos::new(-1, -1, -1).dist_sqr(1, 1, 1), 12.0);
    // 4097² = 16785409 is not representable in f32 and ties to even.
    assert_eq!(Pos::ZERO.dist_sqr(4097, 0, 0), 16_785_408.0);
    // The integer sum wraps: 2 × 46341² = 4294976562 ≡ 9266 (mod 2^32).
    assert_eq!(Pos::ZERO.dist_sqr(46341, 46341, 0), 9266.0);
}

#[test]
fn tile_containing_floors_towards_negative_infinity() {
    let t = TilePos::containing(Vec3::new(-0.5, 64.0, -1e-300));
    assert_eq!(t, TilePos::new(-1, 64, -1));
    let t = TilePos::from(Vec3::new(0.999, -0.0, -64.0));
    assert_eq!(t, TilePos::new(0, 0, -64));
    let t = TilePos::containing(Vec3::new(-30_000_000.5, 255.99, 29_999_999.999));
    assert_eq!(t, TilePos::new(-30_000_001, 255, 29_999_999));
}

#[test]
fn tile_hash_is_a_wrapping_linear_combination() {
    assert_eq!(TilePos::new(0, 0, 0).hash_code(), 0);
    assert_eq!(TilePos::new(1, 0, 0).hash_code(), 8_976_890);
    assert_eq!(TilePos::new(0, 1, 0).hash_code(), 981_131);
    assert_eq!(TilePos::new(0, 0, -7).hash_code(), -7);
    // (2^31 - 1) × even ≡ −even (mod 2^32).
    assert_eq!(TilePos::new(i32::MAX, 0, 0).hash_code(), -8_976_890);
}

#[test]
fn chunk_long_hash_packs_x_low_and_z_high() {
    assert_eq!(ChunkPos::long_hash(0, 0), 0);
    assert_eq!(ChunkPos::long_hash(1, 0), 1);
    assert_eq!(ChunkPos::long_hash(0, 1), 1 << 32);
    // A negative x is not sign-extended into the z half...
    assert_eq!(ChunkPos::long_hash(-1, 0), 0xffff_ffff);
    // ...while a negative z makes the whole key negative.
    assert_eq!(ChunkPos::long_hash(0, -1), -(1 << 32));
    assert_eq!(ChunkPos::long_hash(-1, -1), -1);
    assert_eq!(
        ChunkPos::long_hash(i32::MIN, i32::MAX),
        0x7fff_ffff_8000_0000
    );
    assert_eq!(ChunkPos::new(3, -4).to_long(), ChunkPos::long_hash(3, -4));
}

#[test]
fn chunk_long_hash_never_collides() {
    let coords = [-2, -1, 0, 1, 2, i32::MIN, i32::MAX, 1 << 16, -(1 << 16)];
    let mut seen = HashSet::new();
    for x in coords {
        for z in coords {
            assert!(
                seen.insert(ChunkPos::long_hash(x, z)),
                "({x}, {z}) collided"
            );
        }
    }
}

#[test]
fn chunk_hash_is_x_xor_z() {
    assert_eq!(ChunkPos::new(5, 3).hash_code(), 6);
    assert_eq!(ChunkPos::new(-1, 0).hash_code(), -1);
    // Every diagonal chunk hashes to zero.
    for n in [-100, -1, 0, 1, 7, i32::MAX] {
        assert_eq!(ChunkPos::new(n, n).hash_code(), 0);
    }
}

#[test]
fn chunk_middle_is_tile_eight_of_sixteen() {
    let c = ChunkPos::new(0, 0);
    assert_eq!((c.middle_block_x(), c.middle_block_z()), (8, 8));
    let c = ChunkPos::new(-1, 2);
    assert_eq!((c.middle_block_x(), c.middle_block_z()), (-8, 40));
    assert_eq!(c.middle_block_position(70), TilePos::new(-8, 70, 40));
    // Shifting past the top bit wraps.
    let c = ChunkPos::new(0x0800_0000, -0x0800_0001);
    assert_eq!(c.middle_block_x(), i32::MIN + 8);
    assert_eq!(c.middle_block_z(), i32::MAX - 7);
}

#[test]
fn chunk_distance_is_measured_from_the_middle() {
    let c = ChunkPos::new(0, 0);
    assert_eq!(c.distance_to_sqr(8.0, 8.0), 0.0);
    assert_eq!(c.distance_to_sqr(0.0, 0.0), 128.0);
    assert_eq!(ChunkPos::new(-1, 1).distance_to_sqr(-8.0, 21.0), 9.0);
    assert!(c.distance_to_sqr(f64::NAN, 0.0).is_nan());
}

#[test]
fn chunk_display() {
    assert_eq!(ChunkPos::new(-3, 7).to_string(), "[-3, 7]");
    assert_eq!(
        ChunkPos::new(i32::MIN, i32::MAX).to_string(),
        "[-2147483648, 2147483647]"
    );
}
