//! The executable: window, input, HUD, audio, main loop.

pub mod audio;
pub mod config;
pub mod hud;
pub mod input;
pub mod menu;
pub mod window;

fn main() {
    tracing::info!("lce-rs {}", env!("CARGO_PKG_VERSION"));
    println!("Hello, world!")
}
