//! Optional rendered integration check. Exercises the same systems as normal gameplay.
use crate::level;
mod notes;
mod objects;
use crate::{
    app::settings::Settings,
    input::intents::{Action, Actions},
    level::LevelLoad,
    level::definition::{self as map, DATA},
    level::mission::Mission,
    level::mission::inventory::CoinRun,
    level::navigation::Navigation,
    level::objects::shooting_range::ShootingRange,
    player::Player,
    player::camera::{GameCamera, ViewState},
    ui::{NameInput, UiState},
};
use bevy::{
    camera::RenderTarget,
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
        mouse::AccumulatedMouseMotion,
    },
    input_focus::{FocusCause, InputFocus},
    prelude::*,
    render::{
        render_resource::{TextureFormat, TextureUsages},
        view::screenshot::{Screenshot, save_to_disk},
    },
    text::{EditableText, TextEdit},
    ui::Pressed,
    window::CursorOptions,
};

#[derive(Default)]
pub struct Smoke {
    stage: usize,
    frames: u32,
    total: u32,
    position: Vec2,
    yaw: f32,
    coin: usize,
    occlusion_checked: bool,
    close_checked: bool,
}

#[derive(Resource)]
pub struct OffscreenImage(Handle<Image>);

pub fn setup_offscreen(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut cameras: Query<&mut RenderTarget, With<Camera>>,
) {
    let mut image = Image::new_target_texture(1440, 900, TextureFormat::Rgba8UnormSrgb, None);
    image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    let handle = images.add(image);
    for mut target in &mut cameras {
        *target = RenderTarget::Image(handle.clone().into());
    }
    commands.insert_resource(OffscreenImage(handle));
}

fn capture(world: &mut World, name: &str) {
    let path = format!("{}/artifacts/{name}.png", env!("CARGO_MANIFEST_DIR"));
    let screenshot = world
        .get_resource::<OffscreenImage>()
        .map_or_else(Screenshot::primary_window, |target| {
            Screenshot::image(target.0.clone())
        });
    world.spawn(screenshot).observe(save_to_disk(path));
}

fn press(world: &mut World, key: KeyCode) {
    world.resource_mut::<ButtonInput<KeyCode>>().press(key);
}
fn release(world: &mut World) {
    world.resource_mut::<ButtonInput<KeyCode>>().release_all();
    world
        .resource_mut::<ButtonInput<MouseButton>>()
        .release_all();
    let buttons: Vec<_> = world
        .query_filtered::<Entity, (With<Action>, With<Pressed>)>()
        .iter(world)
        .collect();
    for button in buttons {
        world.entity_mut(button).remove::<Pressed>();
    }
}

fn click_world(world: &mut World, point: Vec3) {
    let cursor = {
        let mut query = world.query_filtered::<(&Camera, &GlobalTransform), With<GameCamera>>();
        let (camera, transform) = query.single(world).unwrap();
        camera.world_to_viewport(transform, point).unwrap()
    };
    world
        .query::<&mut Window>()
        .single_mut(world)
        .unwrap()
        .set_cursor_position(Some(cursor));
    world
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
}

fn route_to(world: &mut World, point: Vec3) {
    let position = world.resource::<Player>().feet();
    let route = world
        .resource::<Navigation>()
        .path(position, point)
        .expect("reachable objective");
    world.resource_mut::<Player>().follow_route(route);
}

fn aim_at(world: &mut World, point: Vec3) {
    // Test teleports bypass Update. Resolve the real head point before aiming.
    world
        .run_system_cached(crate::player::update_model)
        .unwrap();
    world
        .run_system_cached(crate::player::camera::update_camera)
        .unwrap();
    let eye = world
        .resource::<ViewState>()
        .eye()
        .expect("player head ready");
    let direction = (point - eye.translation).normalize();
    world
        .resource_mut::<Player>()
        .face_yaw((-direction.x).atan2(-direction.z));
    world
        .resource_mut::<ViewState>()
        .set_pitch(direction.y.asin());
}

fn fire(world: &mut World) {
    let mut window = world.query::<&mut Window>().single_mut(world).unwrap();
    let center = Vec2::new(window.width(), window.height()) * 0.5;
    window.set_cursor_position(Some(center));
    world
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
}

