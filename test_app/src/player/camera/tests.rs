use super::*;
use crate::player::{self, MODEL_SCALE, PlayerModel};
use bevy::{
    ecs::system::RunSystemOnce,
    gizmos::{AppGizmoBuilder, config::DefaultGizmoConfigGroup},
};

fn scene() -> (App, Entity, Entity, Entity) {
    let mut app = App::new();
    app.add_plugins(TransformPlugin)
        .init_resource::<Time>()
        .init_resource::<Player>()
        .init_resource::<crate::level::navigation::Navigation>()
        .init_resource::<Settings>()
        .init_resource::<Assets<bevy::gizmos::GizmoAsset>>()
        .insert_resource(ViewState {
            first_person: true,
            captured: true,
            pitch: 0.3,
            ..default()
        })
        .init_gizmo_group::<DefaultGizmoConfigGroup>()
        .add_systems(Update, (player::update_model, update_camera).chain());
    let model = app
        .world_mut()
        .spawn((
            PlayerModel,
            Transform::from_scale(Vec3::splat(MODEL_SCALE)),
            Visibility::default(),
        ))
        .id();
    // Use the shipped head hierarchy instead of a second, hard-coded eye height.
    let gltf: serde_json::Value =
        serde_json::from_str(include_str!("../../../asset/Assets/gltf/Dummy_Base.gltf")).unwrap();
    fn spawn_node(world: &mut World, gltf: &serde_json::Value, index: usize, parent: Entity) {
        let node = &gltf["nodes"][index];
        let translation = node["translation"].as_array().map_or(Vec3::ZERO, |v| {
            Vec3::new(
                v[0].as_f64().unwrap() as f32,
                v[1].as_f64().unwrap() as f32,
                v[2].as_f64().unwrap() as f32,
            )
        });
        let entity = world
            .spawn((
                Name::new(node["name"].as_str().unwrap().to_owned()),
                Transform::from_translation(translation),
                ChildOf(parent),
            ))
            .id();
        if let Some(children) = node["children"].as_array() {
            for child in children {
                spawn_node(world, gltf, child.as_u64().unwrap() as usize, entity);
            }
        }
    }
    let root = gltf["scenes"][0]["nodes"][0].as_u64().unwrap() as usize;
    spawn_node(app.world_mut(), &gltf, root, model);
    let head = app
        .world_mut()
        .query::<(Entity, &Name)>()
        .iter(app.world())
        .find_map(|(entity, name)| (name.as_str() == "Dummy_Head").then_some(entity))
        .unwrap();
    app.world_mut()
        .run_system_once(
            move |mut commands: Commands,
                  children: Query<&Children>,
                  mut nodes: Query<(&Name, &mut Transform), Without<PlayerModel>>| {
                for entity in children.iter_descendants(model) {
                    let (name, mut transform) = nodes.get_mut(entity).unwrap();
                    player::pose_part(&mut commands, entity, name.as_str(), &mut transform);
                }
            },
        )
        .unwrap();
    let camera = app
        .world_mut()
        .spawn((GameCamera, Transform::default(), Projection::default()))
        .id();
    (app, model, head, camera)
}

fn assert_eye_matches_camera(app: &App, camera: Entity) {
    let eye = app.world().resource::<ViewState>().eye.unwrap();
    let actual = app.world().get::<GlobalTransform>(camera).unwrap();
    assert!(eye.translation.distance(actual.translation()) < 0.00001);
    assert!(eye.rotation.dot(actual.rotation()).abs() > 0.99999);
    assert!(actual.scale().distance(Vec3::ONE) < 0.00001);
}

