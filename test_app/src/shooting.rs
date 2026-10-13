use crate::{
    camera::{GameCamera, ViewState},
    level::{LevelAssets, LevelLoad, TEAL, ground},
    player::Player,
    ui::UiState,
};
use bevy::{
    camera::visibility::RenderLayers,
    input_focus::InputFocus,
    picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings, RayCastVisibility},
    prelude::*,
    text::EditableText,
    world_serialization::WorldInstanceReady,
};

pub const ENTRY: Vec2 = Vec2::new(crate::blueprint::PRACTICE.x, crate::blueprint::PRACTICE.z);
pub const TARGETS: [Vec3; 3] = [
    crate::blueprint::point(29., 51., 1.85),
    crate::blueprint::point(35., 51., 1.85),
    crate::blueprint::point(41., 51., 1.85),
];
pub const MIN_TARGET_DISTANCE: f32 = 6.0;
const TARGET_RADIUS: f32 = 0.8;
const TARGET_FRONT: f32 = 0.32;

/// A solid disk handles centre-line hits independently of glTF triangle seams.
/// The closest scene intersection still stops shots at walls and other props.
fn target_hit(
    origin: Vec3,
    direction: Vec3,
    limit: f32,
    destroyed: &[bool; 3],
) -> Option<(usize, Vec3)> {
    if direction.z >= -0.00001 {
        return None;
    }
    TARGETS
        .iter()
        .enumerate()
        .filter_map(|(i, &center)| {
            let distance = (center.z + TARGET_FRONT - origin.z) / direction.z;
            let point = origin + direction * distance;
            (!destroyed[i]
                && distance > 0.
                && distance <= limit + 0.001
                && (point - center).xy().length_squared() <= TARGET_RADIUS * TARGET_RADIUS)
                .then_some((i, point, distance))
        })
        .min_by(|a, b| a.2.total_cmp(&b.2))
        .map(|(i, p, _)| (i, p))
}
pub fn in_range(p: Vec2) -> bool {
    p.x > -15. && p.x < 11. && p.y > -27. && p.y < -11.
}

/// Always available. Each target stays broken until the level is reset.
#[derive(Resource, Default)]
pub struct Practice {
    pub targets_hit: [bool; 3],
    pub shots: u32,
    pub hits: u32,
    pub score: u32,
    pub cooldown: f32,
    pub hit_flash: f32,
    pub recoil: f32,
    pub tracer: Option<(Vec3, Vec3, f32)>,
    was_captured: bool,
}
impl Practice {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn hit(&mut self, index: usize, radius: f32, distance: f32) -> Option<u32> {
        if self.targets_hit.get(index) != Some(&false)
            || !distance.is_finite()
            || distance < MIN_TARGET_DISTANCE
        {
            return None;
        }
        let points = if radius < 0.25 {
            50
        } else if radius < 0.55 {
            25
        } else {
            10
        };
        self.score += points;
        self.hits += 1;
        self.targets_hit[index] = true;
        self.hit_flash = 0.18;
        Some(points)
    }
}

#[derive(Component)]
pub struct Target(pub usize);
#[derive(Component)]
pub struct TargetPart(pub usize);
#[derive(Component)]
pub struct Weapon;
#[derive(Component)]
pub struct WeaponCamera;
#[derive(Component)]
pub struct ShotIgnore;
#[derive(Component)]
pub struct Debris {
    velocity: Vec3,
    life: f32,
}
#[derive(Resource, Default)]
pub struct Fragments(Vec<Handle<bevy::world_serialization::WorldAsset>>);

pub fn stand_boxes(center: Vec3) -> [(Vec3, Vec3); 3] {
    [
        (
            Vec3::new(1.3, 0.2, 1.0),
            Vec3::new(center.x, 0.1, center.z - 0.35),
        ),
        (
            Vec3::new(0.22, 1.45, 0.22),
            Vec3::new(center.x, 0.825, center.z - 0.35),
        ),
        (
            Vec3::new(0.8, 0.16, 0.8),
            Vec3::new(center.x, 0.85, center.z - 0.1),
        ),
    ]
}

