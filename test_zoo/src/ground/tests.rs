use super::*;

#[test]
fn ground_blends_only_shadows_and_never_writes_a_depth_prepass_or_casts_shadows() {
    let material = ShadowCatcher::default();
    assert_eq!(material.alpha_mode(), AlphaMode::Blend);
    assert!(!ShadowCatcher::enable_prepass());
    assert!(!ShadowCatcher::enable_shadows());
    assert!(!ShadowCatcher::enable_oit());
    assert_eq!(material.shadow_color.red, 0.);
    assert_eq!(material.shadow_color.green, 0.);
    assert_eq!(material.shadow_color.blue, 0.);
    assert!(material.shadow_color.alpha > 0. && material.shadow_color.alpha < 1.);
}

#[test]
fn ground_uses_a_plane_with_the_shadow_material_and_tracks_the_embedded_shader() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<Shader>()
        .init_asset_loader::<bevy::shader::ShaderLoader>()
        .init_resource::<LevelAssets>()
        .add_plugins(GroundPlugin);
    app.world_mut().run_system_cached(spawn).unwrap();
    let world = app.world_mut();
    let (transform, material, mesh, not_caster, picking) = world
        .query_filtered::<(
            &Transform,
            &MeshMaterial3d<ShadowCatcher>,
            &Mesh3d,
            Has<NotShadowCaster>,
            &Pickable,
        ), With<ShadowGround>>()
        .single(world)
        .unwrap();
    assert_eq!(transform.translation, Vec3::new(0., -0.04, 0.));
    assert!(not_caster);
    assert!(!picking.is_hoverable && !picking.should_block_lower);
    assert!(
        world
            .resource::<Assets<ShadowCatcher>>()
            .contains(&material.0)
    );
    let mesh = world.resource::<Assets<Mesh>>().get(&mesh.0).unwrap();
    assert_eq!(
        mesh.count_vertices(),
        4,
        "the catcher has no opaque side walls"
    );
    let tracked = &world.resource::<LevelAssets>().0;
    assert_eq!(tracked.len(), 1);
    assert_eq!(tracked[0].path().unwrap().to_string(), SHADER);
}

#[test]
fn shadow_shader_compiles_with_bevy_lighting_without_a_window_or_gpu() {
    use bevy::{
        render::{RenderApp, RenderPlugin, settings::WgpuSettings},
        shader::{ShaderCache, ShaderCacheError, ShaderCacheSource, ShaderDefVal},
    };
    use std::time::{Duration, Instant};

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<bevy::winit::WinitPlugin>()
            .disable::<bevy::log::LogPlugin>()
            .set(WindowPlugin {
                primary_window: None,
                ..default()
            })
            .set(RenderPlugin {
                render_creation: WgpuSettings {
                    backends: None,
                    ..default()
                }
                .into(),
                ..default()
            }),
    )
    .init_resource::<LevelAssets>()
    .add_plugins(GroundPlugin);
    assert!(
        app.get_sub_app(RenderApp).is_none(),
        "GPU backends must stay disabled"
    );
    app.finish();
    app.cleanup();
    let shader: Handle<Shader> = app.world().resource::<AssetServer>().load(SHADER);
    let mut cache = ShaderCache::new((), |_, source, _| {
        let ShaderCacheSource::Wgsl(source) = source else {
            panic!("expected compiled WGSL");
        };
        let module = naga::front::wgsl::parse_str(&source)
            .map_err(|error| ShaderCacheError::CreateShaderModule(error.emit_to_string(&source)))?;
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .map_err(|error| ShaderCacheError::CreateShaderModule(error.to_string()))?;
        Ok(module)
    });
    let defs = [
        ShaderDefVal::UInt("MATERIAL_BIND_GROUP".into(), 3),
        ShaderDefVal::UInt(
            "MAX_DIRECTIONAL_LIGHTS".into(),
            bevy::pbr::MAX_DIRECTIONAL_LIGHTS as u32,
        ),
        ShaderDefVal::UInt(
            "MAX_CASCADES_PER_LIGHT".into(),
            bevy::pbr::MAX_CASCADES_PER_LIGHT as u32,
        ),
        ShaderDefVal::UInt("MAX_RECT_LIGHTS".into(), bevy::pbr::MAX_RECT_LIGHTS as u32),
        "VERTEX_POSITIONS".into(),
        "VERTEX_NORMALS".into(),
        "VERTEX_UVS_A".into(),
        "SHADOW_FILTER_METHOD_GAUSSIAN".into(),
        "AVAILABLE_STORAGE_BUFFER_BINDINGS__GE_3".into(),
        "AVAILABLE_STORAGE_BUFFER_BINDINGS__GE_6".into(),
    ];
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        app.update();
        for (id, shader) in app.world().resource::<Assets<Shader>>().iter() {
            cache.set_shader(id, shader.clone());
        }
        match cache.get(0, shader.id(), &defs) {
            Ok(module) => {
                assert!(
                    module
                        .entry_points
                        .iter()
                        .any(|entry| entry.name == "fragment")
                );
                break;
            }
            Err(
                ShaderCacheError::ShaderNotLoaded(_)
                | ShaderCacheError::ShaderImportNotYetAvailable,
            ) if Instant::now() < deadline => {
                std::thread::yield_now();
            }
            Err(error) => panic!("shadow catcher shader failed to compile: {error:?}"),
        }
    }
}
