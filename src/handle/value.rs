// Engine-independent entity identity and its lossless wire representation.
use serde::{Deserialize, Serialize};
use std::fmt;

/// A session-local, lossless Bevy entity handle.
///
/// Handles are valid only for the session that produced them. The two 32-bit fields avoid the
/// precision loss that a JSON number can suffer when consumed by JavaScript.
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
#[serde(deny_unknown_fields)]
pub struct Handle {
    /// Raw Bevy entity index bits.
    pub index: u32,
    /// Raw Bevy entity generation bits.
    pub generation: u32,
}

impl Handle {
    /// Creates a handle from raw entity index and generation bits.
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }
}

impl fmt::Display for Handle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.index, self.generation)
    }
}

/// Failure while reconstructing or resolving a session-local [`Handle`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// The raw entity index bits cannot represent a Bevy entity.
    InvalidBits(Handle),
    /// The encoded entity and generation are not live in the inspected World.
    NotLive(Handle),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBits(handle) => write!(formatter, "invalid entity handle {handle}"),
            Self::NotLive(handle) => write!(formatter, "entity handle {handle} is not live"),
        }
    }
}

impl std::error::Error for Error {}
