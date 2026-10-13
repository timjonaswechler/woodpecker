//! Physical silhouettes, supported labels and small status lamps.
use super::*;
use crate::{interactions::Target, items::ItemKind};
use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology};
use std::f32::consts::{FRAC_PI_2, PI};

/// Eight-sided horizontal profile gives the station housings flat, chamfered corners.
fn housing(s: &mut Scenery, at: Vec3, size: Vec3, bevel: f32, color: Color) -> Entity {
    let h = size * 0.5;
    let b = bevel.min(h.x * 0.5).min(h.z * 0.5);
    let outline = [
        Vec2::new(-h.x + b, -h.z),
        Vec2::new(h.x - b, -h.z),
        Vec2::new(h.x, -h.z + b),
        Vec2::new(h.x, h.z - b),
        Vec2::new(h.x - b, h.z),
        Vec2::new(-h.x + b, h.z),
        Vec2::new(-h.x, h.z - b),
        Vec2::new(-h.x, -h.z + b),
    ];
    let mut positions = Vec::<[f32; 3]>::new();
    let mut normals = Vec::<[f32; 3]>::new();
    let mut triangle = |a: Vec3, b: Vec3, c: Vec3| {
        let normal = (b - a).cross(c - a).normalize().to_array();
        positions.extend([a.to_array(), b.to_array(), c.to_array()]);
        normals.extend([normal; 3]);
    };
    for i in 0..8 {
        let a = outline[i];
        let b = outline[(i + 1) % 8];
        let a0 = Vec3::new(a.x, -h.y, a.y);
        let a1 = Vec3::new(a.x, h.y, a.y);
        let b0 = Vec3::new(b.x, -h.y, b.y);
        let b1 = Vec3::new(b.x, h.y, b.y);
        triangle(Vec3::Y * h.y, b1, a1);
        triangle(-Vec3::Y * h.y, a0, b0);
        triangle(a0, a1, b1);
        triangle(a0, b1, b0);
    }
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    let mesh = s.meshes.add(mesh);
    let material = s.materials.add(StandardMaterial {
        base_color: color,
        perceptual_roughness: 0.75,
        ..default()
    });
    s.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(at),
    ))
}

fn cylinder(
    s: &mut Scenery,
    at: Vec3,
    radius: f32,
    height: f32,
    color: Color,
    rotation: Quat,
) -> Entity {
    let mesh = s
        .meshes
        .add(Cylinder::new(radius, height).mesh().resolution(12));
    let material = s.materials.add(color);
    s.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(at).with_rotation(rotation),
    ))
}

fn mark(s: &mut Scenery, action: Interaction, at: Vec3, depth: f32) {
    for side in [-1., 1.] {
        let center = at + Vec3::Z * side * depth / 2.;
        s.block(center, Vec3::new(0.22, 0.22, 0.032), FRAME);
        let lamp = s.block(
            center + Vec3::Z * side * 0.024,
            Vec3::new(0.175, 0.175, 0.016),
            FRAME,
        );
        s.commands.entity(lamp).insert(StationLamp(action));
        for (x, y, length, angle) in [
            (-0.047, -0.010, 0.094, 45_f32),
            (0.028, 0.015, 0.163, -38_f32),
        ] {
            let position = center + Vec3::new(x * side * 0.75, y * 0.75, side * 0.038);
            let e = s.block(
                position,
                Vec3::new(0.03, length * 0.75, 0.012),
                Color::WHITE,
            );
            s.commands.entity(e).insert((
                CompletionMark(action),
                Transform::from_translation(position)
                    .with_rotation(Quat::from_rotation_z(side * angle.to_radians())),
            ));
        }
    }
}

fn plaque(s: &mut Scenery, label: &str, at: Vec3, width: f32, action: Interaction) {
    let size = Vec2::new(width, width * 0.31);
    housing(
        s,
        at,
        Vec3::new(size.x + 0.04, size.y + 0.04, 0.07),
        0.012,
        FRAME,
    );
    let textures = ["", "_ready", "_done"].map(|suffix| {
        let handle: Handle<Image> = s.assets.load(format!("ui/stations/{label}{suffix}.png"));
        s.tracked.0.push(handle.clone().untyped());
        handle
    });
    let material = s.materials.add(StandardMaterial {
        base_color_texture: Some(textures[0].clone()),
        unlit: true,
        ..default()
    });
    let mesh = s.meshes.add(Rectangle::from_size(size));
    for side in [-1., 1.] {
        s.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            StationPlaque {
                action,
                textures: textures.clone(),
            },
            Transform::from_translation(at + Vec3::Z * side * 0.038)
                .with_rotation(Quat::from_rotation_y(if side < 0. { PI } else { 0. })),
        ));
    }
}

fn labels(s: &mut Scenery, label: &str, height: f32, width: f32, depth: f32, action: Interaction) {
    for z in [-depth / 2., depth / 2.] {
        plaque(s, label, Vec3::new(0., height, z), width, action);
    }
}

fn moving(s: &mut Scenery, e: Entity, action: Interaction, rest: Transform, used: Transform) {
    s.commands
        .entity(e)
        .insert((rest, Mechanism { action, rest, used }));
}

fn board(s: &mut Scenery) {
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

fn assembly(s: &mut Scenery, action: Interaction) {
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

fn lever(s: &mut Scenery, action: Interaction) {
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

fn valve(s: &mut Scenery, action: Interaction) {
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

fn scanner(s: &mut Scenery, index: usize, action: Interaction) {
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

fn generator(s: &mut Scenery, action: Interaction) {
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

fn sender(s: &mut Scenery, action: Interaction) {
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

fn checkpoint(s: &mut Scenery, action: Interaction) {
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

fn chest(s: &mut Scenery, action: Interaction) {
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

fn table(s: &mut Scenery, t: Target) {
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
        Interaction::Cell(0) => "cell_assembly",
        Interaction::Cell(1) => "cell_water",
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

/// The level and model zoo share exactly the same geometry and model roots.
pub fn spawn_model(s: &mut Scenery, target: Target) -> Entity {
    let base = Vec3::new(target.center.x, target.floor, target.center.z);
    let root = s.spawn((
        Name::new(target.name),
        Transform::from_translation(base),
        Visibility::Inherited,
    ));
    let previous = s.parent.replace(root);
    match target.action {
        Interaction::Board => board(s),
        Interaction::Craft => assembly(s, target.action),
        Interaction::Rack => lever(s, target.action),
        Interaction::Valve(_) => valve(s, target.action),
        Interaction::Scan(i) => scanner(s, i, target.action),
        Interaction::Install => generator(s, target.action),
        Interaction::Signal => sender(s, target.action),
        Interaction::Checkpoint => checkpoint(s, target.action),
        Interaction::Chest => chest(s, target.action),
        Interaction::Part(_) | Interaction::Cell(_) => table(s, target),
        Interaction::None => {}
    }
    s.parent = previous;
    root
}
