//! Shared gameplay modules and station models used by the game and model zoo.
pub mod app;
mod commands;
pub mod dev;
pub mod expedition;
mod input;
pub mod items;
pub mod launch;
pub mod level;
mod player;
mod practice;
mod settings;
#[cfg(test)]
mod tests;
mod ui;
#[cfg(feature = "automation")]
pub mod woodpecker;

/// Shared assets remain rooted at this package, independently of the caller's working directory.
pub const ASSET_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/asset");
