use super::*;
pub(super) fn spawn(s: &mut Scenery, index: usize, action: Interaction) {
    housing(s, Vec3::Y * 0.07, Vec3::new(0.86, 0.14, 0.68), 0.08, FRAME);
    housing(
        s,
        Vec3::Y * 0.50,
        Vec3::new(0.62, 0.80, 0.42),
        0.06,
        SURFACE,
    );
    housing(s, Vec3::Y * 0.91, Vec3::new(1.16, 0.15, 0.82), 0.08, FRAME);
    s.block(Vec3::Y * 0.981, Vec3::new(0.79, 0.015, 0.63), BLUE);
    let page = s.spawn((
        Transform::from_xyz(0., 1.02, 0.),
        Visibility::Inherited,
        UnreadPage(index),
    ));
    let previous = s.parent.replace(page);
    housing(s, Vec3::ZERO, Vec3::new(0.70, 0.055, 0.54), 0.025, SURFACE);
    for z in [-0.15, 0., 0.15] {
        s.block(
            Vec3::new(-0.02, 0.032, z),
            Vec3::new(0.45, 0.012, 0.035),
            FRAME,
        );
    }
    s.parent = previous;
    for x in [-0.49, 0.49] {
        s.block(Vec3::new(x, 1.21, 0.), Vec3::new(0.09, 0.51, 0.16), SURFACE);
    }
    housing(
        s,
        Vec3::new(0., 1.46, 0.),
        Vec3::new(1.08, 0.12, 0.24),
        0.045,
        BLUE,
    );
    for x in [-0.27, 0., 0.27] {
        cylinder(
            s,
            Vec3::new(x, 1.393, 0.),
            0.043,
            0.02,
            ItemKind::Cell.color(),
            Quat::IDENTITY,
        );
    }
    labels(s, "scanner", 0.40, 0.74, 0.43, action);
    mark(s, action, Vec3::Y * 0.72, 0.44);
}
