use serde::{Deserialize, Serialize};

/// Capture the primary rendered window beneath the session's artifact directory.
///
/// An unavailable render surface (for example, an occluded window on Metal)
/// is rejected rather than saved as an unfilled readback image.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capture {
    pub path: String,
}

impl Capture {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub overwritten: bool,
}

super::request!(Capture, Screenshot, Output);
