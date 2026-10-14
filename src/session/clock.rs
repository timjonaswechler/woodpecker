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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        command::tick::{self, warp},
        session::{plugin, protocol},
    };
    use std::{sync::mpsc, time::Instant};

    #[derive(Resource, Default)]
    struct Observed {
        deltas: Vec<Duration>,
        fixed_ticks: u32,
    }

    struct Session {
        app: App,
        input: mpsc::Sender<String>,
        output: mpsc::Receiver<(protocol::Message, Option<mpsc::Sender<()>>)>,
    }

    impl Session {
        fn new(clock: Clock) -> Self {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins)
                .insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(5)))
                .init_resource::<Observed>()
                .add_systems(Update, |time: Res<Time>, mut observed: ResMut<Observed>| {
                    observed.deltas.push(time.delta());
                })
                .add_systems(FixedUpdate, |mut observed: ResMut<Observed>| {
                    observed.fixed_ticks += 1;
                });
            let (input, commands) = mpsc::channel();
            let (messages, output) = mpsc::channel();
            plugin::install(
                &mut app,
                commands,
                messages,
                tick::Config {
                    pace: warp::Pace::AsFastAsPossible,
                    clock,
                },
            );
            Self { app, input, output }
        }

        fn warp(&mut self, ticks: u64, pace: Option<warp::Pace>) {
            self.input
                .send(protocol::encode(1, &warp::Start { ticks, pace }.into()))
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                self.app.update();
                if let Ok((message, _)) = self.output.try_recv() {
                    assert!(
                        matches!(message, protocol::Message::Completed { request_id: 1, output, .. }
                        if output["executed_ticks"] == ticks)
                    );
                    return;
                }
                assert!(Instant::now() < deadline, "warp did not finish");
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    }

    #[test]
    #[should_panic(expected = "fixed Woodpecker tick clock requires Bevy TimePlugin")]
    fn fixed_clock_requires_time_plugin_at_installation() {
        let mut app = App::new();
        let (_input, commands) = mpsc::channel();
        let (messages, _output) = mpsc::channel();
        plugin::install(
            &mut app,
            commands,
            messages,
            tick::Config {
                clock: Clock::Fixed {
                    seconds_per_tick: 0.01,
                },
                ..Default::default()
            },
        );
    }

    #[test]
    fn fixed_ticks_ignore_wall_delays_and_pace_and_idle_updates_do_not_advance_time() {
        let mut session = Session::new(Clock::Fixed {
            seconds_per_tick: 0.01,
        });
        session.app.update();
        assert!(session.app.world().resource::<Observed>().deltas.is_empty());
        assert_eq!(
            session.app.world().resource::<Time<Real>>().elapsed(),
            Duration::ZERO
        );
        session.warp(1, None);
        // The first counted tick already advances the full duration.
        assert_eq!(
            session.app.world().resource::<Time<Real>>().elapsed(),
            Duration::from_millis(10)
        );
        assert_eq!(session.app.world().resource::<Observed>().fixed_ticks, 2);

        std::thread::sleep(Duration::from_millis(25));
        for _ in 0..4 {
            session.app.update();
        }
        assert_eq!(
            session.app.world().resource::<Time<Real>>().elapsed(),
            Duration::from_millis(10)
        );
        assert_eq!(session.app.world().resource::<Observed>().deltas.len(), 1);
        session.warp(3, Some(warp::Pace::TicksPerSecond { target: 200.0 }));
        session.warp(2, Some(warp::Pace::AsFastAsPossible));
        assert_eq!(
            session.app.world().resource::<Time<Real>>().elapsed(),
            Duration::from_millis(60)
        );
        assert_eq!(
            session.app.world().resource::<Time<Virtual>>().elapsed(),
            Duration::from_millis(60)
        );
        let observed = session.app.world().resource::<Observed>();
        assert_eq!(observed.deltas, vec![Duration::from_millis(10); 6]);
        assert_eq!(observed.fixed_ticks, 12);
    }

    #[test]
    fn application_policy_is_preserved_including_its_first_zero_delta() {
        let mut session = Session::new(Clock::Application {});
        session
            .app
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                17,
            )));
        session.warp(2, None);
        assert_eq!(
            session.app.world().resource::<Observed>().deltas,
            [Duration::ZERO, Duration::from_millis(17)]
        );
        assert!(
            matches!(session.app.world().resource::<TimeUpdateStrategy>(),
            TimeUpdateStrategy::ManualDuration(duration) if *duration == Duration::from_millis(17))
        );
    }

    #[test]
    fn fixed_policy_preserves_virtual_pause_speed_and_maximum_delta() {
        let mut session = Session::new(Clock::Fixed {
            seconds_per_tick: 0.02,
        });
        session
            .app
            .world_mut()
            .resource_mut::<Time<Virtual>>()
            .pause();
        session.warp(1, None);
        assert_eq!(
            session.app.world().resource::<Time<Real>>().elapsed(),
            Duration::from_millis(20)
        );
        assert_eq!(
            session.app.world().resource::<Time<Virtual>>().elapsed(),
            Duration::ZERO
        );
        {
            let mut virtual_time = session.app.world_mut().resource_mut::<Time<Virtual>>();
            virtual_time.unpause();
            virtual_time.set_relative_speed(2.0);
        }
        session.warp(1, None);
        assert_eq!(
            session.app.world().resource::<Time<Real>>().elapsed(),
            Duration::from_millis(40)
        );
        assert_eq!(
            session.app.world().resource::<Time<Virtual>>().elapsed(),
            Duration::from_millis(40)
        );
        session
            .app
            .world_mut()
            .resource_mut::<Time<Virtual>>()
            .set_max_delta(Duration::from_millis(10));
        session.warp(1, None);
        assert_eq!(
            session.app.world().resource::<Time<Real>>().elapsed(),
            Duration::from_millis(60)
        );
        assert_eq!(
            session.app.world().resource::<Time<Virtual>>().elapsed(),
            Duration::from_millis(50)
        );
        assert_eq!(session.app.world().resource::<Observed>().fixed_ticks, 10);
    }
}