#[test]
fn first_person_camera_is_fixed_to_the_real_head() {
    let (mut app, _, head, camera) = scene();
    app.update();
    let anchor = app.world().get::<ChildOf>(camera).unwrap().parent();
    assert_eq!(app.world().get::<ChildOf>(anchor).unwrap().parent(), head);
    assert_eq!(
        app.world().get::<Transform>(anchor).unwrap().translation,
        EYE_POSITION
    );
    let local = app.world().get::<Transform>(camera).unwrap();
    assert_eq!(local.translation, Vec3::ZERO);
    assert!(local.rotation.dot(Quat::from_rotation_x(0.3)).abs() > 0.99999);
    assert!(matches!(
        app.world().get::<Projection>(camera),
        Some(Projection::Perspective(_))
    ));
    assert_eye_matches_camera(&app, camera);
}

#[test]
fn walking_moves_the_character_and_camera_together_without_a_second_bob() {
    let (mut app, model, head, camera) = scene();
    // This test moves up onto a floor, rather than teleporting into mid-air.
    let destination = app.world().resource::<Player>().position + Vec2::new(1., 2.);
    app.insert_resource(crate::level::navigation::Navigation::new(
        vec![crate::level::navigation::Surface {
            center: destination.to_array(),
            half: [1., 1.],
            height: 3.,
            bottom: 2.7,
        }],
        Vec::new(),
        Vec2::splat(-100.),
        Vec2::splat(100.),
    ));
    app.update();
    let local = *app.world().get::<Transform>(camera).unwrap();
    let head_before = app
        .world()
        .get::<GlobalTransform>(head)
        .unwrap()
        .translation();
    let camera_before = app
        .world()
        .get::<GlobalTransform>(camera)
        .unwrap()
        .translation();
    let model_before = app.world().get::<Transform>(model).unwrap().translation;
    {
        let mut player = app.world_mut().resource_mut::<Player>();
        player.walked = 1.;
        player.position += Vec2::new(1., 2.);
        player.height += 3.;
        player.yaw = 0.7;
    }
    app.update();
    let moved = app
        .world()
        .get::<GlobalTransform>(camera)
        .unwrap()
        .translation()
        - camera_before;
    let model_moved = app.world().get::<Transform>(model).unwrap().translation - model_before;
    let head_moved = app
        .world()
        .get::<GlobalTransform>(head)
        .unwrap()
        .translation()
        - head_before;
    assert!(moved.distance(head_moved) < 0.00001);
    assert!(
        (model_moved.y - 3.).abs() < 0.00001,
        "the base must not bob"
    );
    assert!(
        moved.y - model_moved.y > player::BODY_BOB * 0.5,
        "the torso gait must move the head and camera"
    );
    assert_eq!(*app.world().get::<Transform>(camera).unwrap(), local);
    assert_eye_matches_camera(&app, camera);
    let expected = Quat::from_euler(EulerRot::YXZ, 0.7, 0.3, 0.) * Vec3::NEG_Z;
    assert!(
        app.world()
            .resource::<ViewState>()
            .eye
            .unwrap()
            .forward()
            .dot(expected)
            > 0.99999
    );
    let stopped = app
        .world()
        .get::<GlobalTransform>(camera)
        .unwrap()
        .translation();
    app.update();
    assert_eq!(
        app.world()
            .get::<GlobalTransform>(camera)
            .unwrap()
            .translation(),
        stopped
    );
}

fn walking_scene(first_person: bool) -> (App, Entity, Entity, Entity) {
    let (mut app, model, head, camera) = scene();
    app.init_resource::<crate::input::InputMode>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<bevy::input_focus::InputFocus>()
        .init_resource::<crate::level::navigation::Navigation>()
        .init_resource::<crate::ui::UiState>()
        .insert_resource(crate::level::LevelLoad::Ready)
        .add_systems(Update, player::move_player.before(player::update_model));
    app.world_mut().spawn((
        Window {
            focused: true,
            ..default()
        },
        PrimaryWindow,
    ));
    app.world_mut().resource_mut::<ViewState>().first_person = first_person;
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(1. / 60.));
    app.update();
    (app, model, head, camera)
}

