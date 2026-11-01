//! An invisible ground surface that draws only the models' cast shadows.
use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    light::NotShadowCaster,
    picking::Pickable,
    prelude::*,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};
use test_app::level::LevelAssets;

const SHADER: &str = "embedded://test_zoo/ground/shadow_catcher.wesl";

pub struct GroundPlugin;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "ground/shadow_catcher.wesl");
        app.add_plugins(MaterialPlugin::<ShadowCatcher>::default())
            .add_systems(Startup, spawn);
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct ShadowCatcher {
    #[uniform(0)]
    shadow_color: LinearRgba,
}

impl Default for ShadowCatcher {
    fn default() -> Self {
        Self {
            shadow_color: LinearRgba::new(0., 0., 0., 0.55),
        }
    }
}

impl Material for ShadowCatcher {
    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn enable_oit() -> bool {
        false
    }
}

#[derive(Component)]
pub struct ShadowGround;

pub fn spawn(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut tracked: ResMut<LevelAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ShadowCatcher>>,
) {
    let shader: Handle<Shader> = load_embedded_asset!(&*assets, "ground/shadow_catcher.wesl");
    tracked.track(shader.untyped());
    commands.spawn((
        Name::new("Transparenter Schattenfänger"),
        ShadowGround,
        NotShadowCaster,
        Pickable::IGNORE,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(200., 200.))),
        MeshMaterial3d(materials.add(ShadowCatcher::default())),
        Transform::from_xyz(0., -0.04, 0.),
    ));
}
