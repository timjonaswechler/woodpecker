//! Inspect the same station models as the level, in isolation and in each state.
use crate::{
    expedition::{Expedition, Interaction as Action},
    interactions::{STATIONS, TABLES, Target},
    items::ItemKind,
    level::LevelAssets,
    scenery::Scenery,
    stations::{self, PreviewState},
};
use bevy::{
    camera::RenderTarget,
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    render::{
        render_resource::{TextureFormat, TextureUsages},
        view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
    },
    window::WindowResolution,
};
use std::path::PathBuf;

const MODELS: [(Target, &str, &str); 13] = [
    (STATIONS[0], "Auftragstafel", "board"),
    (STATIONS[1], "Montage", "assembly"),
    (STATIONS[2], "Lagerhebel", "lever"),
    (STATIONS[3], "Wasserventil", "valve"),
    (STATIONS[5], "Archivscanner", "scanner"),
    (STATIONS[8], "Generatorbedienung", "generator"),
    (STATIONS[9], "Sendeschalter", "sender"),
    (STATIONS[10], "Rückkehrpunkt", "checkpoint"),
    (STATIONS[11], "Belohnungstruhe", "chest"),
    (TABLES[0], "Bauteilablage", "part"),
    (TABLES[2], "Montageausgabe", "cell-assembly"),
    (TABLES[3], "Gartenausgabe", "cell-water"),
    (TABLES[4], "Archivausgabe", "cell-archive"),
];
const BG: Color = Color::srgb(0.76, 0.79, 0.76);
const INK: Color = Color::srgb(0.10, 0.18, 0.21);

#[derive(Resource)]
struct Zoo {
    selected: Option<usize>,
    yaw: f32,
    pitch: f32,
    zoom: f32,
}
impl Default for Zoo {
    fn default() -> Self {
        Self {
            selected: None,
            yaw: 0.55,
            pitch: 0.30,
            zoom: 1.,
        }
    }
}
#[derive(Component)]
struct Exhibit(usize);
#[derive(Component)]
struct SamplePickup;
#[derive(Component)]
struct Caption;
#[derive(Component)]
struct ExhibitLabel(usize);
#[derive(Resource)]
struct Capture {
    directory: PathBuf,
    image: Handle<Image>,
    shot: usize,
    frames: usize,
    pending: bool,
    loading_frames: usize,
}

pub fn run() {
    let args: Vec<_> = std::env::args().collect();
    let output = args.iter().position(|arg| arg == "--zoo-capture").map(|i| {
        PathBuf::from(
            args.get(i + 1)
                .expect("--zoo-capture needs an output directory"),
        )
    });
    let offscreen = output.is_some();
    let mut app = App::new();
    app.insert_resource(ClearColor(BG))
        .init_resource::<Zoo>()
        .init_resource::<PreviewState>()
        .init_resource::<Expedition>()
        .init_resource::<LevelAssets>()
        .insert_resource(GlobalAmbientLight {
            brightness: 450.,
            ..default()
        })
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: format!("{}/asset", env!("CARGO_MANIFEST_DIR")),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Signalstation | Modell-Zoo".into(),
                        visible: !offscreen,
                        resolution: WindowResolution::new(1440, 900).with_scale_factor_override(1.),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (input, stations::update_stations, update_view).chain(),
        )
        .add_systems(
            PostUpdate,
            update_labels.after(bevy::transform::TransformSystems::Propagate),
        );
    if let Some(directory) = output {
        std::fs::create_dir_all(&directory).expect("create zoo capture directory");
        let mut image = Image::new_target_texture(1440, 900, TextureFormat::Rgba8UnormSrgb, None);
        image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
        let handle = app.world_mut().resource_mut::<Assets<Image>>().add(image);
        app.insert_resource(Capture {
            directory,
            image: handle,
            shot: 0,
            frames: 0,
            pending: false,
            loading_frames: 0,
        })
        .insert_resource(bevy::winit::WinitSettings::continuous())
        .add_systems(Update, capture.after(update_view));
    }
    app.run();
}

