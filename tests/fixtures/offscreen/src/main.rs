//! Explicit, windowless product/SDK proof. No library tests or CI autorun.
use bevy::{
    camera::RenderTarget,
    core_pipeline::tonemapping::Tonemapping,
    prelude::*,
    render::{
        RenderPlugin,
        pipelined_rendering::PipelinedRenderingPlugin,
        renderer::{RenderAdapter, RenderAdapterInfo, RenderDevice, RenderInstance, RenderQueue},
        settings::RenderCreation,
    },
    window::{ExitCondition, WindowPlugin},
};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use woodpecker::{
    command::{self, screenshot, tick::warp},
    session::{self, Session},
};

#[derive(Resource, Default)]
struct Counts {
    updates: u64,
    fixed: u64,
}
mod scene3d;

#[derive(Resource)]
struct FaultTick(u64);

#[derive(Resource)]
struct Scene {
    moving: Entity,
    material: Handle<ColorMaterial>,
}
fn update(
    mut counts: ResMut<Counts>,
    scene: Option<Res<Scene>>,
    fault: Res<FaultTick>,
    mut transforms: Query<&mut Transform>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    device: Res<RenderDevice>,
) {
    counts.updates += 1;
    if counts.updates == 2
        && let Some(scene) = scene
    {
        transforms.get_mut(scene.moving).unwrap().translation.x = 12.0;
        materials.get_mut(&scene.material).unwrap().color = Color::srgb_u8(0, 255, 0);
    }
    if counts.updates == fault.0 {
        if fault.0 == 2 {
            let marker = PathBuf::from(std::env::var_os("WOODPECKER_ARTIFACT_DIR").unwrap())
                .join("fault-ready");
            let end = Instant::now() + Duration::from_secs(5);
            while !marker.exists() {
                assert!(
                    Instant::now() < end,
                    "driver must have accepted the pending capture before fault"
                );
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        device.wgpu_device().destroy();
    }
}
fn fixed(mut counts: ResMut<Counts>) {
    counts.fixed += 1;
}

fn app(mode: &str) {
    assert_eq!(
        std::env::var("WOODPECKER_OFFSCREEN_TEST_GPU").as_deref(),
        Ok("1"),
        "explicit GPU permission required"
    );
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::METAL,
        flags: wgpu::InstanceFlags::VALIDATION | wgpu::InstanceFlags::DEBUG,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: None,
        force_fallback_adapter: false,
        ..Default::default()
    }))
    .expect("Metal hardware prerequisite, no fallback");
    let info = adapter.get_info();
    assert_eq!(info.backend, wgpu::Backend::Metal);
    assert!(matches!(
        info.device_type,
        wgpu::DeviceType::IntegratedGpu | wgpu::DeviceType::DiscreteGpu
    ));
    eprintln!("ADAPTER {info:?}; surface=false; window=false");
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
        required_limits: wgpu::Limits::default(),
        ..Default::default()
    }))
    .unwrap();
    let creation = RenderCreation::manual(
        RenderDevice::from(device),
        RenderQueue::new(queue),
        RenderAdapterInfo::new(info),
        RenderAdapter::new(adapter),
        RenderInstance::new(instance),
    );
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            })
            .set(RenderPlugin {
                render_creation: creation,
                synchronous_pipeline_compilation: !mode.ends_with("-3d"),
                ..default()
            })
            .set(bevy::log::LogPlugin {
                custom_layer: session::tracing_error_layer,
                ..default()
            })
            .disable::<PipelinedRenderingPlugin>(),
    );
    let mut image = Image::new_target_texture(65, 49, wgpu::TextureFormat::Rgba8UnormSrgb, None);
    image.texture_descriptor.usage |= wgpu::TextureUsages::COPY_SRC;
    let image = app.world_mut().resource_mut::<Assets<Image>>().add(image);
    if mode != "native-default" {
        app.insert_resource(session::OffscreenTarget(image.clone()));
    }
    if mode.ends_with("-3d") {
        scene3d::install(&mut app, image, mode);
    } else {
        install_2d(&mut app, image);
    }
    app.insert_resource(ClearColor(Color::BLACK))
        .init_resource::<Counts>()
        .insert_resource(FaultTick(match mode {
            "early-loss" => 1,
            "device-loss" | "device-loss-3d" => 2,
            _ => u64::MAX,
        }))
        .add_systems(Update, update)
        .add_systems(FixedUpdate, fixed)
        .add_plugins(session::Plugin);
    app.finish();
    app.cleanup();
    let root = PathBuf::from(std::env::var_os("WOODPECKER_ARTIFACT_DIR").unwrap());
    let end = Instant::now() + Duration::from_secs(100);
    let mut previous = serde_json::Value::Null;
    loop {
        assert!(Instant::now() < end, "fixture watchdog; no retries");
        app.update();
        let world = app.world();
        let c = world.resource::<Counts>();
        let snapshot = serde_json::json!({"updates":c.updates,"fixed":c.fixed,
            "real_ns":world.resource::<Time<Real>>().elapsed().as_nanos(),
            "virtual_ns":world.resource::<Time<Virtual>>().elapsed().as_nanos(),
            "fixed_ns":world.resource::<Time<Fixed>>().elapsed().as_nanos()});
        if snapshot != previous {
            std::fs::write(root.join("snapshot.json"), snapshot.to_string()).unwrap();
            previous = snapshot;
        }
        if app.should_exit().is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn install_2d(app: &mut App, image: Handle<Image>) {
    app.world_mut().spawn((
        Camera2d,
        RenderTarget::Image(image.into()),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::Fixed {
                width: 65.0,
                height: 49.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        Msaa::Off,
        Tonemapping::None,
    ));
    let mesh = app
        .world_mut()
        .resource_mut::<Assets<Mesh>>()
        .add(Rectangle::new(20.0, 16.0));
    let material = app
        .world_mut()
        .resource_mut::<Assets<ColorMaterial>>()
        .add(Color::srgb_u8(255, 0, 0));
    let moving = app
        .world_mut()
        .spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material.clone()),
            Transform::from_xyz(-12.0, 0.0, 0.0),
        ))
        .id();
    for (x, y, color) in [
        (-26.0, 18.0, [0, 0, 255]),
        (26.0, 18.0, [255, 255, 0]),
        (-26.0, -18.0, [0, 255, 255]),
        (26.0, -18.0, [255, 0, 255]),
    ] {
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(Rectangle::new(8.0, 8.0));
        let material = app
            .world_mut()
            .resource_mut::<Assets<ColorMaterial>>()
            .add(Color::srgb_u8(color[0], color[1], color[2]));
        app.world_mut().spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(x, y, 0.0),
        ));
    }
    app.insert_resource(Scene { moving, material });
}

