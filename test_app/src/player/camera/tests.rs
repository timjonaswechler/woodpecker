use super::*;
use bevy::ecs::system::RunSystemOnce;
use std::time::Duration;

#[test]
fn reset_restores_camera_pose_and_projection_without_idle_drift() {
    let mut world = World::new();
    world.insert_resource(Player::default());
    world.insert_resource(Settings::default());
    world.insert_resource(ViewState::new(ViewMode::Map));
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_secs_f32(1. / 60.));
    world.insert_resource(time);
    world.run_system_once(spawn_camera).unwrap();
    for _ in 0..4 {
        world.run_system_cached(update_camera).unwrap();
    }
    world.resource_mut::<ViewState>().reset();
    world.run_system_cached(update_camera).unwrap();
    let (before, before_extent) = snapshot(&mut world);
    for _ in 0..32 {
        world.run_system_cached(update_camera).unwrap();
    }
    let (after, after_extent) = snapshot(&mut world);
    assert!(before.translation.abs_diff_eq(after.translation, 1e-5));
    assert!(before.rotation.abs_diff_eq(after.rotation, 1e-5));
    assert!((before_extent - after_extent).abs() < 1e-5);
}

fn snapshot(world: &mut World) -> (Transform, f32) {
    let (transform, projection) = world
        .query_filtered::<(&Transform, &Projection), With<GameCamera>>()
        .single(world)
        .unwrap();
    let Projection::Orthographic(projection) = projection else {
        panic!("reset must restore the overview projection");
    };
    let bevy::camera::ScalingMode::FixedVertical { viewport_height } = projection.scaling_mode
    else {
        panic!("overview must have a fixed vertical extent");
    };
    (*transform, viewport_height)
}
