//! Behavioural unit tests for [`lce_math::random`] and [`lce_math::mth::next_int`].
//! Bit-exact test vectors live in `random_golden.rs`.

use lce_math::mth;
use lce_math::random::JavaRandom;

/// A copy of `random` advanced by one 31-bit draw.
fn after_one_draw(random: &JavaRandom) -> JavaRandom {
    let mut copy = random.clone();
    copy.next_bits::<31>();
    copy
}

#[test]
fn well_known_java_util_random_values() {
    let mut r = JavaRandom::new(42);
    let ints: Vec<i32> = (0..4).map(|_| r.next_int_unbounded()).collect();
    assert_eq!(
        ints,
        [-1_170_105_035, 234_785_527, -1_360_544_799, 205_897_768]
    );

    let mut r = JavaRandom::new(42);
    let digits: Vec<i32> = (0..10).map(|_| r.next_int(10)).collect();
    assert_eq!(digits, [0, 3, 8, 4, 0, 5, 5, 8, 9, 3]);

    assert_eq!(JavaRandom::new(0).next_int_unbounded(), -1_155_484_576);
    assert_eq!(JavaRandom::new(0).next_long(), -4_962_768_465_676_381_896);
    assert_eq!(JavaRandom::new(42).next_long(), -5_025_562_857_975_149_833);
    assert_eq!(JavaRandom::new(0).next_double(), 0.730_967_787_376_657);
    assert_eq!(JavaRandom::new(42).next_double(), 0.727_563_680_032_868_1);
    assert_eq!(JavaRandom::new(0).next_float(), 0.730_967_76);
}

#[test]
fn only_the_low_48_seed_bits_matter() {
    let base = JavaRandom::new(42);
    assert_eq!(JavaRandom::new(42 | (0x1234 << 48)), base);
    assert_eq!(JavaRandom::new(42 | i64::MIN), base);
    assert_ne!(JavaRandom::new(42 | (1 << 47)), base);
}

#[test]
fn a_seed_equal_to_the_multiplier_starts_from_zero() {
    // The first state is then just the increment, 0xB, whose top bits are 0.
    let mut r = JavaRandom::new(0x5_DEEC_E66D);
    assert_eq!(r.next_int_unbounded(), 0);
    assert_ne!(r.next_int_unbounded(), 0);
}

#[test]
fn set_seed_restarts_the_sequence() {
    let mut r = JavaRandom::new(7);
    for _ in 0..10 {
        r.next_long();
    }
    r.set_seed(42);
    assert_eq!(r, JavaRandom::new(42));
    assert_eq!(r.next_int_unbounded(), -1_170_105_035);
}

#[test]
fn next_bits_are_the_top_bits_of_one_step() {
    let r = JavaRandom::new(12345);
    let full = r.clone().next_int_unbounded() as u32;
    assert_eq!(r.clone().next_bits::<1>() as u32, full >> 31);
    assert_eq!(r.clone().next_bits::<8>() as u32, full >> 24);
    assert_eq!(r.clone().next_bits::<31>() as u32, full >> 1);
    assert!(r.clone().next_bits::<31>() >= 0);
}

#[test]
fn next_boolean_is_the_top_bit() {
    let mut a = JavaRandom::new(99);
    let mut b = a.clone();
    for _ in 0..100 {
        assert_eq!(a.next_boolean(), b.next_int_unbounded() < 0);
    }
}

#[test]
fn next_int_stays_in_range() {
    let mut r = JavaRandom::new(3);
    for bound in [
        1,
        2,
        3,
        7,
        10,
        16,
        17,
        1000,
        1 << 30,
        (1 << 30) + 1,
        i32::MAX,
    ] {
        for _ in 0..500 {
            let v = r.next_int(bound);
            assert!((0..bound).contains(&v), "next_int({bound}) = {v}");
        }
    }
    assert_eq!(r.next_int(1), 0);
}

