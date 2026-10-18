pub(crate) mod layout;
pub(crate) mod navigation;
pub mod scenery;

use crate::{settings::Settings, ui::UiState};
use bevy::{light::CascadeShadowConfigBuilder, prelude::*};
use navigation::Navigation;
use scenery::Scenery;

pub const TEAL: Color = Color::srgb(0.15, 0.65, 0.59);
pub const GOLD: Color = Color::srgb(1.0, 0.73, 0.25);
#[derive(Resource, Default, PartialEq, Eq, Reflect)]
#[reflect(Resource)]
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
    let mut obstacles: Vec<_> = layout::DATA
        .obstacles
        .iter()
        .filter(|o| !crate::expedition::stations::replaces(&o.name))
        .cloned()
        .collect();
    obstacles.extend(crate::expedition::stations::colliders());
    for (i, center) in crate::practice::TARGETS.into_iter().enumerate() {
        for (size, position) in crate::practice::stand_boxes(center) {
            obstacles.push(crate::level::navigation::Obstacle {
                name: format!("Target support {i}"),
                center: position.to_array(),
                half: (size * 0.5).to_array(),
            });
        }
    }
    Navigation::new(
        layout::DATA.surfaces.clone(),
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
        "meshes/courtyard/courtyard.gltf".into(),
        Transform::from_xyz(-39., 0., 27.),
    );
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

/// Runtime props replace the generic planning consoles and invisible trigger areas.
pub fn hide_planning_markers(
    ready: On<bevy::world_serialization::WorldInstanceReady>,
    children: Query<&Children>,
    mut nodes: Query<(&Name, &mut Visibility)>,
) {
    for entity in children.iter_descendants(ready.entity) {
        if let Ok((name, mut visibility)) = nodes.get_mut(entity)
            && (name.as_str().contains("interaction ring")
                || crate::expedition::stations::replaces(name.as_str()))
        {
            *visibility = Visibility::Hidden;
        }
    }
}
