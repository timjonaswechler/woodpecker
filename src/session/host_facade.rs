//! Renderer-free session owner. Wire types and host logic have one source owner.
#[path = "startup/config.rs"]
pub mod startup;

include!("host.rs");
