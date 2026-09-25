//! Bit-exact test vectors for [`Aabb`].
//!
//! Every `f64` in `fixtures/aabb_golden.json` is stored as its raw bit pattern
//! in hex, and every finite or infinite result, including the sign of zero, must
//! match bit-for-bit. As in `vec3_golden.rs`, a NaN result only has to be NaN.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use lce_math::aabb::{Aabb, ClipHit};
use lce_math::facing::Facing;
use lce_math::vec3::Vec3;
use serde::Deserialize;

const FIXTURE: &str = include_str!("fixtures/aabb_golden.json");

#[derive(Deserialize)]
struct Fixture {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    op: String,
    #[serde(rename = "box")]
    aabb: [String; 6],
    other: Option<[String; 6]>,
    p: Option<[String; 3]>,
    q: Option<[String; 3]>,
    s: Option<String>,
    /// Expected box, scalar, or hit position (`null` for a miss).
    expect: Option<Vec<String>>,
    /// Expected boolean result.
    flag: Option<bool>,
    /// Face id of the expected hit.
    face: Option<u8>,
    /// Expected `Display` output.
    text: Option<String>,
}

enum Outcome {
    Box(Aabb),
    Scalar(f64),
    Flag(bool),
    Hit(Option<ClipHit>),
    Text(String),
}

static CASES: LazyLock<Vec<Case>> = LazyLock::new(|| {
    serde_json::from_str::<Fixture>(FIXTURE)
        .expect("fixture is valid JSON")
        .cases
});

fn f64_hex(s: &str) -> f64 {
    let digits = s.strip_prefix("0x").expect("hex literal has 0x prefix");
    f64::from_bits(u64::from_str_radix(digits, 16).expect("valid 64-bit hex"))
}

/// Bit-exact comparison, except that any NaN matches any NaN.
fn assert_bits(actual: f64, expected: &str, ctx: &str) {
    let expected = f64_hex(expected);
    if expected.is_nan() {
        assert!(actual.is_nan(), "{ctx}: expected NaN, got {actual:e}");
    } else {
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "{ctx}: got {actual:e}, want {expected:e}"
        );
    }
}

fn assert_all_bits(actual: &[f64], expected: &[String], names: &[&str], ctx: &str) {
    assert_eq!(expected.len(), actual.len(), "{ctx}: expectation arity");
    for ((got, want), name) in actual.iter().zip(expected).zip(names) {
        assert_bits(*got, want, &format!("{ctx}: {name}"));
    }
}

fn aabb(c: &[String; 6]) -> Aabb {
    let [x0, y0, z0, x1, y1, z1] = c.each_ref().map(|s| f64_hex(s));
    Aabb::new(x0, y0, z0, x1, y1, z1)
}

fn vec3(c: &[String; 3]) -> Vec3 {
    let [x, y, z] = c.each_ref().map(|s| f64_hex(s));
    Vec3::new(x, y, z)
}

fn facing(id: u8) -> Facing {
    [
        Facing::Down,
        Facing::Up,
        Facing::North,
        Facing::South,
        Facing::West,
        Facing::East,
    ]
    .into_iter()
    .find(|f| *f as u8 == id)
    .unwrap_or_else(|| panic!("face id {id} out of range"))
}

impl Case {
    fn other(&self) -> Aabb {
        aabb(self.other.as_ref().expect("case has `other`"))
    }

    fn p(&self) -> Vec3 {
        vec3(self.p.as_ref().expect("case has `p`"))
    }

    fn q(&self) -> Vec3 {
        vec3(self.q.as_ref().expect("case has `q`"))
    }

    fn s(&self) -> f64 {
        f64_hex(self.s.as_deref().expect("case has `s`"))
    }

