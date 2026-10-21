//! Playable application and the object models shared with the model zoo.
pub mod app;
mod input;
pub mod level;
mod player;
mod ui;
#[cfg(feature = "automation")]
pub mod woodpecker;

/// Asset paths are independent of the caller's current working directory.
pub const ASSET_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/asset");
