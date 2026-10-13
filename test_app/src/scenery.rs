use crate::level::LevelAssets;
use bevy::prelude::*;
pub struct Scenery<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub assets: &'a AssetServer,
    pub tracked: &'a mut LevelAssets,
    pub meshes: &'a mut Assets<Mesh>,
    pub materials: &'a mut Assets<StandardMaterial>,
}

impl Scenery<'_, '_, '_> {
    pub fn model(&mut self, path: String, transform: Transform) -> Entity {
        let handle = self
            .assets
            .load(GltfAssetLabel::Scene(0).from_asset(path.clone()));
        self.tracked.0.push(handle.clone().untyped());
        self.commands
            .spawn((Name::new(path), WorldAssetRoot(handle), transform))
            .id()
    }

    pub fn kaykit(&mut self, name: &str, transform: Transform) -> Entity {
        self.model(format!("Assets/gltf/{name}.gltf"), transform)
    }

    pub fn block(&mut self, position: Vec3, size: Vec3, color: Color) -> Entity {
        self.commands
            .spawn((
                Mesh3d(self.meshes.add(Cuboid::from_size(size))),
                MeshMaterial3d(self.materials.add(StandardMaterial {
                    base_color: color,
                    alpha_mode: if color.alpha() < 1.0 {
                        AlphaMode::Blend
                    } else {
                        AlphaMode::Opaque
                    },
                    ..default()
                })),
                Transform::from_translation(position),
            ))
            .id()
    }

    pub fn disc(&mut self, position: Vec3, radius: f32, color: Color) -> Entity {
        self.commands
            .spawn((
                Mesh3d(self.meshes.add(Cylinder::new(radius, 0.045))),
                MeshMaterial3d(self.materials.add(StandardMaterial {
                    base_color: color,
                    alpha_mode: if color.alpha() < 1.0 {
                        AlphaMode::Blend
                    } else {
                        AlphaMode::Opaque
                    },
                    ..default()
                })),
                Transform::from_translation(position),
            ))
            .id()
    }
}
