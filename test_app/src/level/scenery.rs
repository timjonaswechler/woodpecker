use crate::level::LevelAssets;
use bevy::prelude::*;
pub struct Scenery<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub assets: &'a AssetServer,
    pub tracked: &'a mut LevelAssets,
    pub meshes: &'a mut Assets<Mesh>,
    pub materials: &'a mut Assets<StandardMaterial>,
    pub parent: Option<Entity>,
}

impl Scenery<'_, '_, '_> {
    pub fn spawn(&mut self, bundle: impl Bundle) -> Entity {
        let entity = self.commands.spawn(bundle).id();
        if let Some(parent) = self.parent {
            self.commands.entity(parent).add_child(entity);
        }
        entity
    }

    pub fn model(&mut self, path: String, transform: Transform) -> Entity {
        let handle = self
            .assets
            .load(GltfAssetLabel::Scene(0).from_asset(path.clone()));
        self.tracked.0.push(handle.clone().untyped());
        self.spawn((Name::new(path), WorldAssetRoot(handle), transform))
    }

    pub fn kaykit(&mut self, name: &str, transform: Transform) -> Entity {
        self.model(format!("meshes/kaykit/{name}.gltf"), transform)
    }

    pub fn block(&mut self, position: Vec3, size: Vec3, color: Color) -> Entity {
        let mesh = self.meshes.add(Cuboid::from_size(size));
        let material = self.materials.add(StandardMaterial {
            base_color: color,
            alpha_mode: if color.alpha() < 1.0 {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            },
            ..default()
        });
        self.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(position),
        ))
    }
}
