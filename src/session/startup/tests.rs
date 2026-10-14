use super::*;
use crate::{
    command::{self, tick},
    session::{plugin, protocol},
};
use bevy::{app::SubApp, render::RenderApp, window::PrimaryWindow};
use std::{sync::mpsc, time::Duration};

#[derive(Resource, Default)]
struct Count {
    ticks: u64,
    elapsed: Duration,
    list_known: bool,
}

fn fixture(
    max_ticks: u64,
) -> (
    App,
    mpsc::Sender<String>,
    mpsc::Receiver<(Message, Option<mpsc::Sender<()>>)>,
) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Count>()
        .add_systems(Update, |mut count: ResMut<Count>, time: Res<Time>| {
            count.ticks += 1;
            count.elapsed = time.elapsed();
            if count.ticks >= 3 {
                count.list_known = true;
            }
        });
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    app.insert_sub_app(RenderApp, SubApp::new());
    let (tx, input) = mpsc::channel();
    let (output, rx) = mpsc::channel();
    plugin::install(
        &mut app,
        input,
        output,
        tick::Config {
            clock: tick::Clock::Fixed {
                seconds_per_tick: 0.01,
            },
            ..default()
        },
    );
    app.insert_resource(Assets(|world| {
        world.resource::<Count>().list_known.then(Vec::new)
    }));
    install(
        &mut app,
        Config::Engine {
            timeout_seconds: 10,
            max_ticks,
        },
    );
    (app, tx, rx)
}

#[test]
fn startup_counts_normal_ticks_waits_for_frame_and_freezes_until_explicit_warp() {
    let (mut app, tx, rx) = fixture(20);
    tx.send(protocol::encode(
        1,
        &command::input::pointer::MoveBy { delta: [1., 2.] }.into(),
    ))
    .unwrap();
    for _ in 0..4 {
        app.update();
    }
    assert!(
        matches!(rx.try_recv().unwrap().0, Message::Rejected { error, .. } if error.code == "startup_in_progress")
    );
    assert_eq!(app.world().resource::<Count>().ticks, 4);
    assert_eq!(
        app.world().resource::<Count>().elapsed,
        Duration::from_millis(40)
    );
    assert!(pending(app.world()));
    for _ in 0..5 {
        app.update();
    }
    assert_eq!(
        app.world().resource::<Count>().ticks,
        4,
        "waiting for GPU must not add hidden ticks"
    );
    assert!(rx.try_recv().is_err());
    render::acknowledge(app.world());
    app.update();
    assert!(matches!(
        rx.try_recv().unwrap().0,
        Message::EngineReady { ticks: 4 }
    ));
    assert!(!pending(app.world()));
    for _ in 0..5 {
        app.update();
    }
    assert!(rx.try_recv().is_err(), "EngineReady is emitted once");
    assert_eq!(
        app.world().resource::<Count>().elapsed,
        Duration::from_millis(40)
    );
    tx.send(protocol::encode(
        2,
        &command::tick::warp::Start {
            ticks: 2,
            pace: None,
        }
        .into(),
    ))
    .unwrap();
    app.update();
    assert_eq!(app.world().resource::<Count>().ticks, 6);
    assert_eq!(
        app.world().resource::<Count>().elapsed,
        Duration::from_millis(60)
    );
}

#[test]
fn tick_limit_and_wall_timeout_fail_once_without_extra_updates() {
    let (mut app, _tx, rx) = fixture(2);
    for _ in 0..4 {
        app.update();
    }
    assert!(
        matches!(rx.try_recv().unwrap().0, Message::StartupFailed { ticks: 2, error } if error.code == "startup_tick_limit")
    );
    assert_eq!(app.world().resource::<Count>().ticks, 2);
    assert!(rx.try_recv().is_err());

    let (mut app, _tx, rx) = fixture(20);
    app.update();
    app.world_mut().resource_mut::<State>().started =
        Some(Instant::now() - Duration::from_secs(20));
    app.update();
    assert!(
        matches!(rx.try_recv().unwrap().0, Message::StartupFailed { ticks: 1, error } if error.code == "startup_timeout")
    );
}

#[test]
fn missing_adapter_or_renderer_cannot_report_engine_ready() {
    let (mut app, _tx, rx) = fixture(20);
    app.world_mut().remove_resource::<Assets>();
    app.update();
    assert!(
        matches!(rx.try_recv().unwrap().0, Message::StartupFailed { ticks: 0, error } if error.code == "startup_asset_scope_missing")
    );
    let mut app = App::new();
    app.insert_resource(Assets(|_| Some(vec![])));
    install(
        &mut app,
        Config::Engine {
            timeout_seconds: 10,
            max_ticks: 20,
        },
    );
    assert!(
        matches!(poll(app.world_mut(), 0), Some(Message::StartupFailed { error, .. }) if error.code == "startup_renderer_missing")
    );
}

#[test]
fn manual_startup_is_default_and_engine_limits_are_required() {
    assert!(!Config::default().enabled());
    for value in [
        r#"{"kind":"engine","timeout_seconds":0,"max_ticks":3}"#,
        r#"{"kind":"engine","timeout_seconds":1,"max_ticks":0}"#,
    ] {
        assert!(!serde_json::from_str::<Config>(value).unwrap().valid());
    }
    assert!(serde_json::from_str::<Config>(r#"{"kind":"manual","max_ticks":3}"#).is_err());
}

#[test]
fn asset_load_failure_is_reported_with_its_cause() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>();
    let handle = app
        .world()
        .resource::<AssetServer>()
        .load::<Image>("woodpecker-test-missing-image.png");
    #[derive(Resource)]
    struct Root(UntypedHandle);
    app.insert_resource(Root(handle.untyped()));
    app.insert_resource(Assets(|world| {
        Some(vec![world.resource::<Root>().0.clone()])
    }));
    assert!(matches!(prerequisites(app.world()), Ok(Probe::Waiting(_))));
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        app.update();
        if let Err(error) = prerequisites(app.world()) {
            assert_eq!(error.code, "startup_asset_failed");
            assert!(
                error.message.contains("woodpecker-test-missing-image.png"),
                "{}",
                error.message
            );
            break;
        }
        assert!(Instant::now() < end, "missing asset did not fail");
        std::thread::yield_now();
    }
}