    fn run(&self) -> Outcome {
        use Outcome::{Box, Flag, Hit, Scalar, Text};

        let a = aabb(&self.aabb);
        match self.op.as_str() {
            "new" => Box(a),
            "expand" => {
                let d = self.p();
                Box(a.expand(d.x, d.y, d.z))
            }
            "grow" => {
                let d = self.p();
                Box(a.grow(d.x, d.y, d.z))
            }
            "shrink" => {
                let d = self.p();
                Box(a.shrink(d.x, d.y, d.z))
            }
            "move" | "clone_move" => {
                let d = self.p();
                Box(a.moved(d.x, d.y, d.z))
            }
            "size" => Scalar(a.size()),
            "distance" => Scalar(a.distance(self.p())),
            "clip_x_collide" => Scalar(a.clip_x_collide(self.other(), self.s())),
            "clip_y_collide" => Scalar(a.clip_y_collide(self.other(), self.s())),
            "clip_z_collide" => Scalar(a.clip_z_collide(self.other(), self.s())),
            "intersects" | "intersects_xyz" => Flag(a.intersects(self.other())),
            "intersects_inner" => Flag(a.intersects_inner(self.other())),
            "contains" => Flag(a.contains(self.p())),
            "contains_including_lower_bound" => Flag(a.contains_including_lower_bound(self.p())),
            "clip" => Hit(a.clip(self.p(), self.q())),
            "to_string" => Text(a.to_string()),
            other => panic!("fixture op `{other}` has no Rust mapping"),
        }
    }

    fn check(&self, index: usize) {
        let ctx = format!(
            "case #{index} `{}` box={:?} other={:?} p={:?} q={:?} s={:?}",
            self.op, self.aabb, self.other, self.p, self.q, self.s
        );

        match self.run() {
            Outcome::Text(actual) => {
                let expected = self.text.as_deref().expect("text case has `text`");
                assert_eq!(actual, expected, "{ctx}");
            }
            Outcome::Flag(actual) => {
                let expected = self.flag.expect("flag case has `flag`");
                assert_eq!(actual, expected, "{ctx}");
            }
            Outcome::Scalar(actual) => {
                let expected = self.expect.as_deref().expect("scalar case has `expect`");
                assert_all_bits(&[actual], expected, &["value"], &ctx);
            }
            Outcome::Box(b) => {
                let expected = self.expect.as_deref().expect("box case has `expect`");
                assert_all_bits(
                    &[b.x0, b.y0, b.z0, b.x1, b.y1, b.z1],
                    expected,
                    &["x0", "y0", "z0", "x1", "y1", "z1"],
                    &ctx,
                );
            }
            Outcome::Hit(actual) => match (actual, self.expect.as_deref()) {
                (None, None) => {}
                (Some(hit), Some(expected)) => {
                    let face = facing(self.face.expect("hit case has `face`"));
                    assert_eq!(hit.face, face, "{ctx}: face");
                    let p = hit.pos;
                    assert_all_bits(&[p.x, p.y, p.z], expected, &["x", "y", "z"], &ctx);
                }
                (actual, expected) => {
                    panic!(
                        "{ctx}: expected {expected:?} (face {:?}), got {actual:?}",
                        self.face
                    );
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

const CONSTRUCTORS: &[&str] = &["new"];
const TRANSFORMS: &[&str] = &["expand", "grow", "shrink", "move", "clone_move", "size"];
const COLLIDE: &[&str] = &["clip_x_collide", "clip_y_collide", "clip_z_collide"];
const INTERSECTS: &[&str] = &["intersects", "intersects_xyz", "intersects_inner"];
const CONTAINS: &[&str] = &["contains", "contains_including_lower_bound"];
const CLIP: &[&str] = &["clip"];
const DISTANCE: &[&str] = &["distance"];
const DISPLAY: &[&str] = &["to_string"];

#[test]
fn constructors() {
    check_ops(CONSTRUCTORS);
}

#[test]
fn transforms() {
    check_ops(TRANSFORMS);
}

#[test]
fn collision_clipping() {
    check_ops(COLLIDE);
}

#[test]
fn intersection() {
    check_ops(INTERSECTS);
}

#[test]
fn containment() {
    check_ops(CONTAINS);
}

#[test]
fn ray_clipping() {
    check_ops(CLIP);
}

#[test]
fn point_distance() {
    check_ops(DISTANCE);
}

#[test]
fn display() {
    check_ops(DISPLAY);
}

/// Guards against the fixture gaining an op that no test above exercises.
#[test]
fn every_fixture_op_is_tested() {
    let tested: BTreeSet<&str> = [
        CONSTRUCTORS,
        TRANSFORMS,
        COLLIDE,
        INTERSECTS,
        CONTAINS,
        CLIP,
        DISTANCE,
        DISPLAY,
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
