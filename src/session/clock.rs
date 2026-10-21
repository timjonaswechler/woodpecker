//! Controlled Bevy time advances only inside an explicit application tick.
use crate::command::tick::Clock;
use bevy::{prelude::*, time::TimeUpdateStrategy};
use std::time::Duration;

pub(super) fn validate(app: &App, clock: &Clock) {
    assert!(clock.is_valid(), "invalid tick clock");
    assert!(
        clock.duration().is_none() || app.is_plugin_added::<bevy::time::TimePlugin>(),
        "fixed Woodpecker tick clock requires Bevy TimePlugin before session::Plugin"
    );
}

pub(super) fn before_tick(world: &mut World, clock: &Clock) {
    let Some(duration) = clock.duration() else {
        return;
    };
    // TimePlugin normally discards its first delta. Establish only its origin,
    // without running a schedule or advancing elapsed time, so tick 1 counts.
    let mut real = world.resource_mut::<Time<Real>>();
    if real.last_update().is_none() {
        real.update_with_duration(Duration::ZERO);
    }
    // Reapply at the tick boundary: controlled sessions own this policy, while
    // the application retains Time<Virtual>'s pause, speed and delta limit.
    world.insert_resource(TimeUpdateStrategy::ManualDuration(duration));
}
