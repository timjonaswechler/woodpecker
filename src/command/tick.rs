use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub pace: warp::Pace,
    #[serde(default)]
    pub clock: Clock,
}

/// Bevy time policy, independent of the wall-clock pace used to execute ticks.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Clock {
    /// Keep the application's configured time policy.
    Application {},
    /// Advance Bevy's real clock by this amount for every explicit tick.
    /// Virtual time still respects the application's pause, speed and maximum delta.
    Fixed { seconds_per_tick: f64 },
}

impl Default for Clock {
    fn default() -> Self {
        Self::Application {}
    }
}

impl Clock {
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Application {})
            || self
                .duration()
                .is_some_and(|duration| std::time::Instant::now().checked_add(duration).is_some())
    }

    pub(crate) fn duration(&self) -> Option<std::time::Duration> {
        match self {
            Self::Application {} => None,
            Self::Fixed { seconds_per_tick } => {
                let duration = std::time::Duration::try_from_secs_f64(*seconds_per_tick).ok()?;
                (!duration.is_zero()).then_some(duration)
            }
        }
    }
}

pub mod warp {
    use super::*;

    #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
    pub enum Pace {
        #[default]
        AsFastAsPossible,
        TicksPerSecond {
            target: f32,
        },
    }

    impl Pace {
        pub fn is_valid(&self) -> bool {
            match self {
                Self::AsFastAsPossible => true,
                Self::TicksPerSecond { target } => target.is_finite() && *target > 0.0,
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Start {
        pub ticks: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub pace: Option<Pace>,
    }

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct SetPace {
        pub pace: Pace,
    }

    #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Stop {}

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Completion {
        pub requested_ticks: u64,
        pub executed_ticks: u64,
        pub outcome: Outcome,
    }

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Outcome {
        Completed,
        Stopped,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct PaceChanged {
        pub pace: Pace,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Stopped {
        pub was_running: bool,
    }

    crate::command::request!(Start, Start, Completion);
    crate::command::request!(SetPace, SetPace, PaceChanged);
    crate::command::request!(Stop, Stop, Stopped);
}
