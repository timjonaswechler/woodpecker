use super::*;
pub(super) fn spawn(s: &mut Scenery, action: Interaction) {
    housing(s, Vec3::Y * 0.07, Vec3::new(1.18, 0.14, 0.78), 0.08, FRAME);
    housing(
        s,
        Vec3::Y * 0.74,
        Vec3::new(1.05, 1.28, 0.66),
        0.08,
        SURFACE,
    );
    housing(
        s,
        Vec3::Y * 1.40,
        Vec3::new(1.12, 0.14, 0.72),
        0.07,
        ItemKind::Cell.color(),
    );
    for side in [-1., 1.] {
        for x in [-0.31, 0., 0.31] {
            s.block(
                Vec3::new(x, 1.08, side * 0.338),
                Vec3::new(0.22, 0.42, 0.022),
                FRAME,
            );
            let e = s.block(
                Vec3::new(x, 1.08, side * 0.355),
                Vec3::new(0.14, 0.32, 0.015),
                ItemKind::Cell.color(),
            );
            s.commands.entity(e).insert(CompletionMark(action));
            s.block(
                Vec3::new(x, 1.32, side * 0.338),
                Vec3::new(0.075, 0.035, 0.022),
                FRAME,
            );
        }
    }
    labels(s, "generator", 0.38, 0.92, 0.67, action);
    mark(s, action, Vec3::Y * 0.72, 0.68);
}
