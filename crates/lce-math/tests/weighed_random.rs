//! Behavioural unit tests for [`lce_math::weighed_random`].
//! Bit-exact test vectors live in `weighed_random_golden.rs`.

use lce_math::random::JavaRandom;
use lce_math::weighed_random::{
    WeighedRandomItem, random_item, random_item_with_total, total_weight,
};

#[derive(Debug, PartialEq)]
struct Entry {
    name: &'static str,
    weight: i32,
}

impl WeighedRandomItem for Entry {
    fn random_weight(&self) -> i32 {
        self.weight
    }
}

const fn entry(name: &'static str, weight: i32) -> Entry {
    Entry { name, weight }
}

fn names(items: &[Entry], random: &mut JavaRandom, n: usize) -> Vec<&'static str> {
    (0..n)
        .map(|_| random_item(random, items).map_or("-", |e| e.name))
        .collect()
}

#[test]
fn total_weight_sums_in_wrapping_arithmetic() {
    assert_eq!(total_weight::<Entry>(&[]), 0);
    assert_eq!(
        total_weight(&[entry("a", 10), entry("b", 5), entry("c", 1)]),
        16
    );
    assert_eq!(total_weight(&[entry("a", 4), entry("b", -1)]), 3);
    assert_eq!(
        total_weight(&[entry("a", i32::MAX), entry("b", 2)]),
        i32::MIN + 1
    );
}

#[test]
fn a_selection_is_one_bounded_draw_mapped_through_the_weights_in_order() {
    let items = [entry("a", 1), entry("b", 3), entry("c", 0), entry("d", 2)];
    let mut random = JavaRandom::new(42);
    for _ in 0..200 {
        let mut copy = random.clone();
        let expected = match copy.next_int(6) {
            0 => "a",
            1..=3 => "b",
            _ => "d",
        };
        assert_eq!(
            random_item(&mut random, &items).map(|e| e.name),
            Some(expected)
        );
        assert_eq!(random, copy);
    }
}

#[test]
fn item_order_changes_the_outcome() {
    let forward = [entry("a", 1), entry("b", 1)];
    let backward = [entry("b", 1), entry("a", 1)];
    let a = names(&forward, &mut JavaRandom::new(7), 32);
    let b = names(&backward, &mut JavaRandom::new(7), 32);
    let flipped: Vec<_> = a
        .iter()
        .map(|&n| if n == "a" { "b" } else { "a" })
        .collect();
    assert_eq!(b, flipped);
}

#[test]
fn frequencies_follow_the_weights() {
    let items = [entry("a", 1), entry("b", 2), entry("c", 7)];
    let mut random = JavaRandom::new(2024);
    let picks = names(&items, &mut random, 100_000);
    for (name, share) in [("a", 0.1), ("b", 0.2), ("c", 0.7)] {
        let seen = picks.iter().filter(|&&n| n == name).count() as f64 / 100_000.0;
        assert!((seen - share).abs() < 0.01, "{name}: {seen}");
    }
}

#[test]
fn zero_weights_are_never_chosen() {
    let items = [
        entry("zero", 0),
        entry("a", 3),
        entry("gap", 0),
        entry("b", 1),
        entry("end", 0),
    ];
    let picks = names(&items, &mut JavaRandom::new(3), 2000);
    assert!(picks.iter().all(|&n| n == "a" || n == "b"));
}

#[test]
fn a_single_positive_item_is_always_chosen() {
    let items = [entry("only", 5)];
    assert_eq!(names(&items, &mut JavaRandom::new(1), 50), ["only"; 50]);
}

#[test]
fn no_positive_total_gives_none_without_drawing() {
    let mut random = JavaRandom::new(9);
    let before = random.clone();
    assert_eq!(random_item::<Entry>(&mut random, &[]), None);
    assert_eq!(
        random_item(&mut random, &[entry("a", 0), entry("b", 0)]),
        None
    );
    assert_eq!(random_item(&mut random, &[entry("a", -3)]), None);
    // A total that wraps negative counts as non-positive too.
    assert_eq!(
        random_item(&mut random, &[entry("a", i32::MAX), entry("b", 1)]),
        None
    );
    let items = [entry("a", 5)];
    assert_eq!(random_item_with_total(&mut random, &items, 0), None);
    assert_eq!(random_item_with_total(&mut random, &items, -1), None);
    assert_eq!(random, before);
}

#[test]
fn a_total_above_the_sum_can_run_off_the_end() {
    let items = [entry("a", 1), entry("b", 1)];
    let mut random = JavaRandom::new(5);
    for _ in 0..200 {
        let mut copy = random.clone();
        let expected = match copy.next_int(4) {
            0 => Some("a"),
            1 => Some("b"),
            _ => None,
        };
        let chosen = random_item_with_total(&mut random, &items, 4);
        assert_eq!(chosen.map(|e| e.name), expected);
        assert_eq!(random, copy);
    }
}

#[test]
fn a_total_below_the_sum_never_reaches_later_items() {
    let items = [entry("a", 2), entry("b", 2), entry("c", 2)];
    let mut random = JavaRandom::new(6);
    for _ in 0..500 {
        let chosen = random_item_with_total(&mut random, &items, 4).map(|e| e.name);
        assert!(matches!(chosen, Some("a" | "b")), "{chosen:?}");
    }
}

#[test]
fn negative_weights_take_part_in_the_walk() {
    // Every selection is below 7, so it goes negative at the first item.
    let items = [entry("a", 10), entry("b", -3)];
    assert_eq!(names(&items, &mut JavaRandom::new(4), 20), ["a"; 20]);

    // -1 then -i32::MAX lifts any selection past i32::MAX, which wraps
    // negative at the second item.
    let items = [
        entry("a", -1),
        entry("b", -i32::MAX),
        entry("c", i32::MAX),
        entry("d", i32::MAX),
    ];
    assert_eq!(total_weight(&items), i32::MAX - 1);
    assert_eq!(names(&items, &mut JavaRandom::new(4), 20), ["b"; 20]);
}

#[test]
fn works_with_references_and_boxed_trait_objects() {
    let owned = [entry("a", 1), entry("b", 9)];
    let refs: Vec<&Entry> = owned.iter().collect();
    let boxed: Vec<Box<dyn WeighedRandomItem>> =
        vec![Box::new(entry("a", 1)), Box::new(entry("b", 9))];
    assert_eq!(total_weight(&refs), 10);
    assert_eq!(total_weight(&boxed), 10);

    let mut r1 = JavaRandom::new(8);
    let mut r2 = r1.clone();
    let mut r3 = r1.clone();
    for _ in 0..50 {
        let a = random_item(&mut r1, &owned).map(|e| e.name);
        let b = random_item(&mut r2, &refs).map(|e| e.name);
        let c = random_item(&mut r3, &boxed).map(|e| e.random_weight());
        assert_eq!(a, b);
        assert_eq!(c, a.map(|n| if n == "a" { 1 } else { 9 }));
    }
}
