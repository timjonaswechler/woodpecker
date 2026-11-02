//! Production panic framing without the session coordinator or a renderer.
#[path = "capture.rs"]
pub mod capture;
#[path = "failure.rs"]
mod failure;
#[path = "marker.rs"]
mod marker;
pub use failure::{BacktraceStatus, Failure, Location, Origin};
