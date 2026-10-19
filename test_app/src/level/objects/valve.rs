use super::*;
pub(super) fn spawn(s: &mut Scenery, action: Interaction) {
    cylinder(
        s,
        Vec3::Y * 0.16,
        0.13,
        0.88,
        BLUE,
        Quat::from_rotation_z(FRAC_PI_2),
    );
    for x in [-0.34, 0.34] {
        cylinder(
            s,
            Vec3::new(x, 0.16, 0.),
            0.19,
            0.09,
            SURFACE,
            Quat::from_rotation_z(FRAC_PI_2),
        );
    }
    cylinder(s, Vec3::Y * 0.61, 0.105, 0.93, BLUE, Quat::IDENTITY);
    cylinder(
        s,
        Vec3::new(0., 1.12, 0.025),
        0.12,
        0.29,
        FRAME,
        Quat::from_rotation_x(FRAC_PI_2),
    );
    let rest = Transform::from_xyz(0., 1.12, 0.14);
    let wheel = s.spawn((rest, Visibility::Inherited));
    let previous = s.parent.replace(wheel);
    let mesh = s.meshes.add(
        Torus::new(0.28, 0.38)
            .mesh()
            .major_resolution(20)
            .minor_resolution(6),
    );
    let material = s.materials.add(BLUE);
    s.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)),
    ));
    for i in 0..3 {
        let angle = i as f32 * PI * 2. / 3.;
        let at = Vec3::new(-angle.sin(), angle.cos(), 0.) * 0.15;
        let spoke = s.block(at, Vec3::new(0.055, 0.34, 0.06), SURFACE);
        s.commands
            .entity(spoke)
            .insert(Transform::from_translation(at).with_rotation(Quat::from_rotation_z(angle)));
    }
    cylinder(
        s,
        Vec3::ZERO,
        0.09,
        0.09,
        SURFACE,
        Quat::from_rotation_x(FRAC_PI_2),
    );
    cylinder(
        s,
        Vec3::new(0., 0.32, 0.035),
        0.045,
        0.075,
        READY,
        Quat::from_rotation_x(FRAC_PI_2),
    );
    s.parent = previous;
    moving(
        s,
        wheel,
        action,
        rest,
        rest.with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
    );
    labels(s, "valve", 0.38, 0.82, 0.22, action);
    mark(s, action, Vec3::new(0.30, 0.66, 0.), 0.23);
}
