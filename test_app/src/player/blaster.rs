//! The player's blaster emits one shot; world objects evaluate its result.
use super::camera::{GameCamera, ViewState};
use crate::{
    app::flow::GameFlow,
    level::{LevelAssets, LevelLoad, TEAL},
};
use bevy::{
    camera::visibility::RenderLayers,
    picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings, RayCastVisibility},
    prelude::*,
    world_serialization::WorldInstanceReady,
};

#[derive(Resource, Default)]
pub struct Blaster {
    cooldown: f32,
    recoil: f32,
    tracer: Option<(Vec3, Vec3, f32)>,
}
#[derive(Component)]
pub struct Weapon;
#[derive(Component)]
pub struct WeaponCamera;
#[derive(Component)]
pub struct ShotIgnore;

pub(crate) struct Shot {
    pub origin: Vec3,
    pub direction: Vec3,
    pub limit: f32,
    pub ground: Vec2,
}
#[derive(Resource, Default)]
pub struct FiredShot(Option<Shot>);
impl FiredShot {
    pub(crate) fn take(&mut self) -> Option<Shot> {
        self.0.take()
    }
}
impl Blaster {
    #[cfg(test)]
    pub fn tracer(&self) -> Option<(Vec3, Vec3, f32)> {
        self.tracer
    }
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub(crate) fn impact(&mut self, point: Vec3) {
        if let Some((_, end, _)) = &mut self.tracer {
            *end = point;
        }
    }
}

pub fn spawn(mut commands: Commands, assets: Res<AssetServer>, mut tracked: ResMut<LevelAssets>) {
    let weapon =
        assets.load(GltfAssetLabel::Scene(0).from_asset("meshes/weapons/training_blaster.gltf"));
    tracked.track(weapon.clone().untyped());
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
    roots: Query<(Has<Weapon>, Has<ShotIgnore>)>,
) {
    let Ok((weapon, ignored)) = roots.get(ready.entity) else {
        return;
    };
    for child in children.iter_descendants(ready.entity) {
        if weapon {
            commands.entity(child).insert(RenderLayers::layer(1));
        }
        if ignored {
            commands.entity(child).insert(ShotIgnore);
        }
    }
}

pub fn tick(time: Res<Time>, mut blaster: ResMut<Blaster>) {
    let dt = time.delta_secs().min(0.05);
    blaster.cooldown = (blaster.cooldown - dt).max(0.);
    blaster.recoil = (blaster.recoil - dt * 7.).max(0.);
    if let Some((_, _, ttl)) = &mut blaster.tracer {
        *ttl -= dt;
        if *ttl <= 0. {
            blaster.tracer = None;
        }
    }
}

pub fn fire(
    frame: Res<crate::input::intents::FrameInput>,
    player: Res<super::Player>,
    view: Res<ViewState>,
    mut blaster: ResMut<Blaster>,
    mut shot: ResMut<FiredShot>,
    mut raycast: MeshRayCast,
    ignored: Query<(), With<ShotIgnore>>,
) {
    shot.0 = None;
    if !frame.fire() || blaster.cooldown > 0. {
        return;
    }
    let Some(eye) = view.eye() else {
        return;
    };
    let origin = eye.translation;
    let direction = eye.forward().as_vec3();
    let filter = |entity| !ignored.contains(entity);
    let hit = raycast
        .cast_ray(
            Ray3d::new(origin, eye.forward()),
            &MeshRayCastSettings::default()
                .with_visibility(RayCastVisibility::Visible)
                .with_filter(&filter),
        )
        .first()
        .map(|(_, hit)| (hit.point, hit.distance));
    let limit = hit.map_or(80., |(_, distance)| distance.min(80.));
    let impact = origin + direction * limit;
    blaster.cooldown = 0.16;
    blaster.recoil = 1.;
    blaster.tracer = Some((
        origin + eye.rotation * Vec3::new(0.25, -0.19, -0.84),
        impact,
        0.1,
    ));
    shot.0 = Some(Shot {
        origin,
        direction,
        limit,
        ground: player.position,
    });
}

#[expect(
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "Disjoint Bevy queries update the player camera, weapon and overlay."
)]
pub fn update_visuals(
    view: Res<ViewState>,
    load: Res<LevelLoad>,
    flow: Res<GameFlow>,
    blaster: Res<Blaster>,
    camera: Single<&Transform, (With<GameCamera>, Without<Weapon>, Without<WeaponCamera>)>,
    mut gun: Single<(&mut Transform, &mut Visibility), (With<Weapon>, Without<WeaponCamera>)>,
    mut overlay: Single<(&mut Transform, &mut Camera), (With<WeaponCamera>, Without<Weapon>)>,
    mut gizmos: Gizmos,
) {
    let visible = view.first_person() && view.eye().is_some() && load.is_ready() && !flow.menu();
    *gun.1 = if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    overlay.1.is_active = visible;
    // GameCamera is local to the head in FPV; the overlay uses its world pose.
    let camera = view.eye().unwrap_or(**camera);
    *overlay.0 = camera;
    *gun.0 = Transform::from_translation(
        camera.translation
            + camera.rotation * Vec3::new(0.25, -0.25, -0.42 + blaster.recoil * 0.075),
    )
    .with_rotation(camera.rotation * Quat::from_rotation_x(blaster.recoil * 0.10))
    .with_scale(Vec3::splat(0.55));
    if let Some((start, end, _)) = blaster.tracer {
        gizmos.line(start, end, TEAL);
    }
}