fn check_body_walk(first_person: bool, click_route: bool) {
    let (mut app, model, _, _) = walking_scene(first_person);
    let body = app
        .world_mut()
        .query::<(Entity, &Name)>()
        .iter(app.world())
        .find_map(|(entity, name)| (name.as_str() == "Dummy_Body").then_some(entity))
        .unwrap();
    let rest_y = app.world().get::<Transform>(body).unwrap().translation.y;
    let base_y = app.world().get::<Transform>(model).unwrap().translation.y;
    let mut body_range = (rest_y, rest_y);
    let mut base_range = (base_y, base_y);
    if click_route {
        let mut player = app.world_mut().resource_mut::<Player>();
        let target = player.feet() + Vec3::X * 10.;
        player.route.push_back(target);
    } else {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowRight);
    }
    for _ in 0..24 {
        app.update();
        let y = app.world().get::<Transform>(body).unwrap().translation.y;
        body_range = (body_range.0.min(y), body_range.1.max(y));
        assert!(
            y >= rest_y && y <= rest_y + player::BODY_BOB / MODEL_SCALE + 0.00001,
            "body displacement must be relative to its authored rest pose, without drift"
        );
        let y = app.world().get::<Transform>(model).unwrap().translation.y;
        base_range = (base_range.0.min(y), base_range.1.max(y));
    }
    assert!(
        app.world().resource::<Player>().walked > 0.5,
        "test must actually walk"
    );
    assert!(
        body_range.1 - body_range.0 > 0.01,
        "Dummy_Body must move locally while walking (FPV={first_person}, route={click_route}); observed y range {body_range:?}"
    );
    assert!(
        base_range.1 - base_range.0 < 0.00001,
        "base must not bob: {base_range:?}"
    );
    assert_eq!(
        *app.world().get::<Visibility>(model).unwrap(),
        if first_person {
            Visibility::Hidden
        } else {
            Visibility::Visible
        }
    );
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    app.world_mut().resource_mut::<Player>().route.clear();
    let stopped = app.world().get::<Transform>(body).unwrap().translation;
    app.update();
    assert_eq!(
        app.world().get::<Transform>(body).unwrap().translation,
        stopped
    );
}

#[test]
fn overview_walk_animates_dummy_body_without_bobbing_the_base() {
    check_body_walk(false, false);
}

#[test]
fn first_person_walk_animates_dummy_body_without_bobbing_the_base() {
    check_body_walk(true, false);
}

#[test]
fn click_routes_animate_dummy_body_without_bobbing_the_base() {
    check_body_walk(false, true);
}

#[test]
fn jumping_suspends_body_bob_until_landing_in_both_perspectives() {
    for first_person in [false, true] {
        let (mut app, _, _, camera) = walking_scene(first_person);
        let body = app
            .world_mut()
            .query_filtered::<Entity, With<player::BodyMotion>>()
            .single(app.world())
            .unwrap();
        let rest_y = app.world().get::<player::BodyMotion>(body).unwrap().rest_y;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowRight);
        for _ in 0..8 {
            app.update();
        }
        assert!(
            app.world().get::<Transform>(body).unwrap().translation.y > rest_y + 0.001,
            "walking must have enabled body bob before jumping"
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear_just_pressed(KeyCode::Space);
        assert!(app.world().resource::<Player>().vertical_speed > 0.);
        let mut saw_falling = false;
        let mut landed = false;
        for _ in 0..120 {
            let player = app.world().resource::<Player>();
            let airborne = player.height > 0.025 || player.vertical_speed > 0.;
            saw_falling |= player.vertical_speed < 0.;
            let body_y = app.world().get::<Transform>(body).unwrap().translation.y;
            if !airborne {
                assert!(
                    body_y > rest_y + 0.001,
                    "walking bob must resume on landing"
                );
                landed = true;
                break;
            }
            assert!(
                (body_y - rest_y).abs() < 0.00001,
                "airborne Dummy_Body must not bob (FPV={first_person}, height={}, speed={}, body_y={body_y}, rest_y={rest_y})",
                player.height,
                player.vertical_speed
            );
            if first_person {
                assert_eye_matches_camera(&app, camera);
            }
            app.update();
        }
        assert!(
            saw_falling && landed,
            "test must include ascent, descent and landing"
        );
    }
}

