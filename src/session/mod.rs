//! Controlled sessions and their optional-in-process Bevy adapter.
mod clock;
mod input;
mod inspect;
mod plugin;
mod screenshot;
pub mod startup;
mod throughput;
mod window;

include!("host.rs");

pub use input::pointer::ID as VIRTUAL_POINTER_ID;
pub use plugin::{Plugin, WarpProgress, tracing_error_layer};
pub use throughput::ThroughputPlugin;
