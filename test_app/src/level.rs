use crate::{
    blueprint, camera::GameCamera, coins::Coin, navigation::Navigation, player::PlayerModel,
    scenery::Scenery, settings::Settings, ui::UiState,
};
use bevy::{light::CascadeShadowConfigBuilder, prelude::*};

pub const TEAL: Color = Color::srgb(0.15, 0.65, 0.59);
pub const GOLD: Color = Color::srgb(1.0, 0.73, 0.25);
#[derive(Resource, Default, PartialEq, Eq)]
pub enum LevelLoad {
    #[default]
    Loading,
    Ready,
    Failed,
}
impl LevelLoad {
    pub fn is_ready(&self) -> bool {
        *self == Self::Ready
    }
}
#[derive(Resource, Default)]
pub struct LevelAssets(pub Vec<UntypedHandle>);
pub fn ground(point: Vec2, height: f32) -> Vec3 {
    Vec3::new(point.x, height, point.y)
}
pub fn layout() -> Navigation {
    let mut obstacles: Vec<_> = blueprint::DATA
        .obstacles
        .iter()
        .filter(|o| !crate::stations::replaces(&o.name))
        .cloned()
        .collect();
    obstacles.extend(crate::stations::colliders());
    for (i, center) in crate::shooting::TARGETS.into_iter().enumerate() {
        for (size, position) in crate::shooting::stand_boxes(center) {
            obstacles.push(crate::navigation::Obstacle {
                name: format!("Target support {i}"),
                center: position.to_array(),
                half: (size * 0.5).to_array(),
            });
        }
    }
    Navigation::new(
        blueprint::DATA.surfaces.clone(),
        obstacles,
        Vec2::new(-39., -27.),
        Vec2::new(39., 27.),
    )
}
pub fn spawn_level(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut tracked: ResMut<LevelAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<Settings>,
) {
    let mut scene = Scenery {
        commands: &mut commands,
        assets: &assets,
        tracked: &mut tracked,
        meshes: &mut meshes,
        materials: &mut materials,
        parent: None,
    };
    scene.model(
        "custom/courtyard/courtyard.glb".into(),
        Transform::from_xyz(-39., 0., 27.),
    );
    let player = scene.kaykit(
        "Dummy_Base",
        Transform::from_translation(blueprint::START)
            .with_scale(Vec3::splat(crate::player::MODEL_SCALE)),
    );
    scene.commands.entity(player).insert(PlayerModel);
    for (i, pos) in blueprint::DATA.coins.into_iter().enumerate() {
        let coin = scene.kaykit(
            "Coin_A",
            Transform::from_translation(Vec3::from(pos) + Vec3::Y).with_scale(Vec3::splat(0.65)),
        );
        scene.commands.entity(coin).insert(Coin(i));
    }
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.94, 0.96, 1.0),
        brightness: 500.,
        ..default()
    });
    commands.spawn((
        DirectionalLight {
            illuminance: 12000.,
            shadow_maps_enabled: settings.shadows,
            ..default()
        },
        bevy::camera::visibility::RenderLayers::from_layers(&[0, 1]),
        CascadeShadowConfigBuilder {
            maximum_distance: 150.,
            ..default()
        }
        .build(),
        Transform::from_xyz(-18., 45., 30.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Name::new("Spielkamera"),
        GameCamera,
        Camera3d::default(),
        bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: 32.,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_translation(blueprint::START + Vec3::new(10., 32., 25.))
            .looking_at(blueprint::START, Vec3::Y),
        Msaa::Sample4,
    ));
}

pub fn check_assets(
    assets: Res<AssetServer>,
    tracked: Res<LevelAssets>,
    mut load: ResMut<LevelLoad>,
    mut ui: ResMut<UiState>,
) {
    if *load != LevelLoad::Loading {
        return;
    }
    for handle in &tracked.0 {
        if let Some(bevy::asset::LoadState::Failed(error)) = assets.get_load_state(handle.id()) {
            ui.status = format!("Asset konnte nicht geladen werden: {error}");
            *load = LevelLoad::Failed;
            error!("{}", ui.status);
            return;
        }
    }
    if !tracked.0.is_empty()
        && tracked
            .0
            .iter()
            .all(|h| assets.is_loaded_with_dependencies(h.id()))
    {
        *load = LevelLoad::Ready;
        ui.status =
            "Erkunde die acht Räume. Drei Energiezellen starten das Signal. E bedient Stationen."
                .into();
    }
}

pub fn update_lighting(settings: Res<Settings>, mut lights: Query<&mut DirectionalLight>) {
    for mut light in &mut lights {
        if light.shadow_maps_enabled != settings.shadows {
            light.shadow_maps_enabled = settings.shadows;
        }
    }
}
