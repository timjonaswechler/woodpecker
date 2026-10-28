use super::*;
pub(super) fn spawn(s: &mut Scenery, action: Interaction) {
    housing(s, Vec3::Y * 0.07, Vec3::new(0.84, 0.14, 0.72), 0.07, FRAME);
    housing(
        s,
        Vec3::Y * 0.47,
        Vec3::new(0.74, 0.68, 0.48),
        0.06,
        SURFACE,
    );
    housing(s, Vec3::Y * 0.81, Vec3::new(0.74, 0.10, 0.48), 0.05, FRAME);
    let rest = Transform::from_xyz(0., 0.88, 0.);
    let root = s.spawn((rest, Visibility::Inherited));
    let previous = s.parent.replace(root);
    cylinder(
        s,
        Vec3::ZERO,
        0.11,
        0.48,
        FRAME,
        Quat::from_rotation_z(FRAC_PI_2),
    );
    for x in [-0.18, 0.18] {
        s.block(Vec3::new(x, 0.16, 0.), Vec3::new(0.055, 0.32, 0.065), FRAME);
    }
    housing(
        s,
        Vec3::Y * 0.34,
        Vec3::new(0.60, 0.13, 0.14),
        0.04,
        ItemKind::Part.color(),
    );
    s.parent = previous;
    moving(
        s,
        root,
        action,
        rest,
        rest.with_rotation(Quat::from_rotation_x(1.0)),
    );
    labels(s, "rack", 0.39, 0.66, 0.49, action);
    mark(s, action, Vec3::Y * 0.64, 0.50);
}

#[derive(Component)]
pub(crate) struct OriginalPose(Transform);

pub(crate) fn sync_collision(
    mission: &Mission,
    nav: &mut crate::level::navigation::Navigation,
    player: &mut crate::player::Player,
) {
    let original = crate::level::definition::DATA
        .obstacles()
        .iter()
        .find(|o| o.name() == crate::level::definition::RACK_COLLIDER)
        .expect("rack collider in checked level");
    let center = Vec3::from(original.center())
        + if mission.rack_open() {
            Vec3::NEG_Z * 3.
        } else {
            Vec3::ZERO
        };
    if nav
        .set_obstacle_pose(crate::level::definition::RACK_COLLIDER, center)
        .expect("rack collider in checked level")
    {
        player.clear_route();
    }
}
pub(crate) fn update_collision(
    mission: Res<Mission>,
    mut nav: ResMut<crate::level::navigation::Navigation>,
    mut player: ResMut<crate::player::Player>,
) {
    sync_collision(&mission, &mut nav, &mut player);
}
pub(crate) fn update_model(
    mut commands: Commands,
    mission: Res<Mission>,
    mut models: Query<(Entity, &Name, &mut Transform, Option<&OriginalPose>)>,
) {
    for (entity, name, mut transform, original) in &mut models {
        if name.as_str() != crate::level::definition::RACK_COLLIDER {
            continue;
        }
        let base = original.map_or(*transform, |p| p.0);
        if original.is_none() {
            commands.entity(entity).insert(OriginalPose(base));
        }
        transform.translation =
            base.translation + Vec3::NEG_Z * if mission.rack_open() { 3. } else { 0. };
    }
}
