//! Shared constants, byte-level IO, and **every cross-crate trait**.
//!
//! Traits live here so that `lce-world` and `lce-entity` can implement interfaces
//! they do not own, which is what keeps the crate graph acyclic.

pub mod binary_heap;
pub mod class;
pub mod compression;
pub mod data_input;
pub mod data_output;
pub mod error;
pub mod file_header;
pub mod hash;
pub mod shared_constants;
pub mod traits;