pub fn spawn_range(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut tracked: ResMut<LevelAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let stand_material = materials.add(Color::srgb(0.38, 0.45, 0.48));
    for (index, center) in TARGETS.into_iter().enumerate() {
        // KayKit's complete stand has a baked-in disk. Use a plain support so
        // only the destructible target remains visible above it.
        for (size, position) in stand_boxes(center) {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::from_size(size))),
                MeshMaterial3d(stand_material.clone()),
                Transform::from_translation(position),
            ));
        }
        let scene = assets.load(GltfAssetLabel::Scene(0).from_asset("Assets/gltf/target.gltf"));
        tracked.0.push(scene.clone().untyped());
        commands.spawn((
            Name::new(format!("Zielscheibe {}", index + 1)),
            Target(index),
            WorldAssetRoot(scene),
            Transform::from_translation(center).with_scale(Vec3::splat(1.6)),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(2.8, 0.035, 1.6))),
            MeshMaterial3d(materials.add(crate::items::ItemKind::Blueprint.color())),
            Transform::from_xyz(center.x, 0.025, ENTRY.y),
        ));
        // Each lane marks the minimum ground distance to its target.
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(4., 0.045, 0.12))),
            MeshMaterial3d(materials.add(crate::items::ItemKind::Blueprint.color())),
            Transform::from_xyz(center.x, 0.03, center.z + MIN_TARGET_DISTANCE),
        ));
    }
    {
        let mut scene = crate::scenery::Scenery {
            commands: &mut commands,
            assets: &assets,
            tracked: &mut tracked,
            meshes: &mut meshes,
            materials: &mut materials,
            parent: None,
        };
        for center in TARGETS {
            scene.item(
                crate::items::ItemKind::Blueprint,
                Transform::from_xyz(center.x, 0.65, center.z + 0.4).with_scale(Vec3::splat(0.55)),
            );
        }
    }
    let mut fragments = Fragments::default();
    for letter in ['A', 'B', 'C', 'D', 'E', 'F'] {
        let scene = assets.load(
            GltfAssetLabel::Scene(0).from_asset(format!("Assets/gltf/target_pieces_{letter}.gltf")),
        );
        tracked.0.push(scene.clone().untyped());
        fragments.0.push(scene);
    }
    commands.insert_resource(fragments);
    let weapon = assets.load(GltfAssetLabel::Scene(0).from_asset("custom/training_blaster.glb"));
    tracked.0.push(weapon.clone().untyped());
    commands.spawn((
        Name::new("Blender Trainingsblaster"),
        Weapon,
        ShotIgnore,
        WorldAssetRoot(weapon),
        Transform::default(),
        Visibility::Hidden,
        RenderLayers::layer(1),
    ));
    commands.spawn((
        Name::new("Blaster-Kamera"),
        WeaponCamera,
        Camera3d::default(),
        Camera {
            order: 1,
            is_active: false,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Projection::Perspective(PerspectiveProjection {
            fov: 65_f32.to_radians(),
            near: 0.01,
            ..default()
        }),
        RenderLayers::layer(1),
        Msaa::Sample4,
        bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted,
    ));
}

pub fn tag_scene_parts(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    roots: Query<(Option<&Target>, Has<Weapon>, Has<ShotIgnore>)>,
) {
    let Ok((target, weapon, ignored)) = roots.get(ready.entity) else {
        return;
    };
    for child in children.iter_descendants(ready.entity) {
        if let Some(target) = target {
            commands.entity(child).insert(TargetPart(target.0));
        }
        if weapon {
            commands.entity(child).insert(RenderLayers::layer(1));
        }
        if ignored {
            commands.entity(child).insert(ShotIgnore);
        }
    }
}

