//! Static environment setup and lighting.
use super::*;
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
        "meshes/environment/courtyard.gltf".into(),
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
            shadow_maps_enabled: settings.shadows(),
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

pub fn update_lighting(settings: Res<Settings>, mut lights: Query<&mut DirectionalLight>) {
    for mut light in &mut lights {
        if light.shadow_maps_enabled != settings.shadows() {
            light.shadow_maps_enabled = settings.shadows();
        }
    }
}

pub fn hide_planning_markers(
    ready: On<bevy::world_serialization::WorldInstanceReady>,
    children: Query<&Children>,
    mut nodes: Query<(&Name, &mut Visibility)>,
) {
    for entity in children.iter_descendants(ready.entity) {
        if let Ok((name, mut visibility)) = nodes.get_mut(entity)
            && (name.as_str().contains("interaction ring")
                || crate::level::objects::replaces(name.as_str()))
        {
            *visibility = Visibility::Hidden;
        }
    }
}
