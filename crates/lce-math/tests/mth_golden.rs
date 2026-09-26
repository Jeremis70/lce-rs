//! Bit-exact test vectors for [`lce_math::mth`].
//!
//! `fixtures/mth_golden.json` holds the complete 65536-entry sine table, not a
//! sample of it, plus operand/result pairs for every other function. Operands
//! and results are raw hex: `i32`/`i64` as two's-complement bits, `f32`/`f64`
//! as IEEE-754 bits. Every result, NaN included, must match bit-for-bit.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use lce_math::mth;
use serde::Deserialize;

const FIXTURE: &str = include_str!("fixtures/mth_golden.json");

#[derive(Deserialize)]
struct Fixture {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    op: String,
    /// Operands as hex bit patterns; their width depends on `op`.
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

fn hex_u32(s: &str) -> u32 {
    u32::try_from(hex_u64(s)).expect("32-bit hex")
}

/// A result reduced to its raw bits, so NaN payloads and signed zeros compare
/// exactly.
#[derive(Debug, PartialEq, Eq)]
enum Bits {
    B32(u32),
    B64(u64),
}

impl Case {
    fn f32s(&self) -> Vec<f32> {
        self.a.iter().map(|s| f32::from_bits(hex_u32(s))).collect()
    }

    fn f64s(&self) -> Vec<f64> {
        self.a.iter().map(|s| f64::from_bits(hex_u64(s))).collect()
    }

    fn i32s(&self) -> Vec<i32> {
        self.a.iter().map(|s| hex_u32(s) as i32).collect()
    }

    fn f32_arg(&self) -> f32 {
        let [v] = self.f32s()[..] else {
            panic!("`{}` takes one f32", self.op);
        };
        v
    }

    fn f64_arg(&self) -> f64 {
        let [v] = self.f64s()[..] else {
            panic!("`{}` takes one f64", self.op);
        };
        v
    }

    /// The Rust result for a single-output op.
    fn run(&self) -> Bits {
        use Bits::{B32, B64};

        let int = |v: i32| B32(v as u32);
        match self.op.as_str() {
            "sin" => B32(mth::sin(self.f32_arg()).to_bits()),
            "cos" => B32(mth::cos(self.f32_arg()).to_bits()),
            "floor" => int(mth::floor(self.f64_arg())),
            "floor_f32" => int(mth::floor_f32(self.f32_arg())),
            "lfloor" => B64(mth::lfloor(self.f64_arg()) as u64),
            "fast_floor" => int(mth::fast_floor(self.f64_arg())),
            "ceil" => int(mth::ceil(self.f32_arg())),
            "clamp" => {
                let [v, lo, hi] = self.i32s()[..] else {
                    panic!("clamp takes three i32");
                };
                int(mth::clamp(v, lo, hi))
            }
            "clamp_f32" => {
                let [v, lo, hi] = self.f32s()[..] else {
                    panic!("clamp_f32 takes three f32");
                };
                B32(mth::clamp_f32(v, lo, hi).to_bits())
            }
            "int_floor_div" => {
                let [a, b] = self.i32s()[..] else {
                    panic!("int_floor_div takes two i32");
                };
                int(mth::int_floor_div(a, b))
            }
            "wrap_degrees" => B64(mth::wrap_degrees(self.f64_arg()).to_bits()),
            "wrap_degrees_f32" => B32(mth::wrap_degrees_f32(self.f32_arg()).to_bits()),
            other => panic!("fixture op `{other}` has no Rust mapping"),
        }
    }

    fn expected(&self) -> Bits {
        let [e] = &self.expect[..] else {
            panic!("`{}` expects one value", self.op);
        };
        let digits = e.len() - 2;
        match digits {
            8 => Bits::B32(hex_u32(e)),
            16 => Bits::B64(hex_u64(e)),
            _ => panic!("`{e}` is neither 32- nor 64-bit hex"),
        }
    }

    fn check(&self, index: usize) {
        assert_eq!(
            self.run(),
            self.expected(),
            "case #{index} `{}` a={:?}",
            self.op,
            self.a
        );
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

/// The one case with the given op.
fn single(op: &str) -> &'static Case {
    let mut found = CASES.iter().filter(|c| c.op == op);
    let case = found
        .next()
        .unwrap_or_else(|| panic!("fixture has no `{op}`"));
    assert!(found.next().is_none(), "fixture has more than one `{op}`");
    case
}

const CONSTANTS: &[&str] = &["constants"];
const SIN_TABLE: &[&str] = &["sin_table"];
const TRIG: &[&str] = &["sin", "cos"];
const FLOORS: &[&str] = &["floor", "floor_f32", "lfloor", "fast_floor", "ceil"];
const CLAMPS: &[&str] = &["clamp", "clamp_f32"];
const DIVISION: &[&str] = &["int_floor_div"];
const WRAPPING: &[&str] = &["wrap_degrees", "wrap_degrees_f32"];

#[test]
fn constants() {
    let expected: Vec<u32> = single("constants")
        .expect
        .iter()
        .map(|s| hex_u32(s))
        .collect();
    let actual = [mth::PI, mth::DEG_TO_RAD, mth::RAD_TO_DEG].map(f32::to_bits);
    assert_eq!(actual[..], expected[..], "PI, DEG_TO_RAD, RAD_TO_DEG");
}

#[test]
fn sin_table_matches_every_entry() {
    let expected = &single("sin_table").expect;
    let table = mth::sin_table();
    assert_eq!(expected.len(), table.len(), "fixture holds the whole table");
    let mismatches: Vec<_> = table
        .iter()
        .zip(expected)
        .enumerate()
        .filter(|&(_, (&actual, e))| actual.to_bits() != hex_u32(e))
        .map(|(i, (actual, e))| format!("[{i}] got {:#010x}, want {e}", actual.to_bits()))
        .collect();
    assert!(
        mismatches.is_empty(),
        "{} of {} entries differ, first: {:?}",
        mismatches.len(),
        table.len(),
        &mismatches[..mismatches.len().min(8)]
    );
}

#[test]
fn trig_lookups() {
    check_ops(TRIG);
}

#[test]
fn floors() {
    check_ops(FLOORS);
}

#[test]
fn clamps() {
    check_ops(CLAMPS);
}

#[test]
fn floor_division() {
    check_ops(DIVISION);
}

#[test]
fn degree_wrapping() {
    check_ops(WRAPPING);
}

/// Guards against the fixture gaining an op that no test above exercises.
#[test]
fn every_fixture_op_is_tested() {
    let tested: BTreeSet<&str> = [
        CONSTANTS, SIN_TABLE, TRIG, FLOORS, CLAMPS, DIVISION, WRAPPING,
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