#[test]
fn airborne_body_stays_at_rest_even_at_the_apex_or_start_of_ascent() {
    for (height, vertical_speed) in [(1., 3.), (1., 0.), (1., -3.), (0., 3.)] {
        let (mut app, _, _, camera) = scene();
        {
            let mut player = app.world_mut().resource_mut::<Player>();
            player.height = height;
            player.vertical_speed = vertical_speed;
            player.walked = 1.;
        }
        app.update();
        let body = app
            .world_mut()
            .query_filtered::<Entity, With<player::BodyMotion>>()
            .single(app.world())
            .unwrap();
        let rest_y = app.world().get::<player::BodyMotion>(body).unwrap().rest_y;
        assert!(
            (app.world().get::<Transform>(body).unwrap().translation.y - rest_y).abs() < 0.00001,
            "airborne torso must stay at rest (height={height}, speed={vertical_speed})"
        );
        assert_eye_matches_camera(&app, camera);
    }
}

#[test]
fn changing_the_head_pose_moves_the_camera_and_same_frame_rays() {
    let (mut app, _, head, camera) = scene();
    app.update();
    let before = app.world().resource::<ViewState>().eye.unwrap();
    {
        let mut head = app.world_mut().get_mut::<Transform>(head).unwrap();
        head.translation += Vec3::new(0.1, 0.2, 0.05);
        head.rotate_local_z(0.1);
    }
    app.update();
    let eye = app.world().resource::<ViewState>().eye.unwrap();
    assert!(eye.translation.distance(before.translation) > 0.1);
    assert!(eye.rotation.dot(before.rotation).abs() < 0.99999);
    assert_eye_matches_camera(&app, camera);
}

#[test]
fn camera_waits_for_its_head_instead_of_inventing_an_eye_height() {
    let (mut app, _, _, camera) = scene();
    let anchor = app
        .world_mut()
        .query_filtered::<Entity, With<EyeAnchor>>()
        .single(app.world())
        .unwrap();
    app.world_mut().entity_mut(anchor).remove::<EyeAnchor>();
    let waiting = Vec3::new(10., 30., 20.);
    app.world_mut()
        .get_mut::<Transform>(camera)
        .unwrap()
        .translation = waiting;
    app.update();
    assert!(app.world().resource::<ViewState>().eye.is_none());
    assert!(app.world().get::<ChildOf>(camera).is_none());
    assert_eq!(
        app.world().get::<Transform>(camera).unwrap().translation,
        waiting
    );
    app.world_mut().entity_mut(anchor).insert(EyeAnchor);
    app.update();
    assert_eq!(app.world().get::<ChildOf>(camera).unwrap().parent(), anchor);
    assert_eye_matches_camera(&app, camera);
}

#[test]
fn perspective_switches_detach_and_reattach_without_losing_the_eye_point() {
    let (mut app, _, _, camera) = scene();
    app.update();
    let anchor = app.world().get::<ChildOf>(camera).unwrap().parent();
    app.world_mut().resource_mut::<ViewState>().toggle();
    app.update();
    assert!(app.world().get::<ChildOf>(camera).is_none());
    assert!(app.world().resource::<ViewState>().eye.is_none());
    assert_eq!(
        app.world().get::<Transform>(camera).unwrap().scale,
        Vec3::ONE
    );
    assert!(matches!(
        app.world().get::<Projection>(camera),
        Some(Projection::Orthographic(_))
    ));
    app.world_mut().resource_mut::<ViewState>().toggle();
    app.update();
    assert_eq!(app.world().get::<ChildOf>(camera).unwrap().parent(), anchor);
    assert_eq!(
        app.world().get::<Transform>(camera).unwrap().translation,
        Vec3::ZERO
    );
    assert_eye_matches_camera(&app, camera);
}
