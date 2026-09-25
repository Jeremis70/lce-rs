//! Hand-rolled NBT: big-endian, modified UTF-8, observable tag ordering.
//!
//! Deliberately not `serde`: the byte layout and tag order are part of the format.

pub mod compound;
pub mod io;
pub mod list;
pub mod tag;
