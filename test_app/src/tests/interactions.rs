use super::*;
#[test]
fn archive_scanner_can_be_used_from_every_accessible_side() {
    let nav = level::navigation_world();
    let mission = Mission::default();
    let center = point(4., 50., 0.);
    for offset in [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z] {
        let position = center + offset * 1.6;
        assert!(nav.walkable(position), "test approach must be accessible");
        let mut player = Player::default();
        player.respawn(position.xz(), 0.);
        assert_eq!(
            mission.interaction(&player, &nav, &crate::player::camera::ViewState::default()),
            Interaction::Scan(crate::level::mission::ScanId::ALL[0]),
            "scanner unavailable from {offset:?}"
        );
        let direction = (center + Vec3::Y - (player.feet() + Vec3::Y * 1.5)).normalize();
        player.face_yaw((-direction.x).atan2(-direction.z));
        let view =
            crate::player::camera::ViewState::new(crate::player::camera::ViewMode::FirstPerson)
                .with_test_pose(
                    direction.y.asin(),
                    Some(
                        Transform::from_translation(player.feet() + Vec3::Y * 1.5)
                            .looking_to(direction, Vec3::Y),
                    ),
                );
        assert_eq!(
            mission.interaction(&player, &nav, &view),
            Interaction::Scan(crate::level::mission::ScanId::ALL[0])
        );
    }
}

#[test]
fn interaction_requires_aim_reach_same_floor_and_no_wall() {
    let nav = level::navigation_world();
    let mut mission = Mission::default();
    let mut player = Player::default();
    player.respawn(point(4., 48.4, 0.).xz(), 0.);
    let mut view =
        crate::player::camera::ViewState::new(crate::player::camera::ViewMode::FirstPerson)
            .with_test_pose(
                -0.3,
                Some(
                    Transform::from_translation(player.feet() + Vec3::Y * 1.5)
                        .with_rotation(Quat::from_rotation_x(-0.3)),
                ),
            );
    assert_eq!(
        mission.interaction(&player, &nav, &view),
        Interaction::Scan(crate::level::mission::ScanId::ALL[0])
    );
    player.face_yaw(std::f32::consts::PI);
    {
        let mut eye = view.eye().unwrap();
        eye.rotation = Quat::from_euler(EulerRot::YXZ, player.yaw(), view.pitch(), 0.);
        view.set_test_eye(Some(eye));
    }
    assert_eq!(mission.interaction(&player, &nav, &view), Interaction::None);
    player.face_yaw(0.);
    {
        let mut eye = view.eye().unwrap();
        eye.rotation = Quat::from_rotation_x(view.pitch());
        view.set_test_eye(Some(eye));
    }
    player.respawn(player.position(), 3.);
    assert_eq!(mission.interaction(&player, &nav, &view), Interaction::None);
    player.respawn(player.position(), 0.);
    player.respawn(point(4., 46., 0.).xz(), player.height());
    assert_eq!(mission.interaction(&player, &nav, &view), Interaction::None);
    player.respawn(point(4., 48.4, 0.).xz(), player.height());

    let blocked_nav = nav.with_obstacle(
        crate::level::navigation::Obstacle::new(
            "test wall".into(),
            point(4., 49.2, 1.5).to_array(),
            [2., 1.5, 0.10],
        )
        .expect("valid obstacle"),
    );
    assert_eq!(
        mission.interaction(&player, &blocked_nav, &view),
        Interaction::None
    );

    mission
        .try_apply(Interaction::Scan(crate::level::mission::ScanId::ALL[0]))
        .unwrap();
    assert_eq!(
        mission.interaction(&player, &nav, &view),
        Interaction::None,
        "used scanner must not offer the same action again"
    );
    mission = Mission::default();
    assert_eq!(
        mission.interaction(&player, &nav, &view),
        Interaction::Scan(crate::level::mission::ScanId::ALL[0])
    );
}

#[test]
fn first_person_interactions_use_the_actual_head_camera_ray() {
    let nav = level::navigation_world();
    let mission = Mission::default();
    let mut player = Player::default();
    player.respawn(point(4., 48.4, 0.).xz(), 0.);
    let mut view =
        crate::player::camera::ViewState::new(crate::player::camera::ViewMode::FirstPerson);
    assert_eq!(mission.interaction(&player, &nav, &view), Interaction::None);
    // A level ray above the scanner must miss, even though the old fixed
    // 1.5-metre origin and player's yaw/pitch would have selected it.
    view.set_test_eye(Some(Transform::from_translation(
        player.feet() + Vec3::Y * 2.,
    )));
    assert_eq!(mission.interaction(&player, &nav, &view), Interaction::None);
    {
        let mut eye = view.eye().unwrap();
        eye.translation.y = 0.9;
        view.set_test_eye(Some(eye));
    }
    assert_eq!(
        mission.interaction(&player, &nav, &view),
        Interaction::Scan(crate::level::mission::ScanId::ALL[0])
    );
    {
        let mut eye = view.eye().unwrap();
        eye.rotation = Quat::from_rotation_y(std::f32::consts::PI);
        view.set_test_eye(Some(eye));
    }
    assert_eq!(mission.interaction(&player, &nav, &view), Interaction::None);
}
