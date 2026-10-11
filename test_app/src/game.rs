use crate::{
    level::{self, COINS, Coin, START, ground},
    navigation::Navigation,
    settings::Settings,
    shooting::{self, Practice},
    ui::{Action, Actions, UiBlock},
};
use bevy::{
    input::mouse::AccumulatedMouseMotion,
    input_focus::InputFocus,
    prelude::*,
    text::EditableText,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use std::collections::VecDeque;

#[derive(Component)]
pub struct PlayerModel;
#[derive(Component)]
pub struct GameCamera;

#[derive(Resource)]
pub struct Game {
    pub position: Vec2,
    pub yaw: f32,
    pub pitch: f32,
    pub first_person: bool,
    pub captured: bool,
    pub menu: bool,
    pub ready: bool,
    pub asset_error: bool,
    pub route: VecDeque<Vec2>,
    pub collected: [bool; 6],
    pub elapsed: f32,
    pub finished: bool,
    pub status: String,
    pub walked: f32,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            position: START,
            yaw: 0.0,
            pitch: 0.0,
            first_person: false,
            captured: false,
            menu: false,
            ready: false,
            asset_error: false,
            route: VecDeque::new(),
            collected: [false; 6],
            elapsed: 0.0,
            finished: false,
            status: "Lade den Parcours …".into(),
            walked: 0.0,
        }
    }
}

impl Game {
    pub fn count(&self) -> usize {
        self.collected.iter().filter(|&&c| c).count()
    }

    pub fn toggle_view(&mut self) {
        self.first_person = !self.first_person;
        self.captured = self.first_person;
        self.pitch = 0.0;
        self.route.clear();
    }

    pub fn reset(&mut self) {
        let ready = self.ready;
        let asset_error = self.asset_error;
        *self = Self {
            ready,
            asset_error,
            status: "Neue Runde. Sammle alle sechs Münzen.".into(),
            ..default()
        };
    }
}

pub fn handle_actions(
    mut actions: ResMut<Actions>,
    mut game: ResMut<Game>,
    mut settings: ResMut<Settings>,
    mut focus: ResMut<InputFocus>,
    mut exit: MessageWriter<AppExit>,
    nav: Res<Navigation>,
    mut practice: ResMut<Practice>,
) {
    for action in actions.0.drain(..) {
        focus.clear();
        match action {
            Action::View => game.toggle_view(),
            Action::Settings => {
                game.menu = true;
                game.captured = false;
                game.route.clear();
            }
            Action::Resume => {
                game.menu = false;
                game.captured = game.first_person;
            }
            Action::Restart => {
                game.reset();
                practice.reset();
            }
            Action::Range => {
                if let Some(route) = nav.path(game.position, shooting::ENTRY) {
                    game.first_person = false;
                    game.captured = false;
                    game.route = route;
                    game.status = "Unterwegs zum Schießstand. Dort startet E die Zielübung.".into();
                }
            }
            Action::Practice => {
                if !game.ready {
                    continue;
                }
                if shooting::in_range(game.position) {
                    practice.start();
                    game.first_person = true;
                    game.captured = true;
                    game.yaw = 0.0;
                    game.pitch = 0.0;
                    game.route.clear();
                    game.status =
                        "30 Sekunden! Linksklick schießt. Die Mitte zählt 50 Punkte.".into();
                } else {
                    game.status =
                        "Betritt zuerst den Schießstand im Norden. Der Button bringt dich hin."
                            .into();
                }
            }
            Action::Sensitivity(delta) => {
                settings.sensitivity = (settings.sensitivity + delta).clamp(0.2, 3.0)
            }
            Action::Speed(delta) => settings.speed = (settings.speed + delta).clamp(2.0, 8.0),
            Action::Fov(delta) => settings.fov = (settings.fov + delta).clamp(55.0, 105.0),
            Action::Path => settings.show_path = !settings.show_path,
            Action::Shadows => settings.shadows = !settings.shadows,
            Action::Invert => settings.invert_y = !settings.invert_y,
            Action::Defaults => {
                let name = settings.player_name.clone();
                *settings = Settings {
                    player_name: name,
                    ..default()
                };
            }
            Action::Quit => {
                exit.write(AppExit::Success);
            }
        }
        if let Err(error) = settings.save() {
            game.status = format!("Einstellungen konnten nicht gespeichert werden: {error}");
        }
    }
}

