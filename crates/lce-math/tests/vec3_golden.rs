//! Bit-exact test vectors for [`Vec3`].
//!
//! Every `f64`/`f32` in `fixtures/vec3_golden.json` is stored as its raw bit
//! pattern in hex, and every finite or infinite result — including the sign
//! of zero — must match bit-for-bit.
//!
//! NaN results only have to be NaN. When both operands of an operation are
//! NaN, IEEE 754 leaves unspecified which one propagates; in practice it is
//! decided by the operand order the compiler emits, and Rust makes no promise
//! about NaN bits either. The payload is therefore not a property of the
//! algorithm and is not compared.
//!
//! The rotation cases call libm `sinf`/`cosf`; the fixture records the libm it
//! was recorded with, and those cases only hold bit-for-bit on a matching
//! libm.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use lce_math::vec3::Vec3;
use serde::Deserialize;

const FIXTURE: &str = include_str!("fixtures/vec3_golden.json");

#[derive(Deserialize)]
struct Fixture {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    op: String,
    a: [String; 3],
    b: Option<[String; 3]>,
    s: Option<String>,
    angle: Option<String>,
    /// Components of the expected vector, a single scalar, or `null` for `None`.
    expect: Option<Vec<String>>,
    /// Expected `Display` output.
    text: Option<String>,
}

enum Outcome {
    Vector(Option<Vec3>),
    Scalar(f64),
    Text(String),
}

static CASES: LazyLock<Vec<Case>> = LazyLock::new(|| {
    serde_json::from_str::<Fixture>(FIXTURE)
        .expect("fixture is valid JSON")
        .cases
});

fn hex_u64(s: &str) -> u64 {
    let digits = s.strip_prefix("0x").expect("hex literal has 0x prefix");
    u64::from_str_radix(digits, 16).expect("valid 64-bit hex")
}

fn hex_u32(s: &str) -> u32 {
    let digits = s.strip_prefix("0x").expect("hex literal has 0x prefix");
    u32::from_str_radix(digits, 16).expect("valid 32-bit hex")
}

/// Bit-exact comparison, except that any NaN matches any NaN.
fn assert_bits(actual: f64, expected: &str, ctx: &str) {
    let expected_bits = hex_u64(expected);
    if f64::from_bits(expected_bits).is_nan() {
        assert!(actual.is_nan(), "{ctx}: expected NaN, got {actual:e}");
    } else {
        assert_eq!(actual.to_bits(), expected_bits, "{ctx}: got {actual:e}");
    }
}

fn vec3(components: &[String; 3]) -> Vec3 {
    let [x, y, z] = components.each_ref().map(|c| f64::from_bits(hex_u64(c)));
    Vec3::new(x, y, z)
}

impl Case {
    fn a(&self) -> Vec3 {
        vec3(&self.a)
    }

    fn b(&self) -> Vec3 {
        vec3(self.b.as_ref().expect("case has `b`"))
    }

    fn s(&self) -> f64 {
        f64::from_bits(hex_u64(self.s.as_deref().expect("case has `s`")))
    }

    fn angle(&self) -> f32 {
        f32::from_bits(hex_u32(self.angle.as_deref().expect("case has `angle`")))
    }

    fn run(&self) -> Outcome {
        use Outcome::{Scalar, Text, Vector};

        let a = self.a();
        match self.op.as_str() {
            "new" => Vector(Some(a)),
            "canonical" => Vector(Some(Vec3::canonical(a.x, a.y, a.z))),
            "add" => Vector(Some(a + self.b())),
            "vector_to" => Vector(Some(self.b() - a)),
            "scale" => Vector(Some(a * self.s())),
            "dot" => Scalar(a.dot(self.b())),
            "cross" => Vector(Some(a.cross(self.b()))),
            "distance" => Scalar(a.distance(self.b())),
            "distance_sqr" | "distance_sqr_xyz" => Scalar(a.distance_sqr(self.b())),
            "lerp" | "interpolate_to" => Vector(Some(a.lerp(self.b(), self.s()))),
            "length" => Scalar(a.length()),
            "normalize" => Vector(Some(a.normalize())),
            "clip_x" => Vector(a.clip_x(self.b(), self.s())),
            "clip_y" => Vector(a.clip_y(self.b(), self.s())),
            "clip_z" => Vector(a.clip_z(self.b(), self.s())),
            "rotate_x" => Vector(Some(a.rotate_x(self.angle()))),
            "rotate_y" => Vector(Some(a.rotate_y(self.angle()))),
            "rotate_z" => Vector(Some(a.rotate_z(self.angle()))),
            "to_string" => Text(a.to_string()),
            other => panic!("fixture op `{other}` has no Rust mapping"),
        }
    }

    fn check(&self, index: usize) {
        let ctx = format!(
            "case #{index} `{}` a={:?} b={:?} s={:?} angle={:?}",
            self.op, self.a, self.b, self.s, self.angle
        );

        match self.run() {
            Outcome::Text(actual) => {
                let expected = self.text.as_deref().expect("text case has `text`");
                assert_eq!(actual, expected, "{ctx}");
            }
            Outcome::Scalar(actual) => {
                let expected = self.expect.as_deref().expect("scalar case has `expect`");
                assert_eq!(expected.len(), 1, "{ctx}: scalar expectation");
                assert_bits(actual, &expected[0], &ctx);
            }
            Outcome::Vector(actual) => match (actual, self.expect.as_deref()) {
                (None, None) => {}
                (Some(v), Some(expected)) => {
                    assert_eq!(expected.len(), 3, "{ctx}: vector expectation");
                    for (axis, (got, want)) in ["x", "y", "z"]
                        .into_iter()
                        .zip([v.x, v.y, v.z].into_iter().zip(expected))
                    {
                        assert_bits(got, want, &format!("{ctx}: {axis}"));
                    }
                }
                (actual, expected) => {
                    panic!("{ctx}: expected {expected:?}, got {actual:?}");
                }
            },
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

const CONSTRUCTORS: &[&str] = &["new", "canonical"];
const ARITHMETIC: &[&str] = &[
    "add",
    "vector_to",
    "scale",
    "dot",
    "cross",
    "distance",
    "distance_sqr",
    "distance_sqr_xyz",
    "lerp",
    "interpolate_to",
];
const LENGTH: &[&str] = &["length", "normalize"];
const CLIP: &[&str] = &["clip_x", "clip_y", "clip_z"];
const ROTATION: &[&str] = &["rotate_x", "rotate_y", "rotate_z"];
const DISPLAY: &[&str] = &["to_string"];

#[test]
fn constructors() {
    check_ops(CONSTRUCTORS);
}

#[test]
fn arithmetic() {
    check_ops(ARITHMETIC);
}

#[test]
fn length_and_normalize() {
    check_ops(LENGTH);
}

#[test]
fn clipping() {
    check_ops(CLIP);
}

#[test]
fn rotation() {
    check_ops(ROTATION);
}

#[test]
fn display() {
    check_ops(DISPLAY);
}

/// Guards against the fixture gaining an op that no test above exercises.
#[test]
fn every_fixture_op_is_tested() {
    let tested: BTreeSet<&str> = [CONSTRUCTORS, ARITHMETIC, LENGTH, CLIP, ROTATION, DISPLAY]
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
