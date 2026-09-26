//! Bit-exact test vectors for [`Facing`] and [`Direction`].
//!
//! `fixtures/facing_golden.json` enumerates every entry of every table these
//! two types expose: there is no sampling here, so a single mismatched or
//! transposed entry is guaranteed to be caught. Every value is a raw `i32`
//! two's-complement hex bit pattern, `-1` included.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use lce_math::direction::Direction;
use lce_math::facing::Facing;
use serde::Deserialize;

const FIXTURE: &str = include_str!("fixtures/facing_golden.json");

#[derive(Deserialize)]
struct Fixture {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    op: String,
    /// Facing id.
    f: Option<u8>,
    /// Direction id.
    d: Option<u8>,
    expect: Vec<String>,
}

static CASES: LazyLock<Vec<Case>> = LazyLock::new(|| {
    serde_json::from_str::<Fixture>(FIXTURE)
        .expect("fixture is valid JSON")
        .cases
});

fn hex_i32(s: &str) -> i32 {
    let digits = s.strip_prefix("0x").expect("hex literal has 0x prefix");
    u32::from_str_radix(digits, 16).expect("valid 32-bit hex") as i32
}

impl Case {
    fn facing(&self) -> Facing {
        Facing::from_id(self.f.expect("case has `f`")).expect("valid facing id")
    }

    fn direction(&self) -> Direction {
        Direction::from_id(self.d.expect("case has `d`")).expect("valid direction id")
    }

    fn expect_ints(&self) -> Vec<i32> {
        self.expect.iter().map(|s| hex_i32(s)).collect()
    }

    fn run(&self) -> Vec<i32> {
        match self.op.as_str() {
            "facing_opposite" => vec![self.facing().opposite().id().into()],
            "facing_step" => {
                let f = self.facing();
                vec![f.step_x(), f.step_y(), f.step_z()]
            }
            "facing_direction" => {
                vec![Direction::from_facing(self.facing()).map_or(-1, |d| i32::from(d.id()))]
            }
            "direction_step" => {
                let d = self.direction();
                vec![d.step_x(), d.step_z()]
            }
            "direction_facing" => vec![self.direction().facing().id().into()],
            "direction_opposite" => vec![self.direction().opposite().id().into()],
            "direction_clockwise" => vec![self.direction().clockwise().id().into()],
            "direction_counter_clockwise" => vec![self.direction().counter_clockwise().id().into()],
            "relative_direction_facing" => {
                vec![self.direction().relative_facing(self.facing()).id().into()]
            }
            other => panic!("fixture op `{other}` has no Rust mapping"),
        }
    }

    fn check(&self, index: usize) {
        let actual = self.run();
        let expected = self.expect_ints();
        assert_eq!(
            actual, expected,
            "case #{index} `{}` f={:?} d={:?}",
            self.op, self.f, self.d
        );
    }
}

fn check_op(op: &str) {
    let mut seen = false;
    for (i, case) in CASES.iter().enumerate() {
        if case.op == op {
            case.check(i);
            seen = true;
        }
    }
    assert!(seen, "fixture has no cases for `{op}`");
}

#[test]
fn facing_opposite_table() {
    check_op("facing_opposite");
}

#[test]
fn facing_step_tables() {
    check_op("facing_step");
}

#[test]
fn facing_to_direction_table() {
    check_op("facing_direction");
}

#[test]
fn direction_step_tables() {
    check_op("direction_step");
}

#[test]
fn direction_to_facing_table() {
    check_op("direction_facing");
}

#[test]
fn direction_opposite_table() {
    check_op("direction_opposite");
}

#[test]
fn direction_clockwise_table() {
    check_op("direction_clockwise");
}

#[test]
fn direction_counter_clockwise_table() {
    check_op("direction_counter_clockwise");
}

#[test]
fn relative_direction_facing_table() {
    check_op("relative_direction_facing");
}

/// Guards against the fixture gaining an op that no test above exercises.
#[test]
fn every_fixture_op_is_tested() {
    const TESTED: &[&str] = &[
        "facing_opposite",
        "facing_step",
        "facing_direction",
        "direction_step",
        "direction_facing",
        "direction_opposite",
        "direction_clockwise",
        "direction_counter_clockwise",
        "relative_direction_facing",
    ];
    let tested: BTreeSet<&str> = TESTED.iter().copied().collect();
    let untested: BTreeSet<&str> = CASES
        .iter()
        .map(|c| c.op.as_str())
        .filter(|op| !tested.contains(op))
        .collect();
    assert!(untested.is_empty(), "untested fixture ops: {untested:?}");
}