pub fn tick_practice(time: Res<Time>, mut practice: ResMut<Practice>) {
    let dt = time.delta_secs().min(0.05);
    practice.cooldown = (practice.cooldown - dt).max(0.0);
    practice.hit_flash = (practice.hit_flash - dt).max(0.0);
    practice.recoil = (practice.recoil - dt * 7.0).max(0.0);
    if let Some((_, _, ref mut ttl)) = practice.tracer {
        *ttl -= dt;
        if *ttl <= 0.0 {
            practice.tracer = None;
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Shooting reads Bevy input, scene meshes and target metadata."
)]
pub fn shoot(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    focus: Res<InputFocus>,
    inputs: Query<(), With<EditableText>>,
    player: Res<Player>,
    view: Res<ViewState>,
    mut ui: ResMut<UiState>,
    mut practice: ResMut<Practice>,
    load: Res<LevelLoad>,
    mut raycast: MeshRayCast,
    ignored: Query<(), With<ShotIgnore>>,
    parts: Query<&TargetPart>,
    fragments: Res<Fragments>,
) {
    // The click that captures the cursor or resumes the menu never fires a shot.
    let was_captured = practice.was_captured;
    practice.was_captured = view.first_person && view.captured && !ui.menu;
    if !buttons.just_pressed(MouseButton::Left)
        || !load.is_ready()
        || !view.first_person
        || !view.captured
        || ui.menu
        || !window.focused
        || !was_captured
        || focus.get().is_some_and(|e| inputs.contains(e))
    {
        return;
    }
    if practice.cooldown > 0.0 {
        return;
    }
    practice.shots += 1;
    practice.cooldown = 0.16;
    practice.recoil = 1.0;
    let rotation = Quat::from_euler(EulerRot::YXZ, player.yaw, view.pitch, 0.0);
    let origin = ground(player.position, player.height + 1.5);
    let direction = rotation * Vec3::NEG_Z;
    let filter = |entity| {
        !ignored.contains(entity)
            && parts
                .get(entity)
                .map_or(true, |part| !practice.targets_hit[part.0])
    };
    let hit = raycast
        .cast_ray(
            Ray3d::new(origin, Dir3::new_unchecked(direction)),
            &MeshRayCastSettings::default()
                .with_visibility(RayCastVisibility::Visible)
                .with_filter(&filter),
        )
        .first()
        .map(|(entity, hit)| (*entity, hit.point, hit.distance));
    let target = target_hit(
        origin,
        direction,
        hit.map_or(80., |(_, _, distance)| distance.min(80.)),
        &practice.targets_hit,
    );
    let impact = target.map(|(_, point)| point).unwrap_or_else(|| {
        hit.filter(|(_, _, distance)| *distance <= 80.0)
            .map_or(origin + direction * 80.0, |(_, point, _)| point)
    });
    practice.tracer = Some((
        origin + rotation * Vec3::new(0.25, -0.19, -0.84),
        impact,
        0.1,
    ));
    let Some((index, point)) = target else {
        return;
    };
    let center = TARGETS[index];
    let radius = (point - center).xy().length() / TARGET_RADIUS;
    // Ground distance prevents a balcony or jump from bypassing the firing line.
    let distance = player.position.distance(center.xz());
    if distance < MIN_TARGET_DISTANCE {
        ui.status = format!(
            "Zu nah: {distance:.1} m. Geh auf mindestens {MIN_TARGET_DISTANCE:.0} m Abstand zurück."
        );
        return;
    }
    if let Some(points) = practice.hit(index, radius, distance) {
        ui.status = format!(
            "{} +{points} Punkte · Triff jede der drei Scheiben.",
            if points == 50 {
                "Volltreffer!"
            } else {
                "Treffer!"
            }
        );
        for (index, fragment) in fragments.0.iter().enumerate() {
            let angle = index as f32 * std::f32::consts::TAU / 6.0;
            commands.spawn((
                WorldAssetRoot(fragment.clone()),
                ShotIgnore,
                Transform::from_translation(center).with_scale(Vec3::splat(1.6)),
                Debris {
                    velocity: Vec3::new(angle.sin() * 1.4, 1.8 + angle.cos(), 1.2),
                    life: 1.0,
                },
            ));
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "The render queries must be disjoint."
)]
pub fn update_visuals(
    mut commands: Commands,
    time: Res<Time>,
    player: Res<Player>,
    view: Res<ViewState>,
    load: Res<LevelLoad>,
    ui: Res<UiState>,
    practice: Res<Practice>,
    camera: Single<
        &Transform,
        (
            With<GameCamera>,
            Without<Weapon>,
            Without<WeaponCamera>,
            Without<Debris>,
        ),
    >,
    mut gun: Single<
        (&mut Transform, &mut Visibility),
        (With<Weapon>, Without<WeaponCamera>, Without<Debris>),
    >,
    mut overlay: Single<
        (&mut Transform, &mut Camera),
        (With<WeaponCamera>, Without<Weapon>, Without<Debris>),
    >,
    mut targets: Query<(&Target, &mut Visibility), Without<Weapon>>,
    mut debris: Query<(Entity, &mut Transform, &mut Debris)>,
    mut gizmos: Gizmos,
) {
    let visible = view.first_person && load.is_ready() && !ui.menu;
    *gun.1 = if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    overlay.1.is_active = visible;
    *overlay.0 = **camera;
    let bob = if view.captured {
        (player.walked * 6.0).sin() * 0.008
    } else {
        0.0
    };
    *gun.0 = Transform::from_translation(
        camera.translation
            + camera.rotation * Vec3::new(0.25, -0.25 + bob, -0.42 + practice.recoil * 0.075),
    )
    .with_rotation(camera.rotation * Quat::from_rotation_x(practice.recoil * 0.10))
    .with_scale(Vec3::splat(0.55));
    for (target, mut visibility) in &mut targets {
        *visibility = if practice.targets_hit[target.0] {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
    let dt = time.delta_secs().min(0.05);
    for (entity, mut transform, mut piece) in &mut debris {
        if ui.menu {
            continue;
        }
        piece.life -= dt;
        piece.velocity.y -= 6.0 * dt;
        transform.translation += piece.velocity * dt;
        transform.rotate_local_x(dt * 2.8);
        if piece.life <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
    if let Some((start, end, _)) = practice.tracer {
        gizmos.line(start, end, TEAL);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_faces_accept_straight_and_oblique_hits_but_respect_blockers() {
        let origin = ground(ENTRY, 1.5);
        for (i, center) in TARGETS.into_iter().enumerate() {
            let direction = (center - origin).normalize();
            assert_eq!(
                target_hit(origin, direction, 80., &[false; 3]).unwrap().0,
                i
            );
            assert!(
                target_hit(origin, direction, 2., &[false; 3]).is_none(),
                "a wall blocks the shot"
            );
            let mut broken = [false; 3];
            broken[i] = true;
            assert!(target_hit(origin, direction, 80., &broken).is_none());
        }
        let direction = (TARGETS[1] + Vec3::Y * 1.2 - origin).normalize();
        assert!(
            target_hit(origin, direction, 80., &[false; 3]).is_none(),
            "outside the disk"
        );
        assert!(
            target_hit(origin, Vec3::Z, 80., &[false; 3]).is_none(),
            "facing away"
        );
    }

    #[test]
    fn every_target_requires_distance_and_only_counts_once() {
        let mut practice = Practice::default();
        for i in 0..3 {
            for distance in [0., MIN_TARGET_DISTANCE - 0.01, f32::NAN] {
                assert_eq!(practice.hit(i, 0., distance), None);
                assert!(!practice.targets_hit[i]);
            }
            assert!(!practice.targets_hit.iter().all(|&hit| hit));
            assert_eq!(practice.hit(i, 0., MIN_TARGET_DISTANCE), Some(50));
            assert_eq!(practice.hit(i, 0., 20.), None);
        }
        assert!(practice.targets_hit.iter().all(|&hit| hit));
        assert_eq!(practice.hits, 3);
        assert_eq!(practice.score, 150);
        practice.reset();
        assert_eq!(practice.targets_hit, [false; 3]);
        assert_eq!(practice.score, 0);
        assert_eq!(practice.hit(2, 0.4, 8.), Some(25));
    }
}
