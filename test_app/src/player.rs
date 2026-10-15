pub mod actions;

use crate::{
    camera::{GameCamera, ViewState},
    game::items::blueprint::START,
    game::level::{self, LevelLoad, ground},
    player::actions::navigation::Navigation,
    settings::Settings,
    ui::{UiBlock, UiState, pointer_over_ui},
};
use bevy::{input_focus::InputFocus, prelude::*, text::EditableText, window::PrimaryWindow};
use std::collections::VecDeque;

pub const MODEL_SCALE: f32 = 0.78;
pub const MODEL_BOB: f32 = 0.07;

pub fn limb_rotation(name: &str) -> Quat {
    match name {
        "Dummy_ArmLeft" => Quat::from_rotation_z(-70_f32.to_radians()),
        "Dummy_ArmRight" => Quat::from_rotation_z(70_f32.to_radians()),
        _ => Quat::IDENTITY,
    }
}
#[derive(Component)]
pub struct PlayerModel;

pub fn pose_player(
    ready: On<bevy::world_serialization::WorldInstanceReady>,
    roots: Query<(), With<PlayerModel>>,
    children: Query<&Children>,
    mut nodes: Query<(&Name, &mut Transform), Without<PlayerModel>>,
) {
    if !roots.contains(ready.entity) {
        return;
    }
    for entity in children.iter_descendants(ready.entity) {
        if let Ok((name, mut transform)) = nodes.get_mut(entity) {
            transform.rotation *= limb_rotation(name.as_str());
        }
    }
}

/// Locomotion state for the demo's single player.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Player {
    pub position: Vec2,
    pub yaw: f32,
    pub height: f32,
    pub vertical_speed: f32,
    pub(crate) coyote: f32,
    pub(crate) jump_buffer: f32,
    pub walked: f32,
    pub route: VecDeque<Vec3>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            position: START.xz(),
            yaw: 0.0,
            height: 0.0,
            vertical_speed: 0.0,
            coyote: 0.0,
            jump_buffer: 0.0,
            walked: 0.0,
            route: VecDeque::new(),
        }
    }
}

impl Player {
    pub fn respawn(&mut self, position: Vec2, height: f32) {
        self.position = position;
        self.height = height;
        self.vertical_speed = 0.0;
        self.coyote = 0.0;
        self.jump_buffer = 0.0;
        self.route.clear();
    }

    pub fn feet(&self) -> Vec3 {
        ground(self.position, self.height)
    }

