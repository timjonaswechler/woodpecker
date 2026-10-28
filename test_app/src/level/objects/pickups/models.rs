use super::super::*;
pub(in crate::level::objects) fn spawn_table(s: &mut Scenery, t: ObjectDefinition) {
    let is_part = matches!(t.action, Interaction::Part(_));
    let top = if is_part { 0.84 } else { 1. };
    let color = if is_part {
        ItemKind::Part.color()
    } else {
        ItemKind::Cell.color()
    };
    housing(
        s,
        Vec3::Y * (top - 0.078),
        Vec3::new(t.half.x * 2., 0.14, t.half.z * 2.),
        0.07,
        SURFACE,
    );
    for x in [-1., 1.] {
        for z in [-1., 1.] {
            let at = Vec3::new(
                x * (t.half.x - 0.13),
                (top - 0.14) / 2.,
                z * (t.half.z - 0.13),
            );
            s.block(at, Vec3::new(0.14, top - 0.14, 0.14), FRAME);
            housing(
                s,
                Vec3::new(at.x, 0.04, at.z),
                Vec3::new(0.21, 0.08, 0.21),
                0.03,
                FRAME,
            );
        }
        s.block(
            Vec3::new(x * (t.half.x - 0.13), 0.23, 0.),
            Vec3::new(0.10, 0.08, t.half.z * 2. - 0.24),
            FRAME,
        );
    }
    for z in [-1., 1.] {
        s.block(
            Vec3::new(0., top - 0.13, z * (t.half.z - 0.04)),
            Vec3::new(t.half.x * 2. - 0.10, 0.07, 0.04),
            color,
        );
    }
    // Recessed tray: the physical pickup is the only object on top. Its empty seat remains.
    let tray = if is_part {
        Vec2::new(0.98, 0.98)
    } else {
        Vec2::new(0.64, 0.46)
    };
    housing(
        s,
        Vec3::Y * (top - 0.004),
        Vec3::new(tray.x, 0.008, tray.y),
        0.06,
        FRAME,
    );
    for x in [-1., 1.] {
        s.block(
            Vec3::new(x * (tray.x / 2. - 0.025), top + 0.016, 0.),
            Vec3::new(0.035, 0.032, tray.y - 0.10),
            color,
        );
    }
    let label = match t.action {
        Interaction::Part(_) => "part",
        Interaction::Cell(id) if id.index() == 0 => "cell_assembly",
        Interaction::Cell(id) if id.index() == 1 => "cell_water",
        _ => "cell_archive",
    };
    let width = (t.half.x * 1.65).min(1.20);
    for z in [-1., 1.] {
        // Two brackets fasten each label to the tabletop; neither face is occluded by a post.
        for x in [-width * 0.32, width * 0.32] {
            s.block(
                Vec3::new(x, top - 0.23, z * (t.half.z - 0.045)),
                Vec3::new(0.045, 0.30, 0.04),
                FRAME,
            );
        }
    }
    labels(s, label, top - 0.33, width, t.half.z * 2. - 0.08, t.action);
}

impl Scenery<'_, '_, '_> {
    /// The same silhouette identifies a pickup and the station that produces it.
    pub fn item(&mut self, kind: ItemKind, transform: Transform) -> Entity {
        let color = kind.color();
        let white = Color::srgb(0.94, 1., 0.98);
        let mut parts = Vec::new();
        match kind {
            ItemKind::Coin | ItemKind::Part => {
                let ring = self
                    .commands
                    .spawn((
                        Mesh3d(self.meshes.add(Torus::new(0.17, 0.33))),
                        MeshMaterial3d(self.materials.add(color)),
                        Transform::from_rotation(Quat::from_rotation_x(
                            std::f32::consts::FRAC_PI_2,
                        )),
                    ))
                    .id();
                parts.push(ring);
                if matches!(kind, ItemKind::Part) {
                    for i in 0..8 {
                        let angle = i as f32 * std::f32::consts::TAU / 8.;
                        let tooth = self.block(
                            Vec3::new(angle.sin(), angle.cos(), 0.) * 0.34,
                            Vec3::new(0.17, 0.22, 0.14),
                            color,
                        );
                        self.commands.entity(tooth).insert(
                            Transform::from_xyz(angle.sin() * 0.34, angle.cos() * 0.34, 0.)
                                .with_rotation(Quat::from_rotation_z(-angle)),
                        );
                        parts.push(tooth);
                    }
                }
            }
            ItemKind::Cell => {
                parts.push(self.block(Vec3::ZERO, Vec3::new(0.44, 0.64, 0.26), color));
                parts.push(self.block(Vec3::Y * 0.37, Vec3::new(0.20, 0.10, 0.18), white));
                for z in [-0.136, 0.136] {
                    parts.push(self.block(
                        Vec3::new(0., 0., z),
                        Vec3::new(0.24, 0.07, 0.012),
                        white,
                    ));
                    parts.push(self.block(
                        Vec3::new(0., 0., z),
                        Vec3::new(0.07, 0.24, 0.012),
                        white,
                    ));
                }
            }
            ItemKind::Blueprint => {
                parts.push(self.block(Vec3::ZERO, Vec3::new(0.60, 0.74, 0.065), color));
                for z in [-0.04, 0.04] {
                    for y in [-0.19, 0., 0.19] {
                        parts.push(self.block(
                            Vec3::new(-0.03, y, z),
                            Vec3::new(0.36, 0.035, 0.01),
                            white,
                        ));
                    }
                    parts.push(self.block(
                        Vec3::new(0.21, 0.28, z),
                        Vec3::new(0.16, 0.16, 0.015),
                        white,
                    ));
                }
            }
        }
        for &part in &parts {
            self.commands
                .entity(part)
                .insert(crate::player::blaster::ShotIgnore);
        }
        let root = self.spawn((
            Name::new(kind.label()),
            transform,
            Visibility::Inherited,
            crate::player::blaster::ShotIgnore,
        ));
        self.commands.entity(root).add_children(&parts);
        root
    }
}
