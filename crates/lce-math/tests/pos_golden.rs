//! Bit-exact test vectors for [`Pos`], [`TilePos`] and [`ChunkPos`].
//!
//! Inputs in `fixtures/pos_golden.json` are plain decimal integers, or raw
//! `f64` bit patterns in hex. Every result is stored as raw hex: `i32`/`i64`
//! as two's-complement bits, `f64`/`f32` as IEEE-754 bits. Integer and finite
//! float results must match bit-for-bit; as in `vec3_golden.rs`, a NaN result
//! only has to be NaN.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use lce_math::pos::{ChunkPos, Pos, TilePos};
use lce_math::vec3::Vec3;
use serde::Deserialize;

const FIXTURE: &str = include_str!("fixtures/pos_golden.json");

#[derive(Deserialize)]
struct Fixture {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    op: String,
    /// Subject `Pos`/`TilePos`.
    p: Option<[i32; 3]>,
    /// Second position, or an `(dx, dy, dz)` offset.
    o: Option<[i32; 3]>,
    /// Subject `ChunkPos` as `[x, z]`.
    c: Option<[i32; 2]>,
    /// Step count or height.
    n: Option<i32>,
    /// `f64` operands as hex bit patterns.
    v: Option<Vec<String>>,
    expect: Option<Vec<String>>,
    /// Expected `Display` output.
    text: Option<String>,
}

enum Outcome {
    Ints(Vec<i32>),
    Long(i64),
    Double(f64),
    Float(f32),
    Text(String),
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

impl Case {
    fn pos(&self) -> Pos {
        let [x, y, z] = self.p.expect("case has `p`");
        Pos::new(x, y, z)
    }

    fn other(&self) -> Pos {
        let [x, y, z] = self.o.expect("case has `o`");
        Pos::new(x, y, z)
    }

    fn chunk(&self) -> ChunkPos {
        let [x, z] = self.c.expect("case has `c`");
        ChunkPos::new(x, z)
    }

    fn n(&self) -> i32 {
        self.n.expect("case has `n`")
    }

    fn doubles(&self) -> Vec<f64> {
        self.v
            .as_deref()
            .expect("case has `v`")
            .iter()
            .map(|s| f64::from_bits(hex_u64(s)))
            .collect()
    }

    fn run(&self) -> Outcome {
        use Outcome::{Double, Float, Ints, Long, Text};

        let pos = |p: Pos| Ints(vec![p.x, p.y, p.z]);
        let tile = |t: TilePos| Ints(vec![t.x, t.y, t.z]);
        let with = |f: fn(&mut Pos)| {
            let mut p = self.pos();
            f(&mut p);
            pos(p)
        };

        match self.op.as_str() {
            "pos_new" => pos(self.pos()),
            "pos_hash" => Ints(vec![self.pos().hash_code()]),
            "pos_above" => pos(self.pos().above()),
            "pos_below" => pos(self.pos().below()),
            "pos_north" => pos(self.pos().north()),
            "pos_south" => pos(self.pos().south()),
            "pos_west" => pos(self.pos().west()),
            "pos_east" => pos(self.pos().east()),
            "pos_move_up" => with(|p| *p = p.above()),
            "pos_move_down" => with(|p| *p = p.below()),
            "pos_move_north" => with(|p| *p = p.north()),
            "pos_move_south" => with(|p| *p = p.south()),
            "pos_move_west" => with(|p| *p = p.west()),
            "pos_move_east" => with(|p| *p = p.east()),
            "pos_above_n" | "pos_move_up_n" | "pos_move_y" => pos(self.pos().above_n(self.n())),
            "pos_below_n" | "pos_move_down_n" => pos(self.pos().below_n(self.n())),
            "pos_north_n" | "pos_move_north_n" => pos(self.pos().north_n(self.n())),
            "pos_south_n" | "pos_move_south_n" | "pos_move_z" => pos(self.pos().south_n(self.n())),
            "pos_move_west_n" => pos(self.pos().west_n(self.n())),
            "pos_east_n" | "pos_move_east_n" | "pos_move_x" => pos(self.pos().east_n(self.n())),
            "pos_offset" | "pos_move" => {
                let d = self.other();
                pos(self.pos().offset(d.x, d.y, d.z))
            }
            "pos_move_pos" => {
                let mut p = self.pos();
                p += self.other();
                pos(p)
            }
            "pos_compare" => Ints(vec![self.pos().compare_to(self.other())]),
            "pos_equals" => Ints(vec![i32::from(self.pos() == self.other())]),
            "pos_dist" => Double(self.pos().dist_to(self.other())),
            "pos_dist_sqr" => {
                let o = self.other();
                Float(self.pos().dist_sqr(o.x, o.y, o.z))
            }
            "tile_hash" => {
                let [x, y, z] = self.p.expect("case has `p`");
                Ints(vec![TilePos::new(x, y, z).hash_code()])
            }
            "tile_containing" => {
                let [x, y, z] = self.doubles()[..] else {
                    panic!("tile_containing takes three doubles");
                };
                tile(TilePos::containing(Vec3::new(x, y, z)))
            }
            "chunk_long_hash" => {
                let [x, z] = self.c.expect("case has `c`");
                Long(ChunkPos::long_hash(x, z))
            }
            "chunk_key" => Long(self.chunk().to_long()),
            "chunk_hash" => Ints(vec![self.chunk().hash_code()]),
            "chunk_middle" => {
                let c = self.chunk();
                Ints(vec![c.middle_block_x(), c.middle_block_z()])
            }
            "chunk_middle_position" => tile(self.chunk().middle_block_position(self.n())),
            "chunk_distance_to_sqr" | "chunk_distance_to_entity_sqr" => {
                let [px, pz] = self.doubles()[..] else {
                    panic!("distance takes two doubles");
                };
                Double(self.chunk().distance_to_sqr(px, pz))
            }
            "chunk_to_string" => Text(self.chunk().to_string()),
            other => panic!("fixture op `{other}` has no Rust mapping"),
        }
    }