fn outcome<T: serde::de::DeserializeOwned>(
    session: &mut Session,
    mut pending: session::Pending<T>,
) -> Result<T, session::Error> {
    let end = Instant::now() + Duration::from_secs(35);
    loop {
        match session.try_receive(&mut pending) {
            Ok(Some(value)) => return Ok(value),
            Err(error) => return Err(error),
            _ => {}
        }
        assert!(
            Instant::now() < end,
            "pending observation timeout; no resubmission"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}
fn warp(session: &mut Session) {
    let pending = session
        .send(warp::Start {
            ticks: 1,
            pace: None,
        })
        .unwrap();
    assert_eq!(outcome(session, pending).unwrap().executed_ticks, 1);
}
fn capture(session: &mut Session, name: &str) -> screenshot::Output {
    let pending = session.send(screenshot::Capture::new(name)).unwrap();
    let output = outcome(session, pending).unwrap();
    assert_eq!((output.width, output.height), (65, 49));
    output
}
fn pixels(root: &Path, name: &str) -> image::RgbImage {
    let image = image::open(root.join(name)).unwrap();
    assert_eq!(
        image.color(),
        image::ColorType::Rgb8,
        "product RGB contract"
    );
    image.to_rgb8()
}
fn driver(mode: &str, root: PathBuf) {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let config = session::Config {
        launch: session::launch::Config {
            manifest_path: manifest,
            package: "woodpecker-offscreen-fixture".into(),
            target: session::launch::Target::Binary {
                name: "woodpecker-offscreen-fixture".into(),
            },
            features: vec![],
            arguments: vec!["app".into(), mode.into()],
            executable: Some(std::env::current_exe().unwrap()),
        },
        artifact_dir: root.clone(),
        tick: command::tick::Config {
            clock: command::tick::Clock::Fixed {
                seconds_per_tick: 0.016666667,
            },
            ..default()
        },
        startup: session::startup::Config::Manual {},
        report: woodpecker::report::Config {
            tracing_errors: false,
            output: "reports".into(),
            provider: woodpecker::report::provider::Config::Local,
        },
    };
    let mut session = Session::start(config).unwrap();
    assert!(session.capabilities().screenshot);
    assert_eq!(session.capabilities().offscreen, mode != "native-default");
    if mode.ends_with("-3d") {
        scene3d::verify(&mut session, &root, mode);
        return;
    }
    if mode == "native-default" {
        let pending = session
            .send(screenshot::Capture::new("no-fallback.png"))
            .unwrap();
        assert_eq!(
            outcome(&mut session, pending).unwrap_err().code(),
            "screenshot_window_unavailable"
        );
        assert!(!root.join("no-fallback.png").exists());
        session.shutdown().unwrap();
        let snapshot: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("snapshot.json")).unwrap()).unwrap();
        assert_eq!(snapshot["updates"], 0);
        assert_eq!(snapshot["real_ns"], 0);
        println!("PASS native default selection; no implicit image fallback or ticks");
        return;
    }
    if mode == "early-loss" {
        let pending = session
            .send(warp::Start {
                ticks: 1,
                pace: None,
            })
            .unwrap();
        let _ = outcome(&mut session, pending); // completion may precede the asynchronous fatal
    } else {
        let pending = session
            .send(screenshot::Capture::new("before.png"))
            .unwrap();
        let error = outcome(&mut session, pending).unwrap_err();
        assert!(matches!(error, session::Error::Render { .. }));
        assert_eq!(error.code(), "render_initial_tick_required");
        assert!(!root.join("before.png").exists());
        warp(&mut session);
        assert!(!capture(&mut session, "a.png").overwritten);
        let a = pixels(&root, "a.png");
        assert_eq!(a.get_pixel(20, 24).0, [255, 0, 0]);
        for (x, y, c) in [
            (6, 6, [0, 0, 255]),
            (58, 6, [255, 255, 0]),
            (6, 42, [0, 255, 255]),
            (58, 42, [255, 0, 255]),
        ] {
            assert_eq!(a.get_pixel(x, y).0, c);
        }
        let before = std::fs::read(root.join("snapshot.json")).unwrap();
        capture(&mut session, "pause.png");
        assert_eq!(pixels(&root, "pause.png"), a);
        assert_eq!(
            std::fs::read(root.join("snapshot.json")).unwrap(),
            before,
            "pause must preserve every clock/counter"
        );
        if mode == "healthy" {
            assert!(capture(&mut session, "a.png").overwritten);
            let bad = session
                .send(screenshot::Capture::new("../escape.png"))
                .unwrap();
            assert_eq!(
                outcome(&mut session, bad).unwrap_err().code(),
                "invalid_screenshot_path"
            );
            warp(&mut session);
            capture(&mut session, "b.png");
            let b = pixels(&root, "b.png");
            assert_eq!(b.get_pixel(20, 24).0, [0, 0, 0]);
            assert_eq!(b.get_pixel(44, 24).0, [0, 255, 0]);
            assert_eq!(
                a.pixels().zip(b.pixels()).filter(|(a, b)| a != b).count(),
                640
            );
            let snapshot: serde_json::Value =
                serde_json::from_slice(&std::fs::read(root.join("snapshot.json")).unwrap())
                    .unwrap();
            assert_eq!(snapshot["updates"], 2);
            session.shutdown().unwrap();
            println!(
                "PASS product RGB capture, pause/time, explicit Tick2, overwrite, invalid path"
            );
            return;
        }
        let warp = session
            .send(warp::Start {
                ticks: 1,
                pace: None,
            })
            .unwrap();
        let shot = session
            .send(screenshot::Capture::new("must-not-publish.png"))
            .unwrap();
        std::fs::write(root.join("fault-ready"), b"pending accepted").unwrap();
        let _ = outcome(&mut session, warp);
        let error = outcome(&mut session, shot).unwrap_err();
        assert!(matches!(error, session::Error::Render { .. }), "{error:?}");
        assert_eq!(
            serde_json::to_value(session.history().last().unwrap()).unwrap()["outcome"]["status"],
            "render_failed"
        );
        assert!(
            ["render_device_lost", "render_panicked"].contains(&error.code()),
            "{error:?}"
        );
    }
    render_end(&mut session, &root, mode);
}

fn render_end(session: &mut Session, root: &Path, mode: &str) {
    let end = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(session::Event::Ended { reason }) = session.try_receive_event().unwrap() {
            match reason {
                session::EndReason::RenderFailed { error } => assert!(
                    ["render_device_lost", "render_panicked"].contains(&error.code.as_str()),
                    "{error:?}"
                ),
                other => panic!("{other:?}"),
            }
            break;
        }
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(!root.join("must-not-publish.png").exists());
    println!("PASS typed fatal transport without another simulation tick or publication: {mode}");
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("app") => app(&args[2]),
        Some("verify") => driver(&args[2], PathBuf::from(&args[3])),
        Some("audit-3d") => scene3d::controls(
            Path::new(&args[2]),
            Path::new(&args[3]),
            Path::new(&args[4]),
        ),
        _ => panic!("explicit app/verify mode required; see tests/headless/README.md"),
    }
}
