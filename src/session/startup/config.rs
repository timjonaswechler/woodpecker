//! Engine-independent startup configuration shared by host and Bevy adapter.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Config {
    /// Preserve explicit control from the first application tick.
    Manual {},
    /// Run ordinary application ticks until scoped technical readiness is observed.
    Engine {
        timeout_seconds: u32,
        max_ticks: u64,
    },
}

impl Default for Config {
    fn default() -> Self {
        Self::Manual {}
    }
}

impl Config {
    pub(crate) fn enabled(&self) -> bool {
        matches!(self, Self::Engine { .. })
    }
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Manual {} => true,
            Self::Engine {
                timeout_seconds,
                max_ticks,
            } => *timeout_seconds > 0 && *max_ticks > 0,
        }
    }
}