    fn check(&self, index: usize) {
        let ctx = format!(
            "case #{index} `{}` p={:?} o={:?} c={:?} n={:?} v={:?}",
            self.op, self.p, self.o, self.c, self.n, self.v
        );
        let expect = || self.expect.as_deref().expect("case has `expect`");

        match self.run() {
            Outcome::Text(actual) => {
                let expected = self.text.as_deref().expect("text case has `text`");
                assert_eq!(actual, expected, "{ctx}");
            }
            Outcome::Ints(actual) => {
                let expected: Vec<i32> = expect().iter().map(|s| hex_u32(s) as i32).collect();
                assert_eq!(actual, expected, "{ctx}");
            }
            Outcome::Long(actual) => {
                let [expected] = expect() else {
                    panic!("{ctx}: expected one value");
                };
                let expected = hex_u64(expected) as i64;
                assert_eq!(
                    actual, expected,
                    "{ctx}: got {actual:#x}, want {expected:#x}"
                );
            }
            Outcome::Double(actual) => {
                let [expected] = expect() else {
                    panic!("{ctx}: expected one value");
                };
                let expected = f64::from_bits(hex_u64(expected));
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
            Outcome::Float(actual) => {
                let [expected] = expect() else {
                    panic!("{ctx}: expected one value");
                };
                let expected = f32::from_bits(hex_u32(expected));
                assert_eq!(
                    actual.to_bits(),
                    expected.to_bits(),
                    "{ctx}: got {actual:e}, want {expected:e}"
                );
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

const POS_BASICS: &[&str] = &["pos_new", "pos_hash", "pos_equals", "pos_compare"];
const POS_STEPS: &[&str] = &[
    "pos_above",
    "pos_below",
    "pos_north",
    "pos_south",
    "pos_west",
    "pos_east",
    "pos_move_up",
    "pos_move_down",
    "pos_move_north",
    "pos_move_south",
    "pos_move_west",
    "pos_move_east",
];
const POS_STEPS_N: &[&str] = &[
    "pos_above_n",
    "pos_below_n",
    "pos_north_n",
    "pos_south_n",
    "pos_east_n",
    "pos_move_up_n",
    "pos_move_down_n",
    "pos_move_north_n",
    "pos_move_south_n",
    "pos_move_west_n",
    "pos_move_east_n",
    "pos_move_x",
    "pos_move_y",
    "pos_move_z",
];
const POS_OFFSET: &[&str] = &["pos_offset", "pos_move", "pos_move_pos"];
const POS_DISTANCE: &[&str] = &["pos_dist", "pos_dist_sqr"];
const TILE: &[&str] = &["tile_hash", "tile_containing"];
const CHUNK_HASH: &[&str] = &["chunk_long_hash", "chunk_key", "chunk_hash"];
const CHUNK_GEOMETRY: &[&str] = &[
    "chunk_middle",
    "chunk_middle_position",
    "chunk_distance_to_sqr",
    "chunk_distance_to_entity_sqr",
];
const CHUNK_DISPLAY: &[&str] = &["chunk_to_string"];

#[test]
fn pos_basics() {
    check_ops(POS_BASICS);
}

#[test]
fn pos_single_steps() {
    check_ops(POS_STEPS);
}

#[test]
fn pos_multi_steps() {
    check_ops(POS_STEPS_N);
}

#[test]
fn pos_offsets() {
    check_ops(POS_OFFSET);
}

#[test]
fn pos_distances() {
    check_ops(POS_DISTANCE);
}

#[test]
fn tile_pos() {
    check_ops(TILE);
}

#[test]
fn chunk_hashes() {
    check_ops(CHUNK_HASH);
}

#[test]
fn chunk_geometry() {
    check_ops(CHUNK_GEOMETRY);
}

#[test]
fn chunk_display() {
    check_ops(CHUNK_DISPLAY);
}

/// Guards against the fixture gaining an op that no test above exercises.
#[test]
fn every_fixture_op_is_tested() {
    let tested: BTreeSet<&str> = [
        POS_BASICS,
        POS_STEPS,
        POS_STEPS_N,
        POS_OFFSET,
        POS_DISTANCE,
        TILE,
        CHUNK_HASH,
        CHUNK_GEOMETRY,
        CHUNK_DISPLAY,
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
