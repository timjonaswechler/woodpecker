use super::*;
pub(super) fn spawn(s: &mut Scenery, action: Interaction) {
    let blue = ItemKind::Blueprint.color();
    housing(s, Vec3::Y * 0.085, Vec3::new(1.60, 0.17, 1.20), 0.10, FRAME);
    for x in [-0.72, 0.72] {
        housing(
            s,
            Vec3::new(x, 0.44, 0.),
            Vec3::new(0.16, 0.59, 1.16),
            0.04,
            blue,
        );
    }
    for z in [-0.51, 0.51] {
        housing(
            s,
            Vec3::new(0., 0.44, z),
            Vec3::new(1.42, 0.59, 0.16),
            0.04,
            blue,
        );
    }
    housing(
        s,
        Vec3::Y * 0.658,
        Vec3::new(1.30, 0.055, 0.86),
        0.04,
        FRAME,
    );
    for x in [-0.59, 0.59] {
        for z in [-0.584, 0.584] {
            s.block(
                Vec3::new(x, 0.43, z),
                Vec3::new(0.075, 0.53, 0.024),
                SURFACE,
            );
        }
    }
    let rest = Transform::from_xyz(0., 0.79, -0.54);
    let lid = s.spawn((rest, Visibility::Inherited));
    let previous = s.parent.replace(lid);
    housing(
        s,
        Vec3::Z * 0.54,
        Vec3::new(1.58, 0.13, 1.16),
        0.10,
        SURFACE,
    );
    housing(
        s,
        Vec3::new(0., 0.077, 0.54),
        Vec3::new(1.32, 0.03, 0.90),
        0.07,
        blue,
    );
    for x in [-0.58, 0.58] {
        s.block(
            Vec3::new(x, 0.079, 0.54),
            Vec3::new(0.08, 0.033, 0.94),
            FRAME,
        );
    }
    s.parent = previous;
    moving(
        s,
        lid,
        action,
        rest,
        rest.with_rotation(Quat::from_rotation_x(-1.32)),
    );
    labels(s, "chest", 0.40, 0.95, 1.13, action);
}