pub fn step(world: &mut World, mut test: Local<Smoke>) {
    // Offscreen input includes focus; the native test follows real window focus.
    if world.contains_resource::<OffscreenImage>() {
        world
            .query::<&mut Window>()
            .single_mut(world)
            .unwrap()
            .focused = true;
    }
    test.frames += 1;
    test.total += 1;
    assert!(
        test.total < 18000,
        "rendered smoke test timed out at stage {}",
        test.stage
    );
    assert!(
        *world.resource::<LevelLoad>() != LevelLoad::Failed,
        "asset loading failed"
    );
    if test.stage >= 100 {
        if objects::step(world, test.stage, test.frames) {
            test.stage += 1;
            test.frames = 0;
        }
        return;
    }
    let next = match test.stage {
        0 if world.resource::<LevelLoad>().is_ready()
            && test.frames > 90
            && world.query::<&Window>().single(world).unwrap().focused =>
        {
            assert!(
                world.query::<&Mesh3d>().iter(world).count() > 100,
                "glTF scenes must instantiate"
            );
            if std::env::args()
                .any(|arg| matches!(arg.as_str(), "--smoke-objects" | "--smoke-stations"))
            {
                world
                    .resource_mut::<Player>()
                    .respawn(map::point(4., 51.6, 0.).xz(), 0.);
                *world.resource_mut::<ViewState>() =
                    ViewState::new(crate::player::camera::ViewMode::FirstPerson);
                world
                    .resource_mut::<crate::input::PointerCapture>()
                    .set(true);
                aim_at(world, map::point(4., 50., 1.));
                test.stage = 100;
                test.frames = 0;
                return;
            }
            if std::env::args().any(|arg| arg == "--smoke-range") {
                world
                    .resource_mut::<Player>()
                    .respawn(map::RANGE_ENTRY.xz(), 0.);
                world.resource_mut::<Actions>().0.push(Action::View);
                test.stage = 17;
                test.frames = 0;
                return;
            }
            capture(world, "overview");
            press(world, KeyCode::ArrowUp);
            true
        }
        1 if test.frames > 20 => {
            let focused = world.query::<&Window>().single(world).unwrap().focused;
            let held = world
                .resource::<ButtonInput<KeyCode>>()
                .pressed(KeyCode::ArrowUp);
            assert!(
                world.resource::<Player>().position().y < map::START.z - 1.0,
                "arrow-key movement: position={:?}, window_focused={focused}, held={held}, ui_focus={:?}, menu={}",
                world.resource::<Player>().position(),
                world.resource::<InputFocus>().get(),
                world.resource::<crate::app::flow::GameFlow>().menu()
            );
            release(world);
            click_world(world, DATA.coins()[0].into());
            true
        }
        2 if test.frames > 2 => {
            release(world);
            assert!(
                !world.resource::<Player>().route().is_empty(),
                "screen ray must start a route"
            );
            capture(world, "pathfinding");
            true
        }
        3 if world.resource::<Player>().route().is_empty() => {
            assert!(
                world.resource::<CoinRun>().collected()[0],
                "click route collects coin"
            );
            test.position = world.resource::<Player>().position();
            let entity = world
                .query_filtered::<Entity, With<NameInput>>()
                .single(world)
                .unwrap();
            world
                .resource_mut::<InputFocus>()
                .set(entity, FocusCause::Pressed);
            world
                .get_mut::<EditableText>(entity)
                .unwrap()
                .queue_edit(TextEdit::SelectAll);
            true
        }
        4 if test.frames > 3 => {
            let window = world
                .query_filtered::<Entity, With<Window>>()
                .single(world)
                .unwrap();
            world.write_message(KeyboardInput {
                key_code: KeyCode::KeyA,
                logical_key: Key::Character("Jäger ÄÖÜ".into()),
                state: ButtonState::Pressed,
                text: Some("Jäger ÄÖÜ".into()),
                repeat: false,
                window,
            });
            press(world, KeyCode::ArrowUp);
            press(world, KeyCode::Space);
            press(world, KeyCode::KeyE);
            press(world, KeyCode::KeyV);
            true
        }
        5 if test.frames > 10 => {
            assert_eq!(
                world.resource::<Player>().position(),
                test.position,
                "typing must block movement"
            );
            assert!(
                !world.resource::<ViewState>().first_person(),
                "typing must block shortcuts"
            );
            assert_eq!(
                world.resource::<Player>().height(),
                0.0,
                "typing cannot jump"
            );
            assert_eq!(world.resource::<Mission>().parts(), [false; 2]);
            assert_eq!(
                world.resource::<Settings>().player_name(),
                "Jäger ÄÖÜ",
                "Unicode keyboard input"
            );
            release(world);
            capture(world, "text-input");
            true
        }
        6 if test.frames == 10 => {
            press(world, KeyCode::Enter);
            false
        }
        6 if test.frames > 13 => {
            release(world);
            assert!(
                world.resource::<InputFocus>().get().is_none(),
                "Enter releases text focus"
            );
            // Drive the actual UI interaction handler, not just the action queue.
            let entity = world
                .query::<(Entity, &Action)>()
                .iter(world)
                .find(|(_, action)| matches!(action, Action::Settings))
                .unwrap()
                .0;
            world.entity_mut(entity).insert(Pressed);
            true
        }
        7 if test.frames > 3 => {
            assert!(
                world.resource::<crate::app::flow::GameFlow>().menu(),
                "settings button opens menu"
            );
            test.position = world.resource::<Player>().position();
            press(world, KeyCode::ArrowUp);
            world.resource_mut::<Actions>().0.extend([
                Action::Sensitivity(0.4),
                Action::Fov(10.0),
                Action::Shadows,
            ]);
            true
        }
        8 if test.frames > 10 => {
            assert_eq!(
                world.resource::<Player>().position(),
                test.position,
                "menu pauses movement"
            );
            let saved = Settings::load();
            assert!((saved.sensitivity() - 1.4).abs() < 0.001);
            assert_eq!(saved.fov(), 85.0);
            assert!(!saved.shadows());
            capture(world, "settings");
            release(world);
            true
        }
        9 => notes::step(world, test.frames, test.position),
        10 if test.frames > 3 => {
            assert!(
                world.resource::<ViewState>().first_person()
                    && world.resource::<crate::input::PointerCapture>().active()
            );
            assert!(
                !world
                    .query::<&CursorOptions>()
                    .single(world)
                    .unwrap()
                    .visible
            );
            test.position = world.resource::<Player>().position();
            test.yaw = world.resource::<Player>().yaw();
            world.resource_mut::<AccumulatedMouseMotion>().delta = Vec2::new(60.0, -15.0);
            press(world, KeyCode::KeyW);
            true
        }
        11 if test.frames > 15 => {
            assert!(
                world
                    .resource::<Player>()
                    .position()
                    .distance(test.position)
                    > 0.5,
                "WASD movement"
            );
            assert!(
                (world.resource::<Player>().yaw() - test.yaw).abs() > 0.1,
                "mouse look"
            );
            capture(world, "first-person");
            release(world);
            true
        }
        12 if test.frames == 10 => {
            press(world, KeyCode::Escape);
            false
        }
        12 if test.frames > 13 => {
            assert!(
                !world.resource::<crate::input::PointerCapture>().active(),
                "Escape releases pointer"
            );
            assert!(
                world
                    .query::<&CursorOptions>()
                    .single(world)
                    .unwrap()
                    .visible
            );
            release(world);
            world.resource_mut::<Actions>().0.push(Action::Restart);
            true
        }
        13 if test.frames > 3 => {
            assert_eq!(world.resource::<Player>().position(), map::START.xz());
            assert_eq!(world.resource::<CoinRun>().count(), 0);
            assert!(!world.resource::<ViewState>().first_person());
            true
        }
        14 => {
            release(world);
            press(world, KeyCode::ShiftLeft);
            if test.frames.is_multiple_of(400) {
                info!(
                    "Task route: player {:?}, destination {:?}, {} waypoints, {}",
                    world.resource::<Player>().feet(),
                    world.resource::<Mission>().destination(),
                    world.resource::<Player>().route().len(),
                    world.resource::<UiState>().status()
                );
            }
            if world.resource::<Mission>().finished() {
                assert_eq!(world.resource::<Mission>().installed_cells(), 3);
                capture(world, "signal-finished");
                true
            } else {
                if world.resource::<Player>().route().is_empty() && test.frames.is_multiple_of(5) {
                    let goal = world.resource::<Mission>().destination();
                    if world.resource::<Player>().feet().distance(goal) < 1.5 {
                        world.resource_mut::<Actions>().0.push(Action::Interact);
                    } else {
                        route_to(world, goal);
                    }
                }
                false
            }
        }
        15 => {
            release(world);
            press(world, KeyCode::ShiftLeft);
            if test.frames.is_multiple_of(400) {
                info!(
                    "Blueprint {}: player {:?}, {} waypoints",
                    test.coin,
                    world.resource::<Player>().feet(),
                    world.resource::<Player>().route().len()
                );
            }
            if test.coin >= 5 {
                test.coin = 0;
                true
            } else {
                if world.resource::<Mission>().blueprints()[test.coin] {
                    test.coin += 1;
                } else if world.resource::<Player>().route().is_empty() {
                    route_to(world, DATA.blueprints()[test.coin].into());
                }
                false
            }
        }
        16 if test.frames == 1 => {
            route_to(world, map::RANGE_ENTRY);
            false
        }
        16 if world.resource::<Player>().route().is_empty() => {
            world.resource_mut::<Actions>().0.push(Action::View);
            true
        }
        17 => {
            release(world);
            if !test.occlusion_checked {
                if test.frames == 5 {
                    aim_at(world, level::objects::shooting_range::TARGETS[2]);
                    fire(world);
                }
                if test.frames >= 10 {
                    assert_eq!(world.resource::<ShootingRange>().shots(), 1);
                    assert_eq!(
                        world.resource::<ShootingRange>().hits(),
                        0,
                        "the bridge support must block this shot"
                    );
                    test.occlusion_checked = true;
                    test.frames = 0;
                }
                return;
            }
            // Shoot the middle target from up close, then physically walk back.
            // No activation interaction is sent at any stage of this challenge.
            if !test.close_checked {
                world.resource_mut::<Player>().face_yaw(0.);
                world.resource_mut::<ViewState>().set_pitch(0.);
                let distance = world
                    .resource::<Player>()
                    .position()
                    .distance(level::objects::shooting_range::TARGETS[1].xz());
                if distance > 3. && test.frames <= 1 {
                    press(world, KeyCode::KeyW);
                    test.frames = 0;
                } else if test.frames == 5 {
                    aim_at(world, level::objects::shooting_range::TARGETS[1]);
                    fire(world);
                } else if test.frames >= 10 {
                    assert_eq!(
                        world.resource::<ShootingRange>().hits(),
                        0,
                        "close shot cannot destroy target"
                    );
                    assert_eq!(world.resource::<ShootingRange>().shots(), 2);
                    assert!(world.resource::<UiState>().status().starts_with("Zu nah:"));
                    assert!(!world.resource::<Mission>().practice_reward());
                    capture(world, "range-too-close");
                    test.close_checked = true;
                    test.frames = 0;
                }
                return;
            }
            if world.resource::<Player>().position().y < map::RANGE_ENTRY.z - 0.15 {
                world.resource_mut::<Player>().face_yaw(0.);
                world.resource_mut::<ViewState>().set_pitch(0.);
                press(world, KeyCode::KeyS);
                test.frames = 0;
                return;
            }
            if test.frames == 3 && test.coin == 0 {
                capture(world, "range-ready");
            }
            if test.coin >= 3 {
                assert_eq!(world.resource::<ShootingRange>().hits(), 3);
                assert!(world.resource::<Mission>().practice_reward());
                capture(world, "shooting_range");
                if std::env::args().any(|arg| arg == "--smoke-range") {
                    test.stage = 98;
                }
                world.resource_mut::<Actions>().0.push(Action::View);
                true
            } else if (level::objects::shooting_range::TARGETS[test.coin].x
                - world.resource::<Player>().position().x)
                .abs()
                > 0.15
            {
                let dx = level::objects::shooting_range::TARGETS[test.coin].x
                    - world.resource::<Player>().position().x;
                world.resource_mut::<Player>().face_yaw(0.);
                world.resource_mut::<ViewState>().set_pitch(0.);
                press(
                    world,
                    if dx > 0. {
                        KeyCode::KeyD
                    } else {
                        KeyCode::KeyA
                    },
                );
                test.frames = 0;
                false
            } else if test.frames == 5 {
                aim_at(world, level::objects::shooting_range::TARGETS[test.coin]);
                fire(world);
                false
            } else if test.frames == 10 {
                assert!(
                    world.resource::<ShootingRange>().targets_hit()[test.coin],
                    "loaded target {} raycast; shots={}, status={}",
                    test.coin,
                    world.resource::<ShootingRange>().shots(),
                    world.resource::<UiState>().status()
                );
                test.coin += 1;
                test.frames = 0;
                false
            } else {
                false
            }
        }
        18 if test.frames == 1 => {
            release(world);
            route_to(world, map::point(46., 49.5, 0.));
            false
        }
        18 if world.resource::<Player>().route().is_empty() => {
            world.resource_mut::<Actions>().0.push(Action::Interact);
            true
        }
        19 if test.frames > 5 => {
            assert_eq!(world.resource::<Mission>().blueprint_count(), 6);
            route_to(world, map::CHECKPOINT);
            true
        }
        20 if world.resource::<Player>().route().is_empty() => {
            world.resource_mut::<Actions>().0.push(Action::Interact);
            true
        }
        21 if test.frames > 5 => {
            assert!(world.resource::<Mission>().checkpoint());
            route_to(world, map::point(37., 33., 0.));
            true
        }
        22 if world.resource::<Player>().route().is_empty() => {
            assert!(
                world.resource::<Player>().height() < 0.1,
                "walk underneath the central gallery"
            );
            capture(world, "under-gallery");
            route_to(world, map::point(37., 33., 3.));
            true
        }
        23 if world.resource::<Player>().route().is_empty() => {
            assert!(
                (world.resource::<Player>().height() - 3.).abs() < 0.01,
                "reach the same position upstairs via stairs"
            );
            capture(world, "upper-gallery");
            world.resource_mut::<Actions>().0.push(Action::Map);
            true
        }
        24 if test.frames > 40 => {
            capture(world, "playable-level");
            true
        }
        25 if test.frames > 15 => {
            world.resource_mut::<Actions>().0.push(Action::Checkpoint);
            true
        }
        26 if test.frames > 5 => {
            assert!(world.resource::<Player>().feet().distance(map::CHECKPOINT) < 0.01);
            assert_eq!(world.resource::<Mission>().installed_cells(), 3);
            assert_eq!(world.resource::<Mission>().blueprint_count(), 6);
            world.resource_mut::<Actions>().0.push(Action::Restart);
            true
        }
        27 if test.frames > 5 => {
            let m = world.resource::<Mission>();
            assert!(
                !m.finished()
                    && !m.rack_open()
                    && m.collected_cells() == 0
                    && m.blueprint_count() == 0
            );
            assert_eq!(world.resource::<CoinRun>().count(), 0);
            assert_eq!(world.resource::<ShootingRange>().targets_hit(), [false; 3]);
            assert_eq!(world.resource::<Player>().position(), map::START.xz());
            let nav = world.resource::<Navigation>();
            let rack = nav
                .obstacles()
                .iter()
                .find(|o| o.name() == "Movable storage rack")
                .unwrap();
            let original = DATA
                .obstacles()
                .iter()
                .find(|o| o.name() == rack.name())
                .unwrap();
            assert_eq!(
                rack.center(),
                original.center(),
                "reset must restore the shelf collider"
            );
            info!(
                "SMOKE TEST PASSED: inputs, settings, all room tasks, six blueprints, shooting, both floors and reset"
            );
            world.write_message(AppExit::Success);
            false
        }
        99 if test.frames > 15 => {
            info!("RANGE SMOKE TEST PASSED");
            world.write_message(AppExit::Success);
            false
        }
        _ => false,
    };
    if next {
        info!("Smoke stage {} passed", test.stage);
        test.stage += 1;
        test.frames = 0;
    }
}
pub fn prepare_artifacts() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("artifacts");
    std::fs::create_dir_all(dir).expect("smoke artifact directory");
}
