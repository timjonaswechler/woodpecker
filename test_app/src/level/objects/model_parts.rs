//! Shared geometry for the concrete object models.
use super::*;
use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology};
pub(super) fn housing(s: &mut Scenery, at: Vec3, size: Vec3, bevel: f32, color: Color) -> Entity {
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

pub(super) fn cylinder(
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

pub(super) fn mark(s: &mut Scenery, action: Interaction, at: Vec3, depth: f32) {
    for side in [-1., 1.] {
        let center = at + Vec3::Z * side * depth / 2.;
        s.block(center, Vec3::new(0.22, 0.22, 0.032), FRAME);
        let lamp = s.block(
            center + Vec3::Z * side * 0.024,
            Vec3::new(0.175, 0.175, 0.016),
            FRAME,
        );
        s.commands.entity(lamp).insert(ObjectLamp(action));
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

pub(super) fn plaque(s: &mut Scenery, label: &str, at: Vec3, width: f32, action: Interaction) {
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
        s.tracked.track(handle.clone().untyped());
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
            ObjectPlaque {
                action,
                textures: textures.clone(),
            },
            Transform::from_translation(at + Vec3::Z * side * 0.038)
                .with_rotation(Quat::from_rotation_y(if side < 0. { PI } else { 0. })),
        ));
    }
}

pub(super) fn labels(
    s: &mut Scenery,
    label: &str,
    height: f32,
    width: f32,
    depth: f32,
    action: Interaction,
) {
    for z in [-depth / 2., depth / 2.] {
        plaque(s, label, Vec3::new(0., height, z), width, action);
    }
}

pub(super) fn moving(
    s: &mut Scenery,
    e: Entity,
    action: Interaction,
    rest: Transform,
    used: Transform,
) {
    s.commands
        .entity(e)
        .insert((rest, Mechanism { action, rest, used }));
}
