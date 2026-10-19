use super::*;
use crate::player::camera::GameCamera;
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
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<bevy::input::mouse::AccumulatedMouseMotion>()
            .init_resource::<InputFocus>()
            .init_resource::<Navigation>()
            .init_resource::<Settings>()
            .init_resource::<Player>()
            .init_resource::<ViewState>()
            .init_resource::<UiState>()
            .init_resource::<crate::app::flow::GameFlow>()
            .init_resource::<crate::input::PointerCapture>()
            .init_resource::<crate::input::intents::FrameInput>()
            .insert_resource(LevelLoad::Ready)
            .init_gizmo_group::<DefaultGizmoConfigGroup>()
            .add_systems(
                Update,
                (
                    crate::input::sample_gameplay,
                    move_player,
                    update_model,
                    crate::player::camera::update_camera,
                )
                    .chain(),
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
        app.world_mut()
            .resource_mut::<crate::input::PointerCapture>()
            .set(true);
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
        app.world_mut()
            .resource_mut::<crate::input::PointerCapture>()
            .set(true);
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
