//! Optional rendered integration check. Exercises the same systems as normal gameplay.
use crate::{
    camera::{GameCamera, ViewState},
    coins::CoinRun,
    level::{COINS, LevelLoad, START, ground},
    navigation::Navigation,
    player::Player,
    settings::Settings,
    shooting::{self, Debris, Practice, Weapon},
    ui::{Action, Actions, NameInput, UiState},
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
    remaining: f32,
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
}

fn click_world(world: &mut World, point: Vec2) {
    let cursor = {
        let mut query = world.query_filtered::<(&Camera, &GlobalTransform), With<GameCamera>>();
        let (camera, transform) = query.single(world).unwrap();
        camera
            .world_to_viewport(transform, ground(point, 0.0))
            .unwrap()
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

fn route_to(world: &mut World, point: Vec2) {
    let position = world.resource::<Player>().position;
    let route = world
        .resource::<Navigation>()
        .path(position, point)
        .expect("reachable objective");
    world.resource_mut::<Player>().route = route;
}

fn aim_at(world: &mut World, point: Vec3) {
    let position = world.resource::<Player>().position;
    let direction = (point - ground(position, 1.5)).normalize();
    world.resource_mut::<Player>().yaw = (-direction.x).atan2(-direction.z);
    world.resource_mut::<ViewState>().pitch = direction.y.asin();
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
        test.total < 9000,
        "rendered smoke test timed out at stage {}",
        test.stage
    );
    assert!(
        *world.resource::<LevelLoad>() != LevelLoad::Failed,
        "asset loading failed"
    );
    let next = match test.stage {
        0 if world.resource::<LevelLoad>().is_ready()
            && test.frames > 90
            && world.query::<&Window>().single(world).unwrap().focused =>
        {
            assert!(
                world.query::<&Mesh3d>().iter(world).count() > 100,
                "glTF scenes must instantiate"
            );
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
                world.resource::<Player>().position.y < START.y - 1.0,
                "arrow-key movement: position={:?}, window_focused={focused}, held={held}, ui_focus={:?}, menu={}",
                world.resource::<Player>().position,
                world.resource::<InputFocus>().get(),
                world.resource::<UiState>().menu
            );
            release(world);
            click_world(world, COINS[0]);
            true
        }
        2 if test.frames > 2 => {
            release(world);
            assert!(
                !world.resource::<Player>().route.is_empty(),
                "screen ray must start a route"
            );
            capture(world, "pathfinding");
            true
        }
        3 if world.resource::<Player>().route.is_empty() => {
            assert!(
                world.resource::<CoinRun>().collected[0],
                "click route collects coin"
            );
            test.position = world.resource::<Player>().position;
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
            press(world, KeyCode::KeyV);
            true
        }
        5 if test.frames > 10 => {
            assert_eq!(
                world.resource::<Player>().position,
                test.position,
                "typing must block movement"
            );
            assert!(
                !world.resource::<ViewState>().first_person,
                "typing must block shortcuts"
            );
            assert_eq!(
                world.resource::<Settings>().player_name,
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
            *world.get_mut::<Interaction>(entity).unwrap() = Interaction::Pressed;
            true
        }
        7 if test.frames > 3 => {
            assert!(
                world.resource::<UiState>().menu,
                "settings button opens menu"
            );
            test.position = world.resource::<Player>().position;
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
                world.resource::<Player>().position,
                test.position,
                "menu pauses movement"
            );
            let saved = Settings::load();
            assert!((saved.sensitivity - 1.4).abs() < 0.001);
            assert_eq!(saved.fov, 85.0);
            assert!(!saved.shadows);
            capture(world, "settings");
            release(world);
            true
        }
        9 if test.frames == 10 => {
            world.resource_mut::<Actions>().0.push(Action::Resume);
            false
        }
        9 if test.frames > 20 => {
            world.resource_mut::<Player>().yaw = 0.0;
            world.resource_mut::<Actions>().0.push(Action::View);
            true
        }
        10 if test.frames > 3 => {
            assert!(
                world.resource::<ViewState>().first_person
                    && world.resource::<ViewState>().captured
            );
            assert!(
                !world
                    .query::<&CursorOptions>()
                    .single(world)
                    .unwrap()
                    .visible
            );
            test.position = world.resource::<Player>().position;
            test.yaw = world.resource::<Player>().yaw;
            world.resource_mut::<AccumulatedMouseMotion>().delta = Vec2::new(60.0, -15.0);
            press(world, KeyCode::KeyW);
            true
        }
        11 if test.frames > 15 => {
            assert!(
                world.resource::<Player>().position.distance(test.position) > 0.5,
                "WASD movement"
            );
            assert!(
                (world.resource::<Player>().yaw - test.yaw).abs() > 0.1,
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
                !world.resource::<ViewState>().captured,
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
            assert_eq!(world.resource::<Player>().position, START);
            assert_eq!(world.resource::<CoinRun>().count(), 0);
            assert!(!world.resource::<ViewState>().first_person);
            assert!(
                world.resource::<LevelLoad>().is_ready(),
                "restart keeps loaded assets ready"
            );
            assert_eq!(
                world.resource::<Settings>().fov,
                85.0,
                "restart preserves settings"
            );
            test.coin = 0;
            route_to(world, COINS[0]);
            true
        }
        14 if world.resource::<Player>().route.is_empty() => {
            assert!(
                world.resource::<CoinRun>().collected[test.coin],
                "autopilot collects objective"
            );
            test.coin += 1;
            if test.coin < COINS.len() {
                route_to(world, COINS[test.coin]);
                false
            } else {
                route_to(world, START);
                true
            }
        }
        15 if world.resource::<CoinRun>().finished => {
            assert_eq!(world.resource::<CoinRun>().count(), 6);
            capture(world, "completed");
            true
        }
        16 if test.frames > 45 => {
            world.resource_mut::<Actions>().0.push(Action::Range);
            true
        }
        17 if test.frames > 3 && world.resource::<Player>().route.is_empty() => {
            assert!(
                world
                    .resource::<Player>()
                    .position
                    .distance(shooting::ENTRY)
                    < 0.1,
                "range remains reachable after completing the coin game"
            );
            capture(world, "range-overview");
            true
        }
        18 if test.frames > 12 => {
            press(world, KeyCode::KeyE);
            true
        }
        19 if test.frames > 5 => {
            release(world);
            assert!(world.resource::<Practice>().running);
            assert!(
                world.resource::<CoinRun>().finished,
                "target practice preserves the completed coin round"
            );
            assert!(
                world.resource::<ViewState>().first_person
                    && world.resource::<ViewState>().captured
            );
            assert_eq!(
                *world
                    .query_filtered::<&Visibility, With<Weapon>>()
                    .single(world)
                    .unwrap(),
                Visibility::Visible
            );
            aim_at(world, shooting::TARGETS[1]);
            capture(world, "range-ready");
            true
        }
        20 if test.frames > 12 => {
            fire(world);
            true
        }
        21 if test.frames > 2 => {
            release(world);
            let practice = world.resource::<Practice>();
            assert_eq!(practice.shots, 1, "one click fires one shot");
            assert_eq!(
                practice.hits, 1,
                "ray hits the glTF target through the crosshair"
            );
            assert_eq!(practice.score, 50, "bullseye scores 50");
            assert!(practice.respawn[1] > 0.0);
            assert_eq!(world.query::<&Debris>().iter(world).count(), 6);
            capture(world, "range-hit");
            true
        }
        22 if test.frames > 30 => {
            assert_eq!(
                world.resource::<Practice>().respawn[1],
                0.0,
                "target respawns"
            );
            aim_at(world, shooting::TARGETS[1] + Vec3::X * 0.6);
            true
        }
        23 if test.frames > 3 => {
            fire(world);
            true
        }
        24 if test.frames > 2 => {
            release(world);
            assert_eq!(
                world.resource::<Practice>().score,
                60,
                "outer ring scores ten"
            );
            aim_at(world, shooting::TARGETS[2]);
            true
        }
        25 if test.frames > 5 => {
            fire(world);
            true
        }
        26 if test.frames > 2 => {
            release(world);
            assert_eq!(world.resource::<Practice>().shots, 3);
            assert_eq!(
                world.resource::<Practice>().hits,
                2,
                "lane divider blocks the side target"
            );
            world.resource_mut::<Actions>().0.push(Action::Settings);
            test.remaining = world.resource::<Practice>().remaining;
            true
        }
        27 if test.frames > 12 => {
            assert_eq!(
                world.resource::<Practice>().remaining,
                test.remaining,
                "menu pauses the round"
            );
            fire(world);
            true
        }
        28 if test.frames > 3 => {
            assert_eq!(
                world.resource::<Practice>().shots,
                3,
                "menu click does not shoot"
            );
            release(world);
            world.resource_mut::<Actions>().0.push(Action::Resume);
            aim_at(world, shooting::TARGETS[1]);
            fire(world);
            true
        }
        29 if test.frames == 4 => {
            release(world);
            assert_eq!(
                world.resource::<Practice>().shots,
                3,
                "the click that resumes a round cannot fire"
            );
            false
        }
        29 if !world.resource::<Practice>().running => {
            let practice = world.resource::<Practice>();
            assert_eq!(practice.remaining, 0.0);
            assert_eq!(practice.best, 60);
            assert_eq!(practice.hits, 2);
            fire(world);
            capture(world, "range-result");
            true
        }
        30 if test.frames > 12 => {
            release(world);
            assert_eq!(
                world.resource::<Practice>().shots,
                3,
                "expired rounds cannot score"
            );
            press(world, KeyCode::KeyE);
            true
        }
        31 if test.frames > 3 => {
            release(world);
            assert_eq!(world.resource::<Practice>().score, 0);
            assert_eq!(world.resource::<Practice>().best, 60);
            assert!(world.resource::<Practice>().running);
            true
        }
        32 if test.frames > 20 => {
            for name in [
                "overview",
                "pathfinding",
                "text-input",
                "settings",
                "first-person",
                "completed",
                "range-overview",
                "range-ready",
                "range-hit",
                "range-result",
            ] {
                assert!(
                    std::path::Path::new(&format!(
                        "{}/artifacts/{name}.png",
                        env!("CARGO_MANIFEST_DIR")
                    ))
                    .exists()
                );
            }
            info!(
                "SMOKE TEST PASSED: movement, text, settings, coin game, Blender blaster, range access, bullseye, outer ring, occlusion, fragments, respawn, pause, timed round and retry"
            );
            world.write_message(AppExit::Success);
            true
        }
        _ => false,
    };
    if next {
        info!("Smoke stage {} passed", test.stage);
        test.stage += 1;
        test.frames = 0;
    }
}
