//! Bit-exact test vectors for [`lce_math::random`] and [`lce_math::mth::next_int`].
//!
//! Each case in `fixtures/random_golden.json` seeds a fresh generator with
//! `seed` and calls `op` (with operands `a`) once per expected value, in
//! order; `next_bytes` is a single call filling as many bytes as are
//! expected. A `script` case instead drives one generator through a list of
//! different calls, including `set_seed` mid-sequence.
//!
//! Operands and results are raw hex: `i32`/`i64` as two's-complement bits,
//! `f32`/`f64` as IEEE-754 bits, booleans as `0`/`1`, bytes as two digits.
//! Results are compared as those strings, so width is checked too.
//!
//! The Gaussian cases call libm `ln`; the fixture records the libm it was
//! recorded with, and those cases only hold bit-for-bit on a matching libm.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use lce_math::mth;
use lce_math::random::JavaRandom;
use serde::Deserialize;

const FIXTURE: &str = include_str!("fixtures/random_golden.json");

#[derive(Deserialize)]
struct Fixture {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    op: String,
    seed: String,
    /// Operands as hex bit patterns; their width depends on `op`.
    #[serde(default)]
    a: Vec<String>,
    #[serde(default)]
    expect: Vec<String>,
    /// Only for `script`.
    #[serde(default)]
    steps: Vec<Step>,
}

#[derive(Deserialize)]
struct Step {
    call: String,
    #[serde(default)]
    a: Vec<String>,
    expect: Vec<String>,
}

static CASES: LazyLock<Vec<Case>> = LazyLock::new(|| {
    serde_json::from_str::<Fixture>(FIXTURE)
        .expect("fixture is valid JSON")
        .cases
});

fn hex_u64(s: &str) -> u64 {
    let digits = s.strip_prefix("0x").expect("hex literal has 0x prefix");
    u64::from_str_radix(digits, 16).expect("valid hex")
}

fn hex_i32(s: &str) -> i32 {
    u32::try_from(hex_u64(s)).expect("32-bit hex") as i32
}

fn hex32(v: u32) -> String {
    format!("{v:#010x}")
}

fn hex64(v: u64) -> String {
    format!("{v:#018x}")
}

fn int(v: i32) -> String {
    hex32(v as u32)
}

/// `random.next_bits::<bits>()` for a width only known at run time.
fn next_bits(random: &mut JavaRandom, bits: i32) -> i32 {
    macro_rules! dispatch {
        ($($n:literal)*) => {
            match bits {
                $($n => random.next_bits::<$n>(),)*
                _ => panic!("next_bits width {bits} is outside 1..=32"),
            }
        };
    }
    dispatch!(1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32)
}

/// Performs one call and returns its results; `expected_len` sizes
/// `next_bytes`.
fn call(random: &mut JavaRandom, op: &str, a: &[String], expected_len: usize) -> Vec<String> {
    let one = |v: String| vec![v];
    match (op, a) {
        ("next_bits", [bits]) => one(int(next_bits(random, hex_i32(bits)))),
        ("next_int_unbounded", []) => one(int(random.next_int_unbounded())),
        ("next_int", [bound]) => one(int(random.next_int(hex_i32(bound)))),
        ("next_long", []) => one(hex64(random.next_long() as u64)),
        ("next_boolean", []) => one(int(i32::from(random.next_boolean()))),
        ("next_float", []) => one(hex32(random.next_float().to_bits())),
        ("next_double", []) => one(hex64(random.next_double().to_bits())),
        ("next_gaussian", []) => one(hex64(random.next_gaussian().to_bits())),
        ("next_bytes", []) => {
            let mut bytes = vec![0; expected_len];
            random.next_bytes(&mut bytes);
            bytes.iter().map(|b| format!("{b:#04x}")).collect()
        }
        ("mth_next_int", [lo, hi]) => one(int(mth::next_int(random, hex_i32(lo), hex_i32(hi)))),
        ("set_seed", [seed]) => {
            random.set_seed(hex_u64(seed) as i64);
            Vec::new()
        }
        _ => panic!(
            "fixture call `{op}` with {} operands has no Rust mapping",
            a.len()
        ),
    }
}

impl Case {
    fn check(&self, index: usize) {
        let mut random = JavaRandom::new(hex_u64(&self.seed) as i64);
        let ctx = format!(
            "case #{index} `{}` seed={} a={:?}",
            self.op, self.seed, self.a
        );
        match self.op.as_str() {
            "script" => {
                assert!(!self.steps.is_empty(), "{ctx}: script has no steps");
                for (i, step) in self.steps.iter().enumerate() {
                    let got = call(&mut random, &step.call, &step.a, step.expect.len());
                    assert_eq!(
                        got, step.expect,
                        "{ctx}: step #{i} `{}` a={:?}",
                        step.call, step.a
                    );
                }
            }
            "next_bytes" => {
                let got = call(&mut random, "next_bytes", &self.a, self.expect.len());
                assert_eq!(got, self.expect, "{ctx}");
            }
            op => {
                assert!(!self.expect.is_empty(), "{ctx}: no expected values");
                for (i, expected) in self.expect.iter().enumerate() {
                    let got = call(&mut random, op, &self.a, 1);
                    assert_eq!(got, [expected.as_str()], "{ctx}: value #{i}");
                }
            }
        }
    }
}

/// Runs every fixture case whose op is in `ops`, asserting each one ran.
fn check_ops(ops: &[&str]) {
    let mut seen = BTreeSet::new();
    for (i, case) in CASES.iter().enumerate() {
        if ops.contains(&case.op.as_str()) {
            case.check(i);
            seen.insert(case.op.as_str());
        }
    }
    let missing: Vec<_> = ops.iter().filter(|op| !seen.contains(*op)).collect();
    assert!(missing.is_empty(), "fixture has no cases for {missing:?}");
}

const DRAWS: &[&str] = &["next_bits", "next_int_unbounded"];
const BOUNDED: &[&str] = &["next_int"];
const LONGS: &[&str] = &["next_long"];
const FLOATS: &[&str] = &["next_float", "next_double"];
const BOOLEANS_AND_BYTES: &[&str] = &["next_boolean", "next_bytes"];
const GAUSSIAN: &[&str] = &["next_gaussian"];
const RANGED: &[&str] = &["mth_next_int"];
const SCRIPTS: &[&str] = &["script"];

#[test]
fn raw_draws() {
    check_ops(DRAWS);
}

#[test]
fn bounded_ints() {
    check_ops(BOUNDED);
}

#[test]
fn longs() {
    check_ops(LONGS);
}

#[test]
fn floats_and_doubles() {
    check_ops(FLOATS);
}

#[test]
fn booleans_and_bytes() {
    check_ops(BOOLEANS_AND_BYTES);
}

#[test]
fn gaussians() {
    check_ops(GAUSSIAN);
}

#[test]
fn ranged_ints() {
    check_ops(RANGED);
}

#[test]
fn mixed_call_scripts() {
    check_ops(SCRIPTS);
}

/// Guards against the fixture gaining an op that no test above exercises.
#[test]
fn every_fixture_op_is_tested() {
    let tested: BTreeSet<&str> = [
        DRAWS,
        BOUNDED,
        LONGS,
        FLOATS,
        BOOLEANS_AND_BYTES,
        GAUSSIAN,
        RANGED,
        SCRIPTS,
    ]
    .concat()
    .into_iter()
    .collect();
    let untested: BTreeSet<&str> = CASES
        .iter()
        .map(|c| c.op.as_str())
        .filter(|op| !tested.contains(op))
        .collect();
    assert!(untested.is_empty(), "untested fixture ops: {untested:?}");
}
