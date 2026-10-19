use super::*;
pub(super) fn spawn(s: &mut Scenery) {
    for x in [-0.48, 0.48] {
        housing(
            s,
            Vec3::new(x, 0.075, 0.),
            Vec3::new(0.29, 0.15, 0.54),
            0.05,
            FRAME,
        );
        s.block(Vec3::new(x, 0.78, 0.), Vec3::new(0.11, 1.40, 0.12), SURFACE);
    }
    housing(s, Vec3::Y * 1.27, Vec3::new(1.32, 0.86, 0.18), 0.04, FRAME);
    labels(s, "board", 1.48, 1.20, 0.19, Interaction::Board);
    // A small route diagram connects three battery symbols to the antenna.
    for side in [-1., 1.] {
        s.block(
            Vec3::new(0., 1.01, side * 0.10),
            Vec3::new(0.96, 0.035, 0.025),
            SURFACE,
        );
        for x in [-0.38, 0., 0.38] {
            s.block(
                Vec3::new(x, 1.115, side * 0.11),
                Vec3::new(0.13, 0.20, 0.025),
                ItemKind::Cell.color(),
            );
            s.block(
                Vec3::new(x, 1.23, side * 0.11),
                Vec3::new(0.055, 0.03, 0.025),
                SURFACE,
            );
        }
    }
}
