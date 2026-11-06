//! A known application-owned policy, independent of Woodpecker's warp pace.
use bevy::{prelude::*, time::TimeUpdateStrategy};
use std::time::Duration;

/// Read-only scalar view of Time<Real>, whose Instant context is not serializable.
/// Updated on extraction, including idle frames, rather than on simulation ticks.
#[derive(Resource, Reflect, Default, PartialEq)]
#[reflect(Resource)]
struct RealSnapshot {
    elapsed: f64,
    delta: f64,
    first_update: Option<f64>,
    last_update: Option<f64>,
}

pub(super) fn install_snapshot(app: &mut App) {
    app.init_resource::<RealSnapshot>()
        .register_type::<RealSnapshot>();
    let render = app.sub_app_mut(bevy::render::RenderApp);
    let mut extract = render.take_extract();
    render.set_extract(move |main, render| {
        let time = main.resource::<Time<Real>>();
        let snapshot = RealSnapshot {
            elapsed: time.elapsed_secs_f64(),
            delta: time.delta_secs_f64(),
            first_update: time
                .first_update()
                .map(|instant| instant.duration_since(time.startup()).as_secs_f64()),
            last_update: time
                .last_update()
                .map(|instant| instant.duration_since(time.startup()).as_secs_f64()),
        };
        main.resource_mut::<RealSnapshot>().set_if_neq(snapshot);
        if let Some(extract) = extract.as_mut() {
            extract(main, render);
        }
    });
}

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
