//! Render-clock diagnostics. Never runs application schedules or updates Bevy time.
mod overlay;

use bevy::{
    prelude::*,
    render::{
        RenderApp,
        camera::ExtractedCamera,
        renderer::{RenderGraph, RenderGraphSystems},
        view::ViewTarget,
    },
    window::PrimaryWindow,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

/// Shared by Inspect, the rendered overlay and the performance evaluator.
#[derive(Resource, Default, Reflect, serde::Serialize)]
#[reflect(Resource)]
pub struct Metrics {
    pub fps: f64,
    pub tps: f64,
    pub simulation_speed: f64,
    pub ticks: u64,
    pub rendered_frames: u64,
    pub sample_seconds: f64,
    pub controlled: bool,
    pub warp_running: bool,
    pub requested_ticks: u64,
    pub executed_ticks: u64,
    pub target_tps: Option<f32>,
    pub overlay: bool,
}

#[derive(Resource, Clone, Default)]
struct Frames(Arc<AtomicU64>);

#[derive(Resource)]
struct Sampler {
    since: Instant,
    ticks: u64,
    frames: u64,
    simulated: Duration,
    log: bool,
}

/// Install after the optional Woodpecker plugin, with an application-owned renderer.
pub fn install(app: &mut App, show: bool, log: bool, controlled: bool) {
    let frames = Frames::default();
    app.insert_resource(frames.clone())
        .insert_resource(Metrics {
            overlay: show,
            controlled,
            ..default()
        })
        .insert_resource(Sampler {
            since: Instant::now(),
            ticks: 0,
            frames: 0,
            simulated: Duration::ZERO,
            log,
        })
        .register_type::<Metrics>()
        .add_systems(Last, count_tick);
    let render = app.sub_app_mut(RenderApp);
    overlay::install(render);
    render
        .insert_resource(frames)
        .add_systems(RenderGraph, count_frame.in_set(RenderGraphSystems::Finish));
    let mut extract = render.take_extract();
    render.set_extract(move |main, render| {
        let updated = sample(main, Instant::now());
        let window = main
            .query_filtered::<Entity, With<PrimaryWindow>>()
            .iter(main)
            .next();
        let size = window
            .and_then(|entity| main.get::<Window>(entity))
            .map_or(UVec2::ZERO, Window::physical_size);
        let target = main
            .query_filtered::<&bevy::camera::RenderTarget, With<crate::player::camera::GameCamera>>(
            )
            .iter(main)
            .next()
            .and_then(|target| target.normalize(window));
        let metrics = main.resource::<Metrics>();
        let mut text = render.resource_mut::<overlay::Text>();
        text.target = target;
        text.size = size;
        let enabled = metrics.overlay;
        if enabled && (updated || !text.enabled) {
            text.lines = lines(metrics);
            text.revision = text.revision.wrapping_add(1);
        }
        text.enabled = enabled;
        if let Some(extract) = extract.as_mut() {
            extract(main, render);
        }
    });
}

fn count_tick(mut metrics: ResMut<Metrics>, keys: Res<ButtonInput<KeyCode>>) {
    metrics.ticks += 1;
    if keys.just_pressed(KeyCode::F8) {
        metrics.overlay = !metrics.overlay;
    }
}

fn count_frame(
    text: Res<overlay::Text>,
    views: Query<(&ViewTarget, &ExtractedCamera)>,
    frames: Res<Frames>,
) {
    if views.iter().any(|(target, camera)| {
        camera.target.as_ref() == text.target.as_ref()
            && target.out_texture().is_some()
            && target.needs_present()
    }) {
        frames.0.fetch_add(1, Ordering::Relaxed);
    }
}

fn sample(world: &mut World, now: Instant) -> bool {
    #[cfg(feature = "automation")]
    let progress = woodpecker::session::Plugin::warp_progress(world);
    let frames = world.resource::<Frames>().0.load(Ordering::Relaxed);
    let simulated = world.resource::<Time<Virtual>>().elapsed();
    world.resource_scope(|world, mut sampler: Mut<Sampler>| {
        let mut metrics = world.resource_mut::<Metrics>();
        #[cfg(feature = "automation")]
        {
            metrics.warp_running = progress.is_some();
            if let Some(progress) = progress {
                metrics.requested_ticks = progress.requested_ticks;
                metrics.executed_ticks = progress.executed_ticks;
                metrics.target_tps = match progress.pace {
                    woodpecker::command::tick::warp::Pace::AsFastAsPossible => None,
                    woodpecker::command::tick::warp::Pace::TicksPerSecond { target } => {
                        Some(target)
                    }
                };
            } else {
                metrics.requested_ticks = 0;
                metrics.executed_ticks = 0;
                metrics.target_tps = None;
            }
        }
        let elapsed = now.duration_since(sampler.since);
        if elapsed >= Duration::from_millis(250) {
            let seconds = elapsed.as_secs_f64();
            metrics.fps = (frames - sampler.frames) as f64 / seconds;
            metrics.tps = (metrics.ticks - sampler.ticks) as f64 / seconds;
            metrics.simulation_speed =
                simulated.saturating_sub(sampler.simulated).as_secs_f64() / seconds;
            metrics.rendered_frames = frames;
            metrics.sample_seconds = seconds;
            sampler.since = now;
            sampler.frames = frames;
            sampler.ticks = metrics.ticks;
            sampler.simulated = simulated;
            if sampler.log {
                eprintln!(
                    "SIGNALSTATION_PERFORMANCE {}",
                    serde_json::to_string(&*metrics).unwrap()
                );
            }
            true
        } else {
            false
        }
    })
}

fn lines(metrics: &Metrics) -> String {
    let status = if !metrics.controlled {
        "LIVE".into()
    } else if !metrics.warp_running {
        "PAUSED".into()
    } else if let Some(target) = metrics.target_tps {
        format!("TARGET {target:.1} TPS")
    } else {
        "MAX THROUGHPUT".into()
    };
    let progress = if metrics.warp_running {
        format!(
            "DONE {:20}\nLEFT {:20}",
            metrics.executed_ticks,
            metrics.requested_ticks - metrics.executed_ticks
        )
    } else {
        format!("TOTAL TICKS {:20}\n", metrics.ticks)
    };
    format!(
        "PERFORMANCE                    F8: TOGGLE\nFPS {:7.1}     TPS {:9.1}\nSIM {:7.2} X REAL TIME\nMODE {status}\n{progress}",
        metrics.fps, metrics.tps, metrics.simulation_speed,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_sampling_counts_real_frames_and_ticks_without_advancing_time() {
        let mut app = App::new();
        let since = Instant::now();
        app.insert_resource(Frames::default())
            .insert_resource(Metrics {
                ticks: 10,
                ..default()
            })
            .insert_resource(Sampler {
                since,
                ticks: 0,
                frames: 0,
                simulated: Duration::ZERO,
                log: false,
            })
            .insert_resource(Time::<Virtual>::default());
        app.world_mut()
            .resource_mut::<Time<Virtual>>()
            .advance_by(Duration::from_secs(2));
        app.world()
            .resource::<Frames>()
            .0
            .store(20, Ordering::Relaxed);
        sample(app.world_mut(), since + Duration::from_secs(1));
        let metrics = app.world().resource::<Metrics>();
        assert_eq!(
            (metrics.fps, metrics.tps, metrics.simulation_speed),
            (20., 10., 2.)
        );
        app.world()
            .resource::<Frames>()
            .0
            .store(50, Ordering::Relaxed);
        sample(app.world_mut(), since + Duration::from_secs(2));
        let metrics = app.world().resource::<Metrics>();
        assert_eq!(
            (metrics.fps, metrics.tps, metrics.simulation_speed),
            (30., 0., 0.)
        );
        assert_eq!(metrics.ticks, 10);
        assert_eq!(
            app.world().resource::<Time<Virtual>>().elapsed(),
            Duration::from_secs(2)
        );
    }

    #[test]
    fn no_ticks_or_frames_are_invented_before_the_sample_interval() {
        let mut app = App::new();
        let since = Instant::now();
        app.insert_resource(Frames::default())
            .insert_resource(Metrics::default())
            .insert_resource(Sampler {
                since,
                ticks: 0,
                frames: 0,
                simulated: Duration::ZERO,
                log: false,
            })
            .insert_resource(Time::<Virtual>::default());
        sample(app.world_mut(), since + Duration::from_millis(249));
        let metrics = app.world().resource::<Metrics>();
        assert_eq!(
            (metrics.tps, metrics.fps, metrics.sample_seconds),
            (0., 0., 0.)
        );
    }
}
