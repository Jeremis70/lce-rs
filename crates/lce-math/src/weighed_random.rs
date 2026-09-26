//! Weighted random selection — selection order is observable.
//!
//! One bounded draw picks a point in `0..total`, and the items are walked in
//! order, each claiming the next `weight` values, until the point is passed.
//! Which item is chosen therefore depends on the order of the items as well
//! as on their weights, and exactly one [`JavaRandom::next_int`] is consumed
//! per selection.
//!
//! All weight arithmetic is wrapping `i32` arithmetic. Weights are not
//! required to be positive: a zero weight is never chosen, and negative
//! weights take part in the walk like any other.

use crate::random::JavaRandom;

/// Something that can be chosen by weight.
///
/// The weight is expected to stay the same for as long as the item is part of
/// a selection.
pub trait WeighedRandomItem {
    /// The relative likelihood of this item being chosen.
    fn random_weight(&self) -> i32;
}

impl<T: WeighedRandomItem + ?Sized> WeighedRandomItem for &T {
    fn random_weight(&self) -> i32 {
        (**self).random_weight()
    }
}

impl<T: WeighedRandomItem + ?Sized> WeighedRandomItem for Box<T> {
    fn random_weight(&self) -> i32 {
        (**self).random_weight()
    }
}

/// The sum of the weights of `items`, wrapping on overflow.
#[must_use]
pub fn total_weight<T: WeighedRandomItem>(items: &[T]) -> i32 {
    items.iter().fold(0, |total: i32, item| {
        total.wrapping_add(item.random_weight())
    })
}

/// Picks one of `items`, each with probability `weight / total_weight`.
///
/// Draws `selection = random.next_int(total_weight)`, then walks `items` in
/// order, subtracting each weight from `selection`, and returns the first item
/// after which `selection` is negative.
///
/// `total_weight` is normally [`total_weight(items)`](total_weight), and
/// passing it in lets a caller that selects repeatedly compute it once. Any
/// other positive value is used as given: if it exceeds the real total, the
/// walk can run off the end and the result is `None`; if it is smaller,
/// later items are never reached.
///
/// A `total_weight` of zero or less returns `None` without drawing, so an
/// empty or all-zero `items` never advances `random`.
pub fn random_item_with_total<'a, T: WeighedRandomItem>(
    random: &mut JavaRandom,
    items: &'a [T],
    total_weight: i32,
) -> Option<&'a T> {
    if total_weight <= 0 {
        return None;
    }
    let mut selection = random.next_int(total_weight);
    items.iter().find(|item| {
        selection = selection.wrapping_sub(item.random_weight());
        selection < 0
    })
}

/// Picks one of `items`, each with probability `weight / total`, where
/// `total` is [`total_weight(items)`](total_weight).
///
/// Equivalent to [`random_item_with_total`] with that total: `None` without
/// drawing if the total is zero or less, which includes an empty `items`.
/// Otherwise, with non-negative weights, always `Some`.
///
/// ```
/// use lce_math::random::JavaRandom;
/// use lce_math::weighed_random::{WeighedRandomItem, random_item};
///
/// struct Loot(&'static str, i32);
///
/// impl WeighedRandomItem for Loot {
///     fn random_weight(&self) -> i32 {
///         self.1
///     }
/// }
///
/// let table = [Loot("common", 9), Loot("never", 0), Loot("rare", 1)];
/// let mut random = JavaRandom::new(42);
/// let picked = random_item(&mut random, &table).map(|loot| loot.0);
/// assert!(matches!(picked, Some("common" | "rare")));
/// assert!(random_item::<Loot>(&mut random, &[]).is_none());
/// ```
pub fn random_item<'a, T: WeighedRandomItem>(
    random: &mut JavaRandom,
    items: &'a [T],
) -> Option<&'a T> {
    random_item_with_total(random, items, total_weight(items))
}