fn position(i: usize) -> Vec3 {
    Vec3::new((i % 5) as f32 * 4.8 - 9.6, 0., (i / 5) as f32 * 4.6 - 4.6)
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut tracked: ResMut<LevelAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    capture: Option<Res<Capture>>,
) {
    let mut s = Scenery {
        commands: &mut commands,
        assets: &assets,
        tracked: &mut tracked,
        meshes: &mut meshes,
        materials: &mut materials,
        parent: None,
    };
    s.block(Vec3::new(0., -0.14, 0.), Vec3::new(200., 0.2, 200.), BG);
    for (i, &(target, _, _)) in MODELS.iter().enumerate() {
        let base = position(i);
        let root = stations::spawn_model(
            &mut s,
            Target {
                center: base + Vec3::Y * target.half.y,
                floor: 0.,
                ..target
            },
        );
        s.commands.entity(root).insert(Exhibit(i));
        let pickup = match target.action {
            Action::Part(_) => Some((
                ItemKind::Part,
                Transform::from_xyz(0., 0.94, 0.)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::splat(1.15)),
            )),
            Action::Cell(_) => Some((ItemKind::Cell, Transform::from_xyz(0., 1.32, 0.))),
            Action::Chest => Some((
                ItemKind::Blueprint,
                Transform::from_xyz(0., 0.72, 0.)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            )),
            _ => None,
        };
        if let Some((kind, transform)) = pickup {
            let item = s.item(kind, transform);
            s.commands.entity(item).insert(SamplePickup);
            s.commands.entity(root).add_child(item);
        }
    }
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-8., 16., 12.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Camera3d::default(),
        IsDefaultUiCamera,
        capture.as_ref().map_or(RenderTarget::default(), |c| {
            RenderTarget::Image(c.image.clone().into())
        }),
        bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: 20.,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::default(),
        Msaa::Sample4,
    ));
    for (i, &(_, name, _)) in MODELS.iter().enumerate() {
        commands.spawn((
            ExhibitLabel(i),
            Text::new(format!("{:02}  {name}", i + 1)),
            TextFont {
                font: assets.load("fonts/Barlow-Regular.ttf").into(),
                font_size: FontSize::Px(16.),
                ..default()
            },
            TextColor(INK),
            TextLayout::justify(Justify::Center),
            Node {
                position_type: PositionType::Absolute,
                width: px(200),
                ..default()
            },
        ));
    }
    commands.spawn((
        Text::new(""),
        Caption,
        TextFont {
            font: assets.load("fonts/Barlow-Regular.ttf").into(),
            font_size: FontSize::Px(25.),
            ..default()
        },
        TextColor(INK),
        Node {
            position_type: PositionType::Absolute,
            top: px(28),
            left: px(36),
            ..default()
        },
    ));
    commands.spawn((
        Text::new("Pfeil links / rechts  Modell wählen     O  Übersicht     1  Gesperrt     2  Bereit     3  Benutzt\nMaus ziehen  Drehen     Mausrad  Zoom     R  Ansicht zurücksetzen     Esc  Schließen"),
        TextFont { font: assets.load("fonts/Barlow-Regular.ttf").into(), font_size: FontSize::Px(18.), ..default() },
        TextColor(INK), Node { position_type: PositionType::Absolute, bottom: px(24), left: px(36), ..default() },
    ));
}

