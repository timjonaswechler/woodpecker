//! Inspect the same station models as the level, in isolation and in each state.

use crate::{
    camera::{self, Separator, View, Views},
    ground::GroundPlugin,
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
use test_app::{
    level::LevelAssets,
    level::mission::inventory::ItemKind,
    level::objects::{self, ModelKind, VisualState},
    level::scene::Scenery,
};

const MODELS: [(ModelKind, &str, &str); 13] = [
    (ModelKind::Board, "Auftragstafel", "board"),
    (ModelKind::Workbench, "Montage", "assembly"),
    (ModelKind::StorageLever, "Lagerhebel", "lever"),
    (ModelKind::Valve, "Wasserventil", "valve"),
    (ModelKind::Scanner, "Archivscanner", "scanner"),
    (ModelKind::Generator, "Generatorbedienung", "generator"),
    (ModelKind::Transmitter, "Sendeschalter", "sender"),
    (ModelKind::Checkpoint, "Rückkehrpunkt", "checkpoint"),
    (ModelKind::Chest, "Belohnungstruhe", "chest"),
    (ModelKind::PartTable, "Bauteilablage", "part"),
    (ModelKind::AssemblyTable, "Montageausgabe", "cell-assembly"),
    (ModelKind::WaterTable, "Gartenausgabe", "cell-water"),
    (ModelKind::ArchiveTable, "Archivausgabe", "cell-archive"),
];
const BG: Color = Color::srgb(0.76, 0.79, 0.76);
const INK: Color = Color::srgb(0.10, 0.18, 0.21);

#[derive(Resource, Default)]
struct Zoo {
    selected: Option<usize>,
}
#[derive(Component)]
struct Exhibit(usize);
#[derive(Component)]
struct SamplePickup;
#[derive(Component)]
struct Caption;
#[derive(Component, Clone, Copy)]
enum ViewControl {
    Single(View),
    All,
}
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
        .init_resource::<Views>()
        .init_resource::<VisualState>()
        .init_resource::<LevelAssets>()
        .insert_resource(GlobalAmbientLight {
            brightness: 450.,
            ..default()
        })
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: test_app::ASSET_DIR.into(),
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
        .add_plugins(GroundPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (input, objects::update_visuals, update_view).chain(),
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
    for (i, &(target, _, _)) in MODELS.iter().enumerate() {
        let base = position(i);
        let root = objects::spawn_model(&mut s, target, base);
        s.commands.entity(root).insert(Exhibit(i));
        let pickup = match target.sample_item() {
            Some(ItemKind::Part) => Some((
                ItemKind::Part,
                Transform::from_xyz(0., 0.94, 0.)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::splat(1.15)),
            )),
            Some(ItemKind::Cell) => Some((ItemKind::Cell, Transform::from_xyz(0., 1.32, 0.))),
            Some(ItemKind::Blueprint) => Some((
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
    camera::spawn(
        &mut commands,
        capture.as_ref().map_or(RenderTarget::default(), |c| {
            RenderTarget::Image(c.image.clone().into())
        }),
    );
    for separator in [Separator::Vertical, Separator::Horizontal] {
        commands.spawn((
            Name::new("Kamera-Trennlinie"),
            separator,
            Node {
                position_type: PositionType::Absolute,
                display: Display::None,
                ..default()
            },
            BackgroundColor(INK.with_alpha(0.55)),
            GlobalZIndex(5),
            bevy::picking::Pickable::IGNORE,
        ));
    }
    for (index, view) in View::ALL.into_iter().enumerate() {
        view_control(
            &mut commands,
            &assets,
            ViewControl::Single(view),
            format!("{} [F{}] · Einzel / Vier", view.label(), index + 1),
        );
    }
    view_control(
        &mut commands,
        &assets,
        ViewControl::All,
        "Vier Ansichten [F5]".into(),
    );
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
        Text::new("Pfeil links / rechts  Modell wählen     O  Übersicht     1 / 2 / 3  Zustand\nF1 Oben · F2 Front · F3 Rechts · F4 Orbit: Einzel / Vier     F5 Vier Ansichten\nMaus ziehen im Orbit  Drehen     Mausrad über Ansicht  Zoom     R  Zurücksetzen     Esc  Schließen"),
        TextFont { font: assets.load("fonts/Barlow-Regular.ttf").into(), font_size: FontSize::Px(16.), ..default() },
        TextColor(INK), Node { position_type: PositionType::Absolute, bottom: px(24), left: px(36), ..default() },
    ));
}

#[expect(
    clippy::too_many_arguments,
    reason = "Viewer controls coordinate selection, preview state and the active viewport."
)]
fn input(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    windows: Query<&Window>,
    mut zoo: ResMut<Zoo>,
    mut views: ResMut<Views>,
    mut state: ResMut<VisualState>,
    capture: Option<Res<Capture>>,
    mut exit: MessageWriter<AppExit>,
) {
    if capture.is_some() {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
    let was_overview = zoo.selected.is_none();
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
    if was_overview && zoo.selected.is_some() {
        views.show_all();
    }
    if zoo.selected.is_some() {
        for (key, view) in [KeyCode::F1, KeyCode::F2, KeyCode::F3, KeyCode::F4]
            .into_iter()
            .zip(View::ALL)
        {
            if keys.just_pressed(key) {
                views.toggle(view);
            }
        }
        if keys.just_pressed(KeyCode::F5) {
            views.show_all();
        }
    }
    if keys.just_pressed(KeyCode::Digit1) {
        *state = VisualState::Locked;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        *state = VisualState::Ready;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        *state = VisualState::Used;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        views.reset();
    }
    let window = windows.single().ok();
    let (size, scale) = surface(window, false);
    let frame = views.frame(size, scale, zoo.selected.is_some());
    let hovered = frame.hovered(window.and_then(Window::cursor_position));
    views.drag(
        hovered,
        buttons.pressed(MouseButton::Left),
        buttons.just_pressed(MouseButton::Left),
        motion.delta,
    );
    views.zoom(hovered, scroll.delta.y);
}

fn surface(window: Option<&Window>, offscreen: bool) -> (UVec2, f32) {
    match window {
        Some(window) if !offscreen => (window.physical_size(), window.scale_factor()),
        _ => (UVec2::new(1440, 900), 1.),
    }
}

fn view_control(commands: &mut Commands, assets: &AssetServer, action: ViewControl, title: String) {
    commands
        .spawn((
            bevy::ui_widgets::Button,
            action,
            Node {
                position_type: PositionType::Absolute,
                display: Display::None,
                padding: UiRect::axes(px(10), px(6)),
                border: px(1).all(),
                border_radius: BorderRadius::all(px(4)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.93, 0.95, 0.93, 0.96)),
            BorderColor::all(INK),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(title),
                TextLayout::no_wrap(),
                TextFont {
                    font: assets.load("fonts/Barlow-Regular.ttf").into(),
                    font_size: FontSize::Px(14.),
                    ..default()
                },
                TextColor(INK),
            ));
        })
        .observe(switch_view);
}

