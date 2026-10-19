use super::*;
pub(super) fn spawn(s: &mut Scenery, action: Interaction) {
    housing(s, Vec3::Y * 0.065, Vec3::new(0.80, 0.13, 0.57), 0.07, FRAME);
    housing(
        s,
        Vec3::Y * 0.49,
        Vec3::new(0.65, 0.75, 0.44),
        0.06,
        SURFACE,
    );
    housing(
        s,
        Vec3::Y * 0.87,
        Vec3::new(0.79, 0.20, 0.55),
        0.06,
        ItemKind::Cell.color(),
    );
    cylinder(
        s,
        Vec3::new(-0.23, 1.08, 0.),
        0.035,
        0.27,
        FRAME,
        Quat::IDENTITY,
    );
    for (height, radius) in [(1.01, 0.11), (1.10, 0.085), (1.19, 0.055)] {
        cylinder(
            s,
            Vec3::new(-0.23, height, 0.),
            radius,
            0.035,
            BLUE,
            Quat::IDENTITY,
        );
    }
    let rest = Transform::from_xyz(0.16, 1.01, 0.);
    let button = cylinder(s, rest.translation, 0.115, 0.11, READY, Quat::IDENTITY);
    moving(s, button, action, rest, Transform::from_xyz(0.16, 0.96, 0.));
    labels(s, "sender", 0.39, 0.60, 0.45, action);
    mark(s, action, Vec3::Y * 0.64, 0.46);
}
