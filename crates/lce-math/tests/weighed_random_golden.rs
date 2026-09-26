//! Bit-exact test vectors for [`lce_math::weighed_random`].
//!
//! Each case in `fixtures/weighed_random_golden.json` lists item weights `w`.
//! `total_weight` cases expect their sum. Selection cases seed a fresh
//! generator with `seed` and select once per expected value, in order, from
//! the same generator; each value is the chosen index, or `-1` for `None`.
//! `t`, when present, is the total passed to [`random_item_with_total`], and
//! `after` is the next [`JavaRandom::next_int_unbounded`] once the selections
//! are done, so a selection consuming too much or too little shows up.
//!
//! All values are raw hex `i32` bit patterns.
//!
//! [`random_item_with_total`]: weighed_random::random_item_with_total

use std::collections::BTreeSet;
use std::sync::LazyLock;

use lce_math::random::JavaRandom;
use lce_math::weighed_random::{self, WeighedRandomItem};
use serde::Deserialize;

const FIXTURE: &str = include_str!("fixtures/weighed_random_golden.json");

#[derive(Deserialize)]
struct Fixture {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    op: String,
    seed: Option<String>,
    w: Vec<String>,
    t: Option<String>,
    expect: Vec<String>,
    after: Option<String>,
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

fn int(v: i32) -> String {
    format!("{:#010x}", v as u32)
}

struct Weight(i32);

impl WeighedRandomItem for Weight {
    fn random_weight(&self) -> i32 {
        self.0
    }
}

/// Position of `chosen` in `items` by identity, or `-1` for `None`.
fn index_of(items: &[Weight], chosen: Option<&Weight>) -> i32 {
    chosen.map_or(-1, |c| {
        let i = items
            .iter()
            .position(|item| std::ptr::eq(item, c))
            .expect("selection is one of the items");
        i32::try_from(i).expect("index fits i32")
    })
}

impl Case {
    fn items(&self) -> Vec<Weight> {
        self.w.iter().map(|s| Weight(hex_i32(s))).collect()
    }

    fn check(&self, index: usize) {
        let ctx = format!(
            "case #{index} `{}` seed={:?} w={:?} t={:?}",
            self.op, self.seed, self.w, self.t
        );
        let items = self.items();
        if self.op == "total_weight" {
            let got = [int(weighed_random::total_weight(&items))];
            assert_eq!(got[..], self.expect[..], "{ctx}");
            return;
        }
        let seed = self.seed.as_deref().expect("selection case has a seed");
        let mut random = JavaRandom::new(hex_u64(seed) as i64);
        assert!(!self.expect.is_empty(), "{ctx}: no expected values");
        for (i, expected) in self.expect.iter().enumerate() {
            let chosen = match self.op.as_str() {
                "random_item" => weighed_random::random_item(&mut random, &items),
                "random_item_with_total" => {
                    let total = hex_i32(self.t.as_deref().expect("case has a total"));
                    weighed_random::random_item_with_total(&mut random, &items, total)
                }
                other => panic!("fixture op `{other}` has no Rust mapping"),
            };
            assert_eq!(
                int(index_of(&items, chosen)),
                *expected,
                "{ctx}: selection #{i}"
            );
        }
        let after = self.after.as_deref().expect("selection case has `after`");
        assert_eq!(
            int(random.next_int_unbounded()),
            after,
            "{ctx}: draw after selections"
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

const TOTALS: &[&str] = &["total_weight"];
const SELECTION: &[&str] = &["random_item"];
const SELECTION_WITH_TOTAL: &[&str] = &["random_item_with_total"];

#[test]
fn total_weights() {
    check_ops(TOTALS);
}

#[test]
fn selections() {
    check_ops(SELECTION);
}

#[test]
fn selections_with_a_given_total() {
    check_ops(SELECTION_WITH_TOTAL);
}

/// Guards against the fixture gaining an op that no test above exercises.
#[test]
fn every_fixture_op_is_tested() {
    let tested: BTreeSet<&str> = [TOTALS, SELECTION, SELECTION_WITH_TOTAL]
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