fn switch_view(
    event: On<bevy::ui_widgets::Activate>,
    controls: Query<&ViewControl>,
    zoo: Res<Zoo>,
    capture: Option<Res<Capture>>,
    mut views: ResMut<Views>,
) {
    if zoo.selected.is_none() || capture.is_some() {
        return;
    }
    let Ok(action) = controls.get(event.entity) else {
        return;
    };
    match action {
        ViewControl::Single(view) => views.toggle(*view),
        ViewControl::All => views.show_all(),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "The selected model, render surface and controls share one viewport layout."
)]
fn update_view(
    zoo: Res<Zoo>,
    state: Res<VisualState>,
    views: Res<Views>,
    capture: Option<Res<Capture>>,
    windows: Query<&Window>,
    mut models: Query<(&Exhibit, &mut Visibility), Without<SamplePickup>>,
    mut items: Query<&mut Visibility, With<SamplePickup>>,
    mut cameras: Query<(&View, &mut Transform, &mut Projection, &mut Camera)>,
    mut caption: Query<&mut Text, With<Caption>>,
    mut controls: Query<(&ViewControl, &mut Node), Without<Separator>>,
    mut separators: Query<(&Separator, &mut Node), Without<ViewControl>>,
) {
    for (model, mut visibility) in &mut models {
        visibility.set_if_neq(if zoo.selected.is_none_or(|i| i == model.0) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    for mut visibility in &mut items {
        visibility.set_if_neq(if *state == VisualState::Ready {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    let (focus, height) = if let Some(i) = zoo.selected {
        (
            position(i) + Vec3::Y * 0.90,
            if i == 10 { 3.7 } else { 3.2 },
        )
    } else {
        (Vec3::ZERO, 16.)
    };
    let detail = zoo.selected.is_some() && capture.is_none();
    let (size, scale) = surface(windows.single().ok(), capture.is_some());
    let frame = views.frame(size, scale, detail);
    for (view, mut transform, mut projection, mut camera) in &mut cameras {
        camera.viewport = frame.viewport(*view);
        camera.is_active = camera.viewport.is_some();
        camera.clear_color = frame.clear_color(*view);
        if !camera.is_active {
            continue;
        }
        transform.set_if_neq(views.transform(*view, focus, zoo.selected.is_none()));
        if let Projection::Orthographic(p) = &mut *projection {
            p.scaling_mode = bevy::camera::ScalingMode::FixedVertical {
                viewport_height: views.height(*view, height),
            };
        }
    }
    for (control, mut node) in &mut controls {
        node.display = Display::None;
        if !detail {
            continue;
        }
        match control {
            ViewControl::Single(view) => {
                if let Some(rect) = frame.rect(*view) {
                    node.display = Display::Flex;
                    node.left = px(rect.min.x as f32 / scale + 8.);
                    node.top = px(rect.min.y as f32 / scale + 8.);
                    node.max_width = px((rect.size().x as f32 / scale - 16.).max(0.));
                }
            }
            ViewControl::All => {
                node.display = Display::Flex;
                node.right = px(24);
                node.top = px(28);
            }
        }
    }
    for (separator, mut node) in &mut separators {
        if let Some(rect) = frame.separator(*separator) {
            node.display = Display::Flex;
            node.left = px(rect.min.x as f32 / scale);
            node.top = px(rect.min.y as f32 / scale);
            node.width = px(rect.size().x as f32 / scale);
            node.height = px(rect.size().y as f32 / scale);
        } else {
            node.display = Display::None;
        }
    }
    let name = zoo.selected.map_or("Alle 13 Modelle".to_owned(), |i| {
        format!("{:02} / 13    {}", i + 1, MODELS[i].1)
    });
    let phase = match *state {
        VisualState::Locked => "Gesperrt",
        VisualState::Ready => "Bereit",
        VisualState::Used => "Benutzt / abgeholt",
    };
    for mut text in &mut caption {
        text.0 = format!("MODELL-ZOO\n{name}    ·    {phase}");
    }
}

// Each model: front, back, profile, used and locked. Includes two overviews.
#[expect(
    clippy::too_many_arguments,
    reason = "Capture coordinates assets, cameras, image output and application exit."
)]
fn capture(
    mut commands: Commands,
    assets: Res<AssetServer>,
    tracked: Res<LevelAssets>,
    mut capture: ResMut<Capture>,
    mut zoo: ResMut<Zoo>,
    mut views: ResMut<Views>,
    mut state: ResMut<VisualState>,
    mut exit: MessageWriter<AppExit>,
) {
    capture.loading_frames += 1;
    assert!(
        capture.loading_frames < 10000,
        "zoo assets or screenshot timed out"
    );
    if !tracked
        .handles()
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
            views.yaw = [0.55, 3.69, 1.57, 0.55, 0.55][view];
            views.pitch = 0.30;
            *state = [
                VisualState::Ready,
                VisualState::Ready,
                VisualState::Ready,
                VisualState::Used,
                VisualState::Locked,
            ][view];
        } else {
            zoo.selected = None;
            views.yaw = 0.20;
            *state = if shot == MODELS.len() * 5 {
                VisualState::Ready
            } else {
                VisualState::Used
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
            if *state == VisualState::Ready {
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
    cameras: Query<(&View, &Camera, &GlobalTransform)>,
    mut labels: Query<(&ExhibitLabel, &mut Node)>,
) {
    let Some((_, camera, transform)) = cameras.iter().find(|(view, _, _)| **view == View::Orbit)
    else {
        return;
    };
    for (label, mut node) in &mut labels {
        node.display = if zoo.selected.is_none() {
            Display::Flex
        } else {
            Display::None
        };
        if zoo.selected.is_none()
            && let Ok(point) =
                camera.world_to_viewport(transform, position(label.0) + Vec3::Y * 2.25)
        {
            node.left = px(point.x - 100.);
            node.top = px(point.y);
        }
    }
}
