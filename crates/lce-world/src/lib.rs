//! The world model: tiles, chunks, levels, lighting, worldgen, save IO.
//!
//! **This crate must never depend on `lce-entity`.** `Level` stores opaque
//! entity handles and receives entity access as a tick parameter.

pub mod chunk;
pub mod level;
pub mod light;
pub mod region;
pub mod save;
pub mod storage;
pub mod tick;
pub mod tile;
pub mod tile_entity;
pub mod tile_registry;
pub mod worldgen;
