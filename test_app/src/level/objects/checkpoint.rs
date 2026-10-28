use super::*;
pub(super) fn spawn(s: &mut Scenery, action: Interaction) {
    housing(s, Vec3::Y * 0.06, Vec3::new(0.48, 0.12, 0.46), 0.08, FRAME);
    cylinder(
        s,
        Vec3::new(-0.16, 0.71, 0.),
        0.04,
        1.30,
        SURFACE,
        Quat::IDENTITY,
    );
    s.block(
        Vec3::new(0.015, 1.13, 0.),
        Vec3::new(0.40, 0.44, 0.07),
        BLUE,
    );
    mark(s, action, Vec3::new(0.015, 1.13, 0.), 0.085);
    labels(s, "checkpoint", 0.53, 0.44, 0.10, action);
}