pub fn keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut focus: ResMut<InputFocus>,
    inputs: Query<(), With<EditableText>>,
    mut game: ResMut<Game>,
    mut actions: ResMut<Actions>,
) {
    if !window.focused {
        game.captured = false;
        return;
    }
    let editing = focus.get().is_some_and(|e| inputs.contains(e));
    if editing {
        game.captured = false;
        if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::Enter) {
            focus.clear();
        }
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        focus.clear();
        if game.menu {
            actions.0.push(Action::Resume);
        } else if game.captured {
            game.captured = false;
        } else {
            actions.0.push(Action::Settings);
        }
        return;
    }
    if !game.menu && keys.just_pressed(KeyCode::KeyV) {
        actions.0.push(Action::View);
    }
    if !game.menu && keys.just_pressed(KeyCode::KeyM) {
        actions.0.push(Action::Settings);
    }
    if !game.menu && keys.just_pressed(KeyCode::KeyR) {
        actions.0.push(Action::Restart);
    }
    if !game.menu && keys.just_pressed(KeyCode::KeyE) {
        actions.0.push(Action::Practice);
    }
}

/// Hit test the actual UI layout, including display scale and window resizing.
pub fn pointer_over_ui(
    window: &Window,
    blocks: &Query<(&ComputedNode, &UiGlobalTransform), With<UiBlock>>,
) -> bool {
    let Some(pointer) = window.cursor_position() else {
        return false;
    };
    let pointer = pointer * window.scale_factor();
    blocks.iter().any(|(node, transform)| {
        node.size().min_element() > 0.0
            && transform.try_inverse().is_some_and(|inverse| {
                let local = inverse.transform_point2(pointer);
                (local.abs().cmple(node.size() * 0.5)).all()
            })
    })
}

