//! A composed level object: targets, scoring and effects.
mod scoring;
mod targets;
mod visuals;
use crate::level::mission::TargetId;
use crate::{level::LevelAssets, ui::UiState};
use bevy::prelude::*;
pub use targets::{ENTRY, MIN_TARGET_DISTANCE, TARGETS, in_range, stand_boxes};
pub use visuals::update_visuals;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum HitRejected {
    AlreadyBroken,
    InvalidSample,
    TooClose(f32),
}
pub struct RangeCompleted {
    _private: (),
}
/// Always available. Each target stays broken until the level is reset.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ShootingRange {
    targets_hit: [bool; 3],
    shots: u32,
    hits: u32,
    score: u32,
    hit_flash: f32,
}
impl ShootingRange {
    pub fn targets_hit(&self) -> [bool; 3] {
        self.targets_hit
    }
    pub fn shots(&self) -> u32 {
        self.shots
    }
    pub fn hits(&self) -> u32 {
        self.hits
    }
    pub fn score(&self) -> u32 {
        self.score
    }
    pub fn hit_flash(&self) -> f32 {
        self.hit_flash
    }
    pub fn completion(&self) -> Option<RangeCompleted> {
        self.targets_hit
            .iter()
            .all(|v| *v)
            .then_some(RangeCompleted { _private: () })
    }
}
#[derive(Component)]
pub struct Target(pub TargetId);
#[derive(Component)]
pub struct Debris {
    velocity: Vec3,
    life: f32,
}
#[derive(Resource, Default)]
pub struct Fragments(Vec<Handle<bevy::world_serialization::WorldAsset>>);
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
        let scene = assets.load(GltfAssetLabel::Scene(0).from_asset("meshes/targets/target.gltf"));
        tracked.track(scene.clone().untyped());
        commands.spawn((
            Name::new(format!("Zielscheibe {}", index + 1)),
            Target(TargetId::ALL[index]),
            WorldAssetRoot(scene),
            Transform::from_translation(center).with_scale(Vec3::splat(1.6)),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(2.8, 0.035, 1.6))),
            MeshMaterial3d(
                materials.add(crate::level::mission::inventory::ItemKind::Blueprint.color()),
            ),
            Transform::from_xyz(center.x, 0.025, ENTRY.y),
        ));
        // Each lane marks the minimum ground distance to its target.
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(4., 0.045, 0.12))),
            MeshMaterial3d(
                materials.add(crate::level::mission::inventory::ItemKind::Blueprint.color()),
            ),
            Transform::from_xyz(center.x, 0.03, center.z + MIN_TARGET_DISTANCE),
        ));
    }
    {
        let mut scene = crate::level::scene::Scenery {
            commands: &mut commands,
            assets: &assets,
            tracked: &mut tracked,
            meshes: &mut meshes,
            materials: &mut materials,
            parent: None,
        };
        for center in TARGETS {
            scene.item(
                crate::level::mission::inventory::ItemKind::Blueprint,
                Transform::from_xyz(center.x, 0.65, center.z + 0.4).with_scale(Vec3::splat(0.55)),
            );
        }
    }
    let mut fragments = Fragments::default();
    for letter in ['A', 'B', 'C', 'D', 'E', 'F'] {
        let scene = assets.load(
            GltfAssetLabel::Scene(0)
                .from_asset(format!("meshes/targets/target_pieces_{letter}.gltf")),
        );
        tracked.track(scene.clone().untyped());
        fragments.0.push(scene);
    }
    commands.insert_resource(fragments);
}
pub fn tick_feedback(time: Res<Time>, mut range: ResMut<ShootingRange>) {
    range.hit_flash = (range.hit_flash - time.delta_secs().min(0.05)).max(0.);
}
pub fn resolve_shot(
    mut shot: ResMut<crate::player::blaster::FiredShot>,
    mut blaster: ResMut<crate::player::blaster::Blaster>,
    mut range: ResMut<ShootingRange>,
    mut ui: ResMut<UiState>,
    mut commands: Commands,
    fragments: Res<Fragments>,
) {
    let Some(shot) = shot.take() else {
        return;
    };
    range.shots = range.shots.saturating_add(1);
    let Some((index, point)) =
        targets::target_hit(shot.origin, shot.direction, shot.limit, &range.targets_hit)
    else {
        return;
    };
    blaster.impact(point);
    let center = TARGETS[index.index()];
    let radius = (point - center).xy().length() / targets::TARGET_RADIUS;
    let distance = shot.ground.distance(center.xz());
    let points = match range.hit(index, radius, distance) {
        Ok(points) => points,
        Err(reason) => {
            if let Some(message) = crate::ui::messages::rejected_hit(reason) {
                ui.show(message);
            }
            return;
        }
    };
    ui.show(crate::ui::messages::hit(points));
    {
        for (index, fragment) in fragments.0.iter().enumerate() {
            let angle = index as f32 * std::f32::consts::TAU / 6.;
            commands.spawn((
                WorldAssetRoot(fragment.clone()),
                crate::player::blaster::ShotIgnore,
                Transform::from_translation(center).with_scale(Vec3::splat(1.6)),
                Debris {
                    velocity: Vec3::new(angle.sin() * 1.4, 1.8 + angle.cos(), 1.2),
                    life: 1.,
                },
            ));
        }
    }
}
