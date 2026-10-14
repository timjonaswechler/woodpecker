//! The command bridge and Bevy SubApp boundary, without a GPU or native window.
use super::*;
use bevy::{app::SubApp, render::RenderApp, time::TimeUpdateStrategy};
use std::time::Duration;

#[derive(Resource, Default)]
struct Simulation {
    updates: u64,
    post_updates: u64,
}

#[derive(Resource, Default)]
struct Rendered {
    extractions: u64,
    frames: u64,
    prepared: bool,
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct RenderFrame;

type Responses = mpsc::Receiver<(Message, Option<mpsc::Sender<()>>)>;

#[test]
fn idle_extraction_retains_shadow_tables_but_counted_ticks_allow_removal() {
    use bevy::render::{
        Render,
        view::visibility::{
            RenderExtractedShadowMapVisibleEntities, RenderShadowMapVisibleEntities,
        },
    };
    let (mut app, sender, _receiver) = app();
    // Exercise the same command/extraction boundary as Bevy's changed-light
    // cleanup. Its deferred removal must be repaired before render preparation.
    let render_app = app.sub_app_mut(RenderApp);
    render_app.world_mut().spawn((
        RenderExtractedShadowMapVisibleEntities::default(),
        RenderShadowMapVisibleEntities::default(),
    ));
    render_app.add_systems(RenderFrame, |world: &mut World| {
        world.run_schedule(Render);
    });
    let mut wrapped = render_app.take_extract().unwrap();
    render_app.set_extract(move |main, render| {
        wrapped(main, render);
        let entities: Vec<_> = render
            .iter_entities()
            .filter(|entity| entity.contains::<RenderShadowMapVisibleEntities>())
            .map(|entity| entity.id())
            .collect();
        for entity in entities {
            render.entity_mut(entity).remove::<(
                RenderExtractedShadowMapVisibleEntities,
                RenderShadowMapVisibleEntities,
            )>();
        }
    });
    sender
        .send(protocol::encode(
            1,
            &warp::Start {
                ticks: 1,
                pace: None,
            }
            .into(),
        ))
        .unwrap();
    app.update();
    let render_world = app.sub_app_mut(RenderApp).world_mut();
    let light = render_world
        .spawn((
            RenderExtractedShadowMapVisibleEntities::default(),
            RenderShadowMapVisibleEntities::default(),
        ))
        .id();
    app.update();
    assert!(
        app.sub_app(RenderApp)
            .world()
            .get::<RenderShadowMapVisibleEntities>(light)
            .is_some()
    );
    sender
        .send(protocol::encode(
            2,
            &warp::Start {
                ticks: 1,
                pace: None,
            }
            .into(),
        ))
        .unwrap();
    app.update();
    assert!(
        app.sub_app(RenderApp)
            .world()
            .get::<RenderShadowMapVisibleEntities>(light)
            .is_none()
    );
}

fn app() -> (App, mpsc::Sender<String>, Responses) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Simulation>()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            17,
        )))
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera3d::default());
        })
        .add_systems(Update, |mut simulation: ResMut<Simulation>| {
            simulation.updates += 1
        })
        .add_systems(PostUpdate, |mut simulation: ResMut<Simulation>| {
            simulation.post_updates += 1
        });
    let mut render = SubApp::new();
    render.init_resource::<Rendered>();
    render.update_schedule = Some(RenderFrame.intern());
    render.set_extract(|main, render| {
        let mut frame = render.resource_mut::<Rendered>();
        frame.extractions += 1;
        frame.prepared = main.resource::<Simulation>().post_updates > 0;
    });
    render.add_systems(RenderFrame, |mut frame: ResMut<Rendered>| {
        assert!(
            frame.prepared,
            "renderer received unprepared startup data before an explicit tick"
        );
        frame.frames += 1;
    });
    app.insert_sub_app(RenderApp, render);
    let (sender, input) = mpsc::channel();
    let (output, receiver) = mpsc::channel();
    install(&mut app, input, output, tick::Config::default());
    (app, sender, receiver)
}

#[test]
fn pipelined_renderer_keeps_its_handoff_live_while_waiting_for_the_first_tick() {
    #[derive(Resource)]
    struct Frames(mpsc::Sender<()>);
    let (mut app, sender, _receiver) = app();
    let (frames, received) = mpsc::channel();
    app.sub_app_mut(RenderApp)
        .insert_resource(Frames(frames))
        .add_systems(RenderFrame, |frames: Res<Frames>| {
            frames.0.send(()).unwrap();
        });
    app.add_plugins(bevy::render::pipelined_rendering::PipelinedRenderingPlugin);
    app.finish();
    app.cleanup();
    // These updates perform the real pipelined send/receive handoff, but must
    // neither extract unprepared camera data nor execute the render workload.
    for _ in 0..5 {
        app.update();
    }
    assert!(received.try_recv().is_err());
    assert_eq!(app.world().resource::<Simulation>().post_updates, 0);
    assert_eq!(
        app.world().resource::<Time<Real>>().elapsed(),
        Duration::ZERO
    );
    sender
        .send(protocol::encode(
            1,
            &warp::Start {
                ticks: 1,
                pace: None,
            }
            .into(),
        ))
        .unwrap();
    app.update();
    app.update(); // Receive the render app from the render thread again.
    received.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(app.world().resource::<Simulation>().updates, 1);
    assert_eq!(app.world().resource::<Simulation>().post_updates, 1);
}

#[test]
fn render_startup_waits_for_an_explicit_tick_without_advancing_application_state() {
    let (mut app, sender, receiver) = app();
    for _ in 0..5 {
        app.update();
    }
    let simulation = app.world().resource::<Simulation>();
    assert_eq!((simulation.updates, simulation.post_updates), (0, 0));
    assert_eq!(
        app.world().resource::<Time<Real>>().elapsed(),
        Duration::ZERO
    );
    let mut cameras = app.world_mut().query::<&Camera>();
    assert!(cameras.single(app.world()).unwrap().is_active);
    let frame = app.sub_app(RenderApp).world().resource::<Rendered>();
    assert_eq!((frame.extractions, frame.frames), (0, 0));

    sender
        .send(protocol::encode(
            1,
            &warp::Start {
                ticks: 1,
                pace: None,
            }
            .into(),
        ))
        .unwrap();
    app.update();
    assert!(matches!(
        receiver.try_recv().unwrap().0,
        Message::Completed { request_id: 1, .. }
    ));
    let frame = app.sub_app(RenderApp).world().resource::<Rendered>();
    assert_eq!((frame.extractions, frame.frames), (1, 1));
    let before = app.world().resource::<Time<Real>>().elapsed();
    for _ in 0..5 {
        app.update();
    }
    let simulation = app.world().resource::<Simulation>();
    assert_eq!((simulation.updates, simulation.post_updates), (1, 1));
    assert_eq!(app.world().resource::<Time<Real>>().elapsed(), before);
    assert!(cameras.single(app.world()).unwrap().is_active);
    let frame = app.sub_app(RenderApp).world().resource::<Rendered>();
    assert_eq!((frame.extractions, frame.frames), (6, 6));
}
