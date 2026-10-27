//! A known application-owned policy, independent of Woodpecker's warp pace.
use bevy::{prelude::*, time::TimeUpdateStrategy};
use std::time::Duration;

pub(super) fn install(app: &mut App, mode: &str) {
    assert!(
        matches!(mode, "steady" | "double" | "paused"),
        "invalid --woodpecker-clock: {mode}; expected steady, double or paused"
    );
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        50,
    )));
    // Establish only the clock origin: no schedule, elapsed time or tick advances.
    app.world_mut()
        .resource_mut::<Time<Real>>()
        .update_with_duration(Duration::ZERO);
    let mut time = app.world_mut().resource_mut::<Time<Virtual>>();
    time.set_relative_speed(if mode == "double" { 2.0 } else { 1.0 });
    if mode == "paused" {
        time.pause();
    } else {
        time.unpause();
    }
}