    /// Ground support and ceiling collision are evaluated on the current floor.
    pub fn advance(&mut self, nav: &Navigation, delta: Vec2, dt: f32, jump: bool) {
        let floor = nav.support(self.position, self.height + 0.02);
        let grounded = (self.height - floor).abs() < 0.025 && self.vertical_speed <= 0.0;
        self.coyote = if grounded {
            0.10
        } else {
            (self.coyote - dt).max(0.0)
        };
        self.jump_buffer = if jump {
            0.12
        } else {
            (self.jump_buffer - dt).max(0.0)
        };
        if self.coyote > 0.0 && self.jump_buffer > 0.0 {
            self.vertical_speed = 7.5;
            self.coyote = 0.0;
            self.jump_buffer = 0.0;
            self.route.clear();
        }
        let stepping = grounded && self.vertical_speed <= 0.0;
        let old_height = self.height;
        let ceiling = nav.ceiling(
            self.position,
            self.height + crate::player::actions::navigation::BODY_HEIGHT,
        );
        let mut next_height = self.height + self.vertical_speed * dt - 9.0 * dt * dt;
        self.vertical_speed -= 18.0 * dt;
        if next_height + crate::player::actions::navigation::BODY_HEIGHT > ceiling {
            next_height = ceiling - crate::player::actions::navigation::BODY_HEIGHT;
            self.vertical_speed = self.vertical_speed.min(0.0);
        }
        let from = ground(
            self.position,
            if stepping {
                self.height
            } else {
                next_height.max(0.0)
            },
        );
        let moved = nav.move_body(from, delta, stepping);
        self.position = moved.xz();
        self.height = if stepping {
            moved.y + next_height - old_height
        } else {
            next_height
        };
        let landing = nav.support(self.position, old_height.max(moved.y) + 0.025);
        if self.height <= landing && self.vertical_speed <= 0.0 {
            self.height = landing;
            self.vertical_speed = 0.0;
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects the input and movement resources."
)]
pub fn click_to_move(
    mode: Res<crate::input::InputMode>,
    cursor: crate::input::Cursor,
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<(Entity, &Window), With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<GameCamera>>,
    blocks: Query<(&ComputedNode, &UiGlobalTransform), With<UiBlock>>,
    nav: Res<Navigation>,
    mut player: ResMut<Player>,
    mut view: ResMut<ViewState>,
    load: Res<LevelLoad>,
    mut ui: ResMut<UiState>,
    mut focus: ResMut<InputFocus>,
) {
    let (window_entity, window) = window.into_inner();
    if !buttons.just_pressed(MouseButton::Left)
        || !mode.accepts_input(window)
        || ui.menu
        || !load.is_ready()
    {
        return;
    }
    let Some(cursor) = cursor.position(window_entity, window) else {
        return;
    };
    if pointer_over_ui(cursor, window.scale_factor(), &blocks) {
        return;
    }
    focus.clear();
    if view.first_person {
        view.captured = true;
        return;
    }
    let Ok(ray) = camera.0.viewport_to_world(camera.1, cursor) else {
        return;
    };
    if (player.height - nav.support(player.position, player.height + 0.02)).abs() > 0.15 {
        ui.status = "Lande zuerst, bevor du eine Laufroute auswählst.".into();
        return;
    }
    let Some(target) = nav.pick(ray) else {
        player.route.clear();
        ui.status = "Wähle einen Boden, eine Treppe oder eine Galerie als Laufziel.".into();
        return;
    };
    match nav.path(player.feet(), target) {
        Some(route) => {
            player.route = route;
            ui.status = "Unterwegs. Pfeiltasten übernehmen jederzeit die Steuerung.".into();
        }
        None => {
            player.route.clear();
            ui.status = "Kein Laufweg dorthin. Treppen sind begehbar; Lücken überspringst du mit der Leertaste.".into();
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects the input and movement resources."
)]
pub fn move_player(
    mode: Res<crate::input::InputMode>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window, With<PrimaryWindow>>,
    focus: Res<InputFocus>,
    inputs: Query<(), With<EditableText>>,
    nav: Res<Navigation>,
    settings: Res<Settings>,
    mut player: ResMut<Player>,
    view: Res<ViewState>,
    load: Res<LevelLoad>,
    ui: Res<UiState>,
) {
    if !load.is_ready()
        || ui.menu
        || !mode.accepts_input(&window)
        || focus.get().is_some_and(|e| inputs.contains(e))
        || (view.first_person && !view.captured)
    {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    let mut axis = Vec2::ZERO;
    if keys.pressed(KeyCode::ArrowLeft) || (view.first_person && keys.pressed(KeyCode::KeyA)) {
        axis.x -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) || (view.first_person && keys.pressed(KeyCode::KeyD)) {
        axis.x += 1.0;
    }
    if keys.pressed(KeyCode::ArrowUp) || (view.first_person && keys.pressed(KeyCode::KeyW)) {
        axis.y -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) || (view.first_person && keys.pressed(KeyCode::KeyS)) {
        axis.y += 1.0;
    }
    let speed = settings.speed
        * if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            1.65
        } else {
            1.0
        };
    let before = player.position;
    let mut delta = Vec2::ZERO;
    if axis != Vec2::ZERO {
        player.route.clear();
        let mut direction = axis.normalize();
        if view.first_person {
            direction = (Quat::from_rotation_y(player.yaw) * ground(direction, 0.0)).xz();
        }
        delta = direction * speed * dt;
    } else if let Some(&target) = player.route.front() {
        delta = (target.xz() - player.position).clamp_length_max(speed * dt);
    }
    player.advance(
        &nav,
        delta,
        dt,
        keys.just_pressed(KeyCode::Space) && focus.get().is_none(),
    );
    if player.route.front().is_some_and(|target| {
        target.xz().distance(player.position) < 0.04 && (target.y - player.height).abs() < 0.05
    }) {
        player.route.pop_front();
    }
    let travelled = player.position - before;
    player.walked += travelled.length();
    if !view.first_person && travelled.length_squared() > 0.000001 {
        player.yaw = (-travelled.x).atan2(-travelled.y);
    }
}

pub fn update_model(
    player: Res<Player>,
    view: Res<ViewState>,
    settings: Res<Settings>,
    mut model: Single<(&mut Transform, &mut Visibility), With<PlayerModel>>,
    mut gizmos: Gizmos,
) {
    let (ref mut player_transform, ref mut visibility) = *model;
    player_transform.translation = ground(
        player.position,
        player.height + MODEL_BOB - 0.045 + (player.walked * 7.0).sin().abs() * 0.045,
    );
    // Dummy_Base faces local +Z; movement and the FPV camera use local -Z.
    player_transform.rotation = Quat::from_rotation_y(player.yaw + std::f32::consts::PI);
    **visibility = if view.first_person {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };
    if settings.show_path && !view.first_person && !player.route.is_empty() {
        gizmos.linestrip(
            std::iter::once(player.feet())
                .chain(player.route.iter().copied())
                .map(|p| p + Vec3::Y * 0.12),
            level::TEAL,
        );
        if let Some(&goal) = player.route.back() {
            gizmos.circle(
                Isometry3d::new(
                    goal + Vec3::Y * 0.12,
                    Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                ),
                0.4,
                level::TEAL,
            );
        }
    }
    if !view.first_person {
        gizmos.circle(
            Isometry3d::new(
                ground(player.position, player.height + 0.1),
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
                .init_resource::<crate::input::InputMode>()
                .init_resource::<ButtonInput<KeyCode>>()
                .init_resource::<InputFocus>()
                .init_resource::<Navigation>()
                .init_resource::<Settings>()
                .init_resource::<Player>()
                .init_resource::<ViewState>()
                .init_resource::<UiState>()
                .insert_resource(LevelLoad::Ready)
                .init_gizmo_group::<DefaultGizmoConfigGroup>()
                .add_systems(
                    Update,
                    (move_player, update_model, crate::camera::update_camera).chain(),
                );
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
                let mut player = app.world_mut().resource_mut::<Player>();
                let target = player.position + direction * 3.0;
                player.route.push_back(ground(target, 0.0));
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

            let before = app.world().resource::<Player>().position;
            app.world_mut().resource_mut::<ViewState>().toggle();
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release_all();
            keys.press(KeyCode::KeyW);
            app.world_mut().run_schedule(Update);
            let travelled = (app.world().resource::<Player>().position - before).normalize();
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
            app.world_mut().resource_mut::<ViewState>().toggle();
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
