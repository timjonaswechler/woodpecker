use super::*;
pub(super) fn spawn(s: &mut Scenery, action: Interaction) {
    let orange = ItemKind::Part.color();
    housing(s, Vec3::Y * 0.08, Vec3::new(1.15, 0.16, 0.79), 0.09, FRAME);
    housing(
        s,
        Vec3::Y * 0.43,
        Vec3::new(0.94, 0.62, 0.60),
        0.06,
        SURFACE,
    );
    housing(s, Vec3::Y * 0.80, Vec3::new(1.18, 0.16, 0.78), 0.07, orange);
    for x in [-0.46, 0.46] {
        s.block(Vec3::new(x, 1.10, 0.), Vec3::new(0.14, 0.54, 0.24), orange);
    }
    housing(s, Vec3::Y * 1.36, Vec3::new(1.12, 0.18, 0.35), 0.05, orange);
    housing(s, Vec3::Y * 0.90, Vec3::new(0.55, 0.05, 0.40), 0.04, FRAME);
    let rest = Transform::from_xyz(0., 1.12, 0.);
    let press = s.spawn((rest, Visibility::Inherited));
    let previous = s.parent.replace(press);
    cylinder(s, Vec3::Y * 0.16, 0.065, 0.32, SURFACE, Quat::IDENTITY);
    housing(s, Vec3::ZERO, Vec3::new(0.48, 0.12, 0.32), 0.04, FRAME);
    s.parent = previous;
    moving(s, press, action, rest, Transform::from_xyz(0., 0.98, 0.));
    labels(s, "assembly", 0.34, 0.82, 0.62, action);
    mark(s, action, Vec3::Y * 0.605, 0.63);
}

/// Producing the output consumes both collected parts in the mission inventory.
pub(crate) fn can_produce(parts: [bool; 2], output: crate::level::mission::CellState) -> bool {
    parts.into_iter().all(|collected| collected)
        && output == crate::level::mission::CellState::Locked
}
