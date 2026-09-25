//! Pure numeric primitives, bit-exact.
//!
//! Nothing in this crate may use libm where a lookup table is specified,
//! and `glam` types must never appear in bit-exact code paths.

pub mod aabb;
pub mod direction;
pub mod facing;
pub mod mth;
pub mod pos;
pub mod random;
pub mod vec3;
pub mod weighed_random;