#[test]
fn power_of_two_bounds_take_exactly_one_draw() {
    let mut r = JavaRandom::new(5);
    for shift in 0..31 {
        let bound = 1 << shift;
        let expected_state = after_one_draw(&r);
        let draw = r.clone().next_bits::<31>();
        assert_eq!(r.next_int(bound), draw >> (31 - shift), "bound {bound}");
        assert_eq!(r, expected_state, "bound {bound}");
    }
}

#[test]
fn other_bounds_reject_draws_from_the_incomplete_last_block() {
    // With bound 2^30 + 1 only draws below 2^30 + 1 are accepted (the second
    // block is incomplete), so about half are rejected.
    let bound = (1 << 30) + 1;
    let mut r = JavaRandom::new(11);
    let (mut rejected, mut accepted) = (0, 0);
    for _ in 0..1000 {
        let mut expected = r.clone();
        let mut draw = expected.next_bits::<31>();
        while draw >= bound {
            rejected += 1;
            draw = expected.next_bits::<31>();
        }
        accepted += 1;
        assert_eq!(r.next_int(bound), draw);
        assert_eq!(r, expected);
    }
    let share = f64::from(rejected) / f64::from(rejected + accepted);
    assert!(
        (0.45..0.55).contains(&share),
        "{rejected} rejected, {accepted} accepted"
    );
}

#[test]
fn non_positive_bounds_give_zero_after_one_draw() {
    let mut r = JavaRandom::new(8);
    for bound in [0, -1, -2, -16, -1000, i32::MIN + 1, i32::MIN] {
        let expected_state = after_one_draw(&r);
        assert_eq!(r.next_int(bound), 0, "bound {bound}");
        assert_eq!(r, expected_state, "bound {bound}");
    }
}

#[test]
fn next_long_adds_the_second_draw_as_signed() {
    let mut r = JavaRandom::new(1);
    for _ in 0..200 {
        let mut copy = r.clone();
        let high = i64::from(copy.next_int_unbounded());
        let low = i64::from(copy.next_int_unbounded());
        assert_eq!(r.next_long(), (high << 32).wrapping_add(low));
        assert_eq!(r, copy);
    }
}

#[test]
fn floats_and_doubles_are_exact_fractions_in_the_unit_interval() {
    let mut r = JavaRandom::new(2);
    for _ in 0..1000 {
        let mut copy = r.clone();
        let f = r.next_float();
        assert!((0.0..1.0).contains(&f));
        assert_eq!(
            f64::from(f) * f64::from(1 << 24),
            f64::from(copy.next_bits::<24>())
        );

        let d = r.next_double();
        let high = i64::from(copy.next_bits::<26>());
        let low = i64::from(copy.next_bits::<27>());
        assert!((0.0..1.0).contains(&d));
        assert_eq!(d * (1u64 << 53) as f64, ((high << 27) + low) as f64);
        assert_eq!(r, copy);
    }
}

#[test]
fn next_bytes_takes_one_draw_per_byte() {
    let mut r = JavaRandom::new(4);
    let mut copy = r.clone();
    let mut bytes = [0; 9];
    r.next_bytes(&mut bytes);
    let expected: Vec<u8> = (0..9).map(|_| copy.next_bits::<8>() as u8).collect();
    assert_eq!(bytes[..], expected[..]);
    assert_eq!(r, copy);

    let before = r.clone();
    r.next_bytes(&mut []);
    assert_eq!(r, before);
}

#[test]
fn gaussians_come_in_pairs_and_the_second_is_cached() {
    let mut r = JavaRandom::new(42);
    let first = r.next_gaussian();
    // Other draws in between leave the cached value in place.
    r.next_int_unbounded();
    // Cloning carries the cache along.
    let mut twin = r.clone();
    let second = r.next_gaussian();
    assert_ne!(first, second);
    // Handing out the cached value draws nothing.
    assert_eq!(r.next_int_unbounded(), twin.next_int_unbounded());
    assert_eq!(twin.next_gaussian(), second);
    assert_eq!(r, twin);
    // With the cache empty, the next call starts a new pair.
    let before_third = r.clone();
    assert_ne!(r.next_gaussian(), second);
    assert_ne!(r, before_third);
}