pub fn click_to_move(
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<GameCamera>>,
    blocks: Query<(&ComputedNode, &UiGlobalTransform), With<UiBlock>>,
    nav: Res<Navigation>,
    mut game: ResMut<Game>,
    mut focus: ResMut<InputFocus>,
) {
    if !buttons.just_pressed(MouseButton::Left)
        || !window.focused
        || game.menu
        || !game.ready
        || pointer_over_ui(&window, &blocks)
    {
        return;
    }
    focus.clear();
    if game.first_person {
        game.captured = true;
        return;
    }
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(ray) = camera.0.viewport_to_world(camera.1, cursor) else {
        return;
    };
    let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else {
        return;
    };
    let target = ray.get_point(distance).xz();
    match nav.path(game.position, target) {
        Some(route) => {
            game.route = route;
            game.status = "Unterwegs. Pfeiltasten übernehmen jederzeit die Steuerung.".into();
        }
        None => {
            game.route.clear();
            game.status = "Dieses Ziel ist blockiert. Klicke auf freien Boden.".into();
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects the independent input and world resources."
)]
pub fn move_player(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<AccumulatedMouseMotion>,
    window: Single<&Window, With<PrimaryWindow>>,
    focus: Res<InputFocus>,
    inputs: Query<(), With<EditableText>>,
    nav: Res<Navigation>,
    settings: Res<Settings>,
    mut game: ResMut<Game>,
) {
    if !game.ready
        || game.menu
        || !window.focused
        || focus.get().is_some_and(|e| inputs.contains(e))
        || (game.first_person && !game.captured)
    {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    if !game.finished && !shooting::in_range(game.position) {
        game.elapsed += dt;
    }
    if game.first_person {
        game.yaw -= mouse.delta.x * 0.0025 * settings.sensitivity;
        game.pitch = (game.pitch
            - mouse.delta.y
                * 0.0025
                * settings.sensitivity
                * if settings.invert_y { -1.0 } else { 1.0 })
        .clamp(-1.45, 1.45);
    }
    let mut axis = Vec2::ZERO;
    if keys.pressed(KeyCode::ArrowLeft) || (game.first_person && keys.pressed(KeyCode::KeyA)) {
        axis.x -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) || (game.first_person && keys.pressed(KeyCode::KeyD)) {
        axis.x += 1.0;
    }
    if keys.pressed(KeyCode::ArrowUp) || (game.first_person && keys.pressed(KeyCode::KeyW)) {
        axis.y -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) || (game.first_person && keys.pressed(KeyCode::KeyS)) {
        axis.y += 1.0;
    }
    let speed = settings.speed
        * if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            1.65
        } else {
            1.0
        };
    let before = game.position;
    if axis != Vec2::ZERO {
        game.route.clear();
        let mut direction = axis.normalize();
        if game.first_person {
            direction = (Quat::from_rotation_y(game.yaw) * ground(direction, 0.0)).xz();
        }
        game.position = nav.move_and_slide(game.position, direction * speed * dt);
    } else {
        let mut budget = speed * dt;
        while let Some(&target) = game.route.front() {
            let delta = target - game.position;
            let distance = delta.length();
            if distance <= budget {
                game.position = target;
                game.route.pop_front();
                budget -= distance;
            } else {
                game.position = nav.move_and_slide(game.position, delta / distance * budget);
                break;
            }
            if budget <= 0.0 {
                break;
            }
        }
    }
    let travelled = game.position - before;
    game.walked += travelled.length();
    if !game.first_person && travelled.length_squared() > 0.000001 {
        game.yaw = (-travelled.x).atan2(-travelled.y);
    }
    for (index, coin) in COINS.into_iter().enumerate() {
        if !game.collected[index] && game.position.distance(coin) < 0.85 {
            game.collected[index] = true;
            game.status = if game.count() == COINS.len() {
                "Alle Münzen gefunden! Kehre zur grünen Startplattform zurück.".into()
            } else {
                format!("Münze gefunden. Noch {} übrig.", COINS.len() - game.count())
            };
        }
    }
    if !game.finished && game.count() == COINS.len() && game.position.distance(START) < 1.35 {
        game.finished = true;
        game.captured = false;
        game.route.clear();
        game.status = format!(
            "Geschafft, {}! Alle Münzen in {:.1} Sekunden. R startet eine neue Runde.",
            settings.player_name, game.elapsed
        );
    }
}

pub fn sync_cursor(
    game: Res<Game>,
    window: Single<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    let (window, mut cursor) = window.into_inner();
    let captured = game.first_person && game.captured && !game.menu && window.focused;
    cursor.visible = !captured;
    cursor.grab_mode = if captured {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
}

#[expect(
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "Bevy injects disjoint transform queries and render resources."
)]
pub fn update_scene(
    time: Res<Time>,
    game: Res<Game>,
    settings: Res<Settings>,
    mut camera: Single<
        (&mut Transform, &mut Projection),
        (With<GameCamera>, Without<PlayerModel>, Without<Coin>),
    >,
    mut player: Single<(&mut Transform, &mut Visibility), (With<PlayerModel>, Without<Coin>)>,
    mut coins: Query<(&Coin, &mut Transform, &mut Visibility), Without<PlayerModel>>,
    mut lights: Query<&mut DirectionalLight>,
    mut gizmos: Gizmos,
) {
    let (ref mut player_transform, ref mut visibility) = *player;
    player_transform.translation = ground(
        game.position,
        0.025 + (game.walked * 7.0).sin().abs() * 0.045,
    );
    // Dummy_Base faces local +Z; movement and the FPV camera use local -Z.
    player_transform.rotation = Quat::from_rotation_y(game.yaw + std::f32::consts::PI);
    **visibility = if game.first_person {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };
    let (ref mut camera_transform, ref mut projection) = *camera;
    if let Projection::Perspective(perspective) = &mut **projection {
        perspective.fov = if game.first_person {
            settings.fov
        } else {
            50.0
        }
        .to_radians();
    }
    if game.first_person {
        **camera_transform = Transform::from_translation(ground(game.position, 1.5))
            .with_rotation(Quat::from_euler(EulerRot::YXZ, game.yaw, game.pitch, 0.0));
    } else {
        // A fixed overview keeps world directions aligned with the arrow keys and the click target stable.
        **camera_transform = if shooting::in_range(game.position) {
            Transform::from_xyz(0.0, 21.0, -5.0).looking_at(Vec3::new(0.0, 0.0, -20.0), Vec3::Y)
        } else {
            Transform::from_xyz(0.0, 39.0, 24.0).looking_at(Vec3::new(0.0, 0.0, -5.0), Vec3::Y)
        };
    }
    for (coin, mut transform, mut visibility) in &mut coins {
        *visibility = if game.collected[coin.0] {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        transform.rotation = Quat::from_rotation_y(time.elapsed_secs() * 1.4 + coin.0 as f32);
        transform.translation.y = 1.0 + (time.elapsed_secs() * 2.0 + coin.0 as f32).sin() * 0.14;
    }
    for mut light in &mut lights {
        if light.shadow_maps_enabled != settings.shadows {
            light.shadow_maps_enabled = settings.shadows;
        }
    }
    if settings.show_path && !game.first_person && !game.route.is_empty() {
        gizmos.linestrip(
            std::iter::once(game.position)
                .chain(game.route.iter().copied())
                .map(|p| ground(p, 0.12)),
            level::TEAL,
        );
        if let Some(&goal) = game.route.back() {
            gizmos.circle(
                Isometry3d::new(
                    ground(goal, 0.12),
                    Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                ),
                0.4,
                level::TEAL,
            );
        }
    }
    if !game.first_person {
        gizmos.circle(
            Isometry3d::new(
                ground(game.position, 0.1),
                Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            ),
            0.65,
            level::TEAL,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::gizmos::{AppGizmoBuilder, config::DefaultGizmoConfigGroup};
    use std::time::Duration;

    fn check_facing(pathfinding: bool) {
        for direction in [
            Vec2::X,
            Vec2::NEG_X,
            Vec2::Y,
            Vec2::NEG_Y,
            Vec2::new(1.0, 1.0),
            Vec2::new(-1.0, 1.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(-1.0, -1.0),
        ]
        .map(Vec2::normalize)
        {
            let mut app = App::new();
            app.init_resource::<Time>()
                .init_resource::<ButtonInput<KeyCode>>()
                .init_resource::<AccumulatedMouseMotion>()
                .init_resource::<InputFocus>()
                .init_resource::<Navigation>()
                .init_resource::<Settings>()
                .insert_resource(Game {
                    ready: true,
                    ..default()
                })
                .init_gizmo_group::<DefaultGizmoConfigGroup>()
                .add_systems(Update, (move_player, update_scene).chain());
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::from_secs_f32(1.0 / 60.0));
            app.world_mut().spawn((
                Window {
                    focused: true,
                    ..default()
                },
                PrimaryWindow,
            ));
            let player = app
                .world_mut()
                .spawn((PlayerModel, Transform::default(), Visibility::default()))
                .id();
            let camera = app
                .world_mut()
                .spawn((GameCamera, Transform::default(), Projection::default()))
                .id();
            if pathfinding {
                let mut game = app.world_mut().resource_mut::<Game>();
                let target = game.position + direction * 3.0;
                game.route.push_back(target);
            } else {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                for (key, pressed) in [
                    (KeyCode::ArrowRight, direction.x > 0.0),
                    (KeyCode::ArrowLeft, direction.x < 0.0),
                    (KeyCode::ArrowDown, direction.y > 0.0),
                    (KeyCode::ArrowUp, direction.y < 0.0),
                ] {
                    if pressed {
                        keys.press(key);
                    }
                }
            }
            app.world_mut().run_schedule(Update);
            let expected = ground(direction, 0.0);
            // Dummy_Base's yellow face points along local +Z in the supplied glTF.
            let facing = app.world().get::<Transform>(player).unwrap().rotation * Vec3::Z;
            assert!(
                facing.dot(expected) > 0.999,
                "model faces {facing:?}, but moves {expected:?}; pathfinding={pathfinding}"
            );

            let before = app.world().resource::<Game>().position;
            app.world_mut().resource_mut::<Game>().toggle_view();
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release_all();
            keys.press(KeyCode::KeyW);
            app.world_mut().run_schedule(Update);
            let travelled = (app.world().resource::<Game>().position - before).normalize();
            assert!(
                travelled.dot(direction) > 0.999,
                "FPV W must preserve the character's heading"
            );
            let camera_forward = app.world().get::<Transform>(camera).unwrap().forward();
            assert!(
                camera_forward.dot(expected) > 0.999,
                "FPV must look where the character faces"
            );

            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .release_all();
            app.world_mut().resource_mut::<Game>().toggle_view();
            app.world_mut().run_schedule(Update);
            let facing = app.world().get::<Transform>(player).unwrap().rotation * Vec3::Z;
            assert!(
                facing.dot(expected) > 0.999,
                "returning to overview must preserve the heading"
            );
        }
    }

    #[test]
    fn character_faces_arrow_key_movement() {
        check_facing(false);
    }

    #[test]
    fn character_faces_pathfinding_movement() {
        check_facing(true);
    }
}