#[expect(
    clippy::too_many_arguments,
    reason = "Viewer keyboard, mouse and mode controls."
)]
fn input(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut zoo: ResMut<Zoo>,
    mut state: ResMut<PreviewState>,
    capture: Option<Res<Capture>>,
    mut exit: MessageWriter<AppExit>,
) {
    if capture.is_some() {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
    if keys.just_pressed(KeyCode::ArrowRight) {
        zoo.selected = Some(zoo.selected.map_or(0, |i| (i + 1) % MODELS.len()));
    }
    if keys.just_pressed(KeyCode::ArrowLeft) {
        zoo.selected = Some(
            zoo.selected
                .map_or(MODELS.len() - 1, |i| (i + MODELS.len() - 1) % MODELS.len()),
        );
    }
    if keys.just_pressed(KeyCode::KeyO) {
        zoo.selected = None;
    }
    if keys.just_pressed(KeyCode::Digit1) {
        *state = PreviewState::Locked;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        *state = PreviewState::Ready;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        *state = PreviewState::Used;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        let selected = zoo.selected;
        *zoo = Zoo {
            selected,
            ..default()
        };
    }
    if buttons.pressed(MouseButton::Left) {
        zoo.yaw -= motion.delta.x * 0.006;
        zoo.pitch = (zoo.pitch + motion.delta.y * 0.006).clamp(-0.1, 1.35);
    }
    zoo.zoom = (zoo.zoom * (-scroll.delta.y * 0.08).exp()).clamp(0.45, 2.);
}

fn update_view(
    zoo: Res<Zoo>,
    state: Res<PreviewState>,
    mut models: Query<(&Exhibit, &mut Visibility), Without<SamplePickup>>,
    mut items: Query<&mut Visibility, With<SamplePickup>>,
    mut cameras: Query<(&mut Transform, &mut Projection), With<Camera3d>>,
    mut caption: Query<&mut Text, With<Caption>>,
) {
    for (model, mut visibility) in &mut models {
        visibility.set_if_neq(if zoo.selected.is_none_or(|i| i == model.0) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    for mut visibility in &mut items {
        visibility.set_if_neq(if *state == PreviewState::Ready {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    let (focus, height, pitch) = if let Some(i) = zoo.selected {
        (
            position(i) + Vec3::Y * 0.90,
            if i == 10 { 3.7 } else { 3.2 },
            zoo.pitch,
        )
    } else {
        (Vec3::ZERO, 16., 0.82)
    };
    for (mut transform, mut projection) in &mut cameras {
        let direction = Vec3::new(
            zoo.yaw.sin() * pitch.cos(),
            pitch.sin(),
            zoo.yaw.cos() * pitch.cos(),
        );
        transform.set_if_neq(
            Transform::from_translation(focus + direction * 35.).looking_at(focus, Vec3::Y),
        );
        if let Projection::Orthographic(p) = &mut *projection {
            p.scaling_mode = bevy::camera::ScalingMode::FixedVertical {
                viewport_height: height * zoo.zoom,
            };
        }
    }
    let name = zoo.selected.map_or("Alle 13 Modelle".to_owned(), |i| {
        format!("{:02} / 13    {}", i + 1, MODELS[i].1)
    });
    let phase = match *state {
        PreviewState::Locked => "Gesperrt",
        PreviewState::Ready => "Bereit",
        PreviewState::Used => "Benutzt / abgeholt",
    };
    for mut text in &mut caption {
        text.0 = format!("MODELL-ZOO\n{name}    ·    {phase}");
    }
}

// Each model: front, back, profile, used and locked. Includes two overviews.
fn capture(
    mut commands: Commands,
    assets: Res<AssetServer>,
    tracked: Res<LevelAssets>,
    mut capture: ResMut<Capture>,
    mut zoo: ResMut<Zoo>,
    mut state: ResMut<PreviewState>,
    mut exit: MessageWriter<AppExit>,
) {
    capture.loading_frames += 1;
    assert!(
        capture.loading_frames < 10000,
        "zoo assets or screenshot timed out"
    );
    if !tracked
        .0
        .iter()
        .all(|h| assets.is_loaded_with_dependencies(h.id()))
    {
        return;
    }
    if capture.pending {
        return;
    }
    if capture.shot >= MODELS.len() * 5 + 2 {
        info!("ZOO CAPTURE COMPLETE: {} images", capture.shot);
        exit.write(AppExit::Success);
        return;
    }
    capture.frames += 1;
    let shot = capture.shot;
    if capture.frames == 1 {
        if shot < MODELS.len() * 5 {
            zoo.selected = Some(shot / 5);
            let view = shot % 5;
            zoo.yaw = [0.55, 3.69, 1.57, 0.55, 0.55][view];
            zoo.pitch = 0.30;
            *state = [
                PreviewState::Ready,
                PreviewState::Ready,
                PreviewState::Ready,
                PreviewState::Used,
                PreviewState::Locked,
            ][view];
        } else {
            zoo.selected = None;
            zoo.yaw = 0.20;
            *state = if shot == MODELS.len() * 5 {
                PreviewState::Ready
            } else {
                PreviewState::Used
            };
        }
        return;
    }
    if capture.frames < if shot == 0 { 80 } else { 12 } {
        return;
    }
    let name = if shot < MODELS.len() * 5 {
        format!(
            "{:02}-{}-{}.png",
            shot / 5 + 1,
            MODELS[shot / 5].2,
            ["front", "back", "side", "used", "locked"][shot % 5]
        )
    } else {
        format!(
            "overview-{}.png",
            if *state == PreviewState::Ready {
                "ready"
            } else {
                "used"
            }
        )
    };
    commands
        .spawn(Screenshot::image(capture.image.clone()))
        .observe(save_to_disk(capture.directory.join(name)))
        .observe(|_: On<ScreenshotCaptured>, mut capture: ResMut<Capture>| {
            capture.pending = false;
            capture.shot += 1;
            capture.frames = 0;
        });
    capture.pending = true;
}

fn update_labels(
    zoo: Res<Zoo>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut labels: Query<(&ExhibitLabel, &mut Node)>,
) {
    for (label, mut node) in &mut labels {
        node.display = if zoo.selected.is_none() {
            Display::Flex
        } else {
            Display::None
        };
        if let Ok(point) = camera
            .0
            .world_to_viewport(camera.1, position(label.0) + Vec3::Y * 2.25)
        {
            node.left = px(point.x - 100.);
            node.top = px(point.y);
        }
    }
}