#[test]
fn set_seed_discards_a_cached_gaussian() {
    let mut r = JavaRandom::new(42);
    let first = r.next_gaussian();
    r.set_seed(42);
    assert_eq!(r, JavaRandom::new(42));
    assert_eq!(r.next_gaussian(), first);
}

#[test]
fn gaussians_follow_the_polar_method() {
    let mut r = JavaRandom::new(0);
    let mut copy = r.clone();
    let first = r.next_gaussian();
    let second = r.next_gaussian();
    let (v1, v2, s) = loop {
        let v1 = 2.0 * copy.next_double() - 1.0;
        let v2 = 2.0 * copy.next_double() - 1.0;
        let s = v1 * v1 + v2 * v2;
        if s < 1.0 && s != 0.0 {
            break (v1, v2, s);
        }
    };
    let m = (-2.0 * s.ln() / s).sqrt();
    assert_eq!((first, second), (v1 * m, v2 * m));
    assert_eq!(r, copy);
    // Well-known value for this seed; allow for last-bit libm differences.
    assert!((first - 0.802_533_063_739_030_5).abs() < 1e-15, "{first}");
}

#[test]
fn gaussian_samples_have_unit_variance() {
    let mut r = JavaRandom::new(2024);
    let n = 100_000;
    let samples: Vec<f64> = (0..n).map(|_| r.next_gaussian()).collect();
    let mean = samples.iter().sum::<f64>() / f64::from(n);
    let variance = samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / f64::from(n);
    assert!(mean.abs() < 0.02, "mean {mean}");
    assert!((variance - 1.0).abs() < 0.02, "variance {variance}");
}

#[test]
fn ranged_next_int_is_inclusive() {
    let mut r = JavaRandom::new(6);
    let mut seen = [false; 5];
    for _ in 0..500 {
        let v = mth::next_int(&mut r, -2, 2);
        assert!((-2..=2).contains(&v), "{v}");
        seen[(v + 2) as usize] = true;
    }
    assert_eq!(seen, [true; 5]);
}

#[test]
fn ranged_next_int_is_an_offset_bounded_draw() {
    let mut a = JavaRandom::new(10);
    let mut b = a.clone();
    for _ in 0..100 {
        assert_eq!(mth::next_int(&mut a, 100, 199), b.next_int(100) + 100);
    }
    assert_eq!(a, b);
}

#[test]
fn ranged_next_int_of_an_empty_or_single_range_does_not_draw() {
    let mut r = JavaRandom::new(12);
    let before = r.clone();
    assert_eq!(mth::next_int(&mut r, 5, 5), 5);
    assert_eq!(mth::next_int(&mut r, 5, -5), 5);
    assert_eq!(mth::next_int(&mut r, i32::MAX, i32::MIN), i32::MAX);
    assert_eq!(r, before);
}

#[test]
fn ranged_next_int_wider_than_i32_max_gives_the_minimum() {
    for (lo, hi) in [
        (i32::MIN, i32::MAX),
        (i32::MIN, -1),
        (-1, i32::MAX),
        (0, i32::MAX),
    ] {
        let mut r = JavaRandom::new(13);
        let expected_state = after_one_draw(&r);
        assert_eq!(mth::next_int(&mut r, lo, hi), lo, "{lo}..={hi}");
        assert_eq!(r, expected_state, "{lo}..={hi}");
    }
    // One value narrower still draws normally.
    let mut r = JavaRandom::new(13);
    let v = mth::next_int(&mut r, 1, i32::MAX);
    assert!(v >= 1);
}

#[test]
fn usable_in_const_context() {
    const FIRST: i32 = JavaRandom::new(42).next_int_unbounded();
    const DIGIT: i32 = {
        let mut r = JavaRandom::new(42);
        r.next_int(10);
        r.next_int(10)
    };
    const RANGED: i32 = mth::next_int(&mut JavaRandom::new(42), 10, 19);
    assert_eq!((FIRST, DIGIT, RANGED), (-1_170_105_035, 3, 10));
}
