use crate::{
    camera::{GameCamera, ViewState},
    level::{self, LevelLoad, START, ground},
    navigation::Navigation,
    settings::Settings,
    ui::{UiBlock, UiState, pointer_over_ui},
};
use bevy::{input_focus::InputFocus, prelude::*, text::EditableText, window::PrimaryWindow};
use std::collections::VecDeque;

#[derive(Component)]
pub struct PlayerModel;

/// Locomotion state for the demo's single player.
#[derive(Resource)]
pub struct Player {
    pub position: Vec2,
    pub yaw: f32,
    pub walked: f32,
    pub route: VecDeque<Vec2>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            position: START,
            yaw: 0.0,
            walked: 0.0,
            route: VecDeque::new(),
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects the input and movement resources."
)]
pub fn click_to_move(
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<GameCamera>>,
    blocks: Query<(&ComputedNode, &UiGlobalTransform), With<UiBlock>>,
    nav: Res<Navigation>,
    mut player: ResMut<Player>,
    mut view: ResMut<ViewState>,
    load: Res<LevelLoad>,
    mut ui: ResMut<UiState>,
    mut focus: ResMut<InputFocus>,
) {
    if !buttons.just_pressed(MouseButton::Left)
        || !window.focused
        || ui.menu
        || !load.is_ready()
        || pointer_over_ui(&window, &blocks)
    {
        return;
    }
    focus.clear();
    if view.first_person {
        view.captured = true;
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
    match nav.path(player.position, target) {
        Some(route) => {
            player.route = route;
            ui.status = "Unterwegs. Pfeiltasten übernehmen jederzeit die Steuerung.".into();
        }
        None => {
            player.route.clear();
            ui.status = "Dieses Ziel ist blockiert. Klicke auf freien Boden.".into();
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects the input and movement resources."
)]
pub fn move_player(
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
        || !window.focused
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
    if axis != Vec2::ZERO {
        player.route.clear();
        let mut direction = axis.normalize();
        if view.first_person {
            direction = (Quat::from_rotation_y(player.yaw) * ground(direction, 0.0)).xz();
        }
        player.position = nav.move_and_slide(player.position, direction * speed * dt);
    } else {
        let mut budget = speed * dt;
        while let Some(&target) = player.route.front() {
            let delta = target - player.position;
            let distance = delta.length();
            if distance <= budget {
                player.position = target;
                player.route.pop_front();
                budget -= distance;
            } else {
                player.position = nav.move_and_slide(player.position, delta / distance * budget);
                break;
            }
            if budget <= 0.0 {
                break;
            }
        }
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
        0.025 + (player.walked * 7.0).sin().abs() * 0.045,
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
            std::iter::once(player.position)
                .chain(player.route.iter().copied())
                .map(|p| ground(p, 0.12)),
            level::TEAL,
        );
        if let Some(&goal) = player.route.back() {
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
    if !view.first_person {
        gizmos.circle(
            Isometry3d::new(
                ground(player.position, 0.1),
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
                player.route.push_back(target);
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
