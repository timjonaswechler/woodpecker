//! Real forward-PBR qualification through the same public product/SDK seam.
use super::*;
use bevy::{
    camera::{Exposure, Hdr},
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap, ShadowFilteringMethod},
};

#[derive(Resource)]
struct Scene3d {
    cube: Entity,
    material: Handle<StandardMaterial>,
}

pub(super) fn install(app: &mut App, image: Handle<Image>, mode: &str) {
    app.insert_resource(GlobalAmbientLight {
        brightness: 0.0,
        ..default()
    })
    .insert_resource(DirectionalLightShadowMap { size: 512 });
    app.world_mut().spawn((
        Camera3d::default(),
        RenderTarget::Image(image.into()),
        Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::FRAC_PI_4,
            near: 0.1,
            far: 30.0,
            ..default()
        }),
        Transform::from_xyz(5.0, 5.0, 7.0).looking_at(Vec3::new(0.0, 0.4, 0.0), Vec3::Y),
        Hdr,
        Exposure::OVERCAST,
        Tonemapping::None,
        Msaa::Off,
        bevy::core_pipeline::tonemapping::DebandDither::Disabled,
        ShadowFilteringMethod::Hardware2x2,
    ));
    let plane = app
        .world_mut()
        .resource_mut::<Assets<Mesh>>()
        .add(Plane3d::default().mesh().size(10.0, 10.0));
    let ground = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.6, 0.6),
            perceptual_roughness: 1.0,
            reflectance: 0.0,
            ..default()
        });
    app.world_mut()
        .spawn((Mesh3d(plane), MeshMaterial3d(ground)));
    let mesh = app
        .world_mut()
        .resource_mut::<Assets<Mesh>>()
        .add(Cuboid::from_size(Vec3::splat(1.6)));
    let material = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.02, 0.02),
            perceptual_roughness: 1.0,
            reflectance: 0.0,
            ..default()
        });
    let cube = app
        .world_mut()
        .spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(-1.3, 0.8, 0.0),
        ))
        .id();
    app.world_mut().spawn((
        DirectionalLight {
            illuminance: if mode == "light-off-3d" {
                0.0
            } else {
                10_000.0
            },
            shadow_maps_enabled: mode != "shadow-off-3d",
            ..default()
        },
        Transform::from_xyz(-3.0, 6.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 25.0,
            ..default()
        }
        .build(),
    ));
    app.insert_resource(Scene3d { cube, material })
        .add_systems(Update, advance.after(super::update));
}

fn advance(
    counts: Res<Counts>,
    scene: Res<Scene3d>,
    mut transforms: Query<&mut Transform>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if counts.updates == 2 {
        transforms.get_mut(scene.cube).unwrap().translation.x = 1.3;
        materials.get_mut(&scene.material).unwrap().base_color = Color::srgb(0.02, 0.9, 0.02);
    }
}

fn colored(image: &image::RgbImage, channel: usize) -> Vec<(u32, u32)> {
    image
        .enumerate_pixels()
        .filter(|(_, _, p)| {
            p[channel] > 24
                && (0..3)
                    .filter(|&i| i != channel)
                    .all(|i| i16::from(p[channel]) > i16::from(p[i]) + 12)
        })
        .map(|(x, y, _)| (x, y))
        .collect()
}
fn geometry(image: &image::RgbImage, channel: usize) -> f32 {
    let points = colored(image, channel);
    assert!(
        points.len() >= 20,
        "real lit cube missing: {} colored pixels",
        points.len()
    );
    let gray = image
        .pixels()
        .filter(|p| p[0] > 30 && p[0].abs_diff(p[1]) <= 3 && p[0].abs_diff(p[2]) <= 3)
        .count();
    assert!(gray >= 300, "lit ground plane missing: {gray}");
    let min = points
        .iter()
        .map(|&(x, y)| image.get_pixel(x, y)[channel])
        .min()
        .unwrap();
    let max = points
        .iter()
        .map(|&(x, y)| image.get_pixel(x, y)[channel])
        .max()
        .unwrap();
    assert!(
        max - min >= 10,
        "cube has no face-lighting variation: {min}..{max}"
    );
    points.iter().map(|&(x, _)| x as f32).sum::<f32>() / points.len() as f32
}

pub(super) fn verify(session: &mut Session, root: &Path, mode: &str) {
    let before_tick = session
        .send(screenshot::Capture::new("before.png"))
        .unwrap();
    assert_eq!(
        outcome(session, before_tick).unwrap_err().code(),
        "render_initial_tick_required"
    );
    assert!(!root.join("before.png").exists());
    warp(session);
    capture(session, "a.png");
    let a = pixels(root, "a.png");
    let red_x = if mode == "light-off-3d" {
        assert!(
            a.pixels().all(|p| p.0.into_iter().all(|c| c <= 2)),
            "unlit control must be black with zero ambient/emission"
        );
        0.0
    } else {
        geometry(&a, 0)
    };
    let snapshot = std::fs::read(root.join("snapshot.json")).unwrap();
    capture(session, "pause.png");
    assert_eq!(
        pixels(root, "pause.png"),
        a,
        "paused 3D geometry/shadows changed"
    );
    assert_eq!(
        std::fs::read(root.join("snapshot.json")).unwrap(),
        snapshot,
        "3D pause advanced a clock/tick"
    );
    if mode == "device-loss-3d" {
        let warp = session
            .send(warp::Start {
                ticks: 1,
                pace: None,
            })
            .unwrap();
        let shot = session
            .send(screenshot::Capture::new("must-not-publish.png"))
            .unwrap();
        std::fs::write(root.join("fault-ready"), b"pending accepted").unwrap();
        let _ = outcome(session, warp);
        let error = outcome(session, shot).unwrap_err();
        assert!(matches!(error, session::Error::Render { .. }), "{error:?}");
        assert!(
            ["render_device_lost", "render_panicked"].contains(&error.code()),
            "{error:?}"
        );
        assert_eq!(
            serde_json::to_value(session.history().last().unwrap()).unwrap()["outcome"]["status"],
            "render_failed"
        );
        super::render_end(session, root, mode);
        return;
    }
    if mode == "healthy-3d" {
        warp(session);
        capture(session, "b.png");
        let b = pixels(root, "b.png");
        let green_x = geometry(&b, 1);
        assert!(
            green_x > red_x + 5.0,
            "cube did not move right: {red_x} -> {green_x}"
        );
        assert!(
            colored(&b, 0).is_empty(),
            "old red cube remained after Tick2"
        );
        assert!(
            a.pixels().zip(b.pixels()).filter(|(a, b)| a != b).count() >= 50,
            "3D pose/material did not change real pixels"
        );
        let final_state: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("snapshot.json")).unwrap()).unwrap();
        assert_eq!(final_state["updates"], 2);
    } else {
        assert!(
            matches!(mode, "shadow-off-3d" | "light-off-3d"),
            "unknown 3D case"
        );
    }
    session.shutdown().unwrap();
    println!("PASS Camera3d/StandardMaterial RGB, pause/time and explicit 3D case: {mode}");
}

/// Independent output comparisons: setting a light/shadow flag alone isn't proof.
pub(super) fn controls(lit_path: &Path, shadow_off_path: &Path, dark_path: &Path) {
    let load = |path: &Path| {
        let image = image::open(path).unwrap();
        assert_eq!(image.color(), image::ColorType::Rgb8);
        image.to_rgb8()
    };
    let lit = load(lit_path);
    let no_shadow = load(shadow_off_path);
    let dark = load(dark_path);
    assert_eq!(lit.dimensions(), (65, 49));
    assert_eq!(lit.dimensions(), no_shadow.dimensions());
    assert_eq!(lit.dimensions(), dark.dimensions());
    assert!(
        dark.pixels().all(|p| p.0.into_iter().all(|c| c <= 2)),
        "zero-light control isn't dark"
    );
    let gray =
        |p: &image::Rgb<u8>| p[0] > 30 && p[0].abs_diff(p[1]) <= 3 && p[0].abs_diff(p[2]) <= 3;
    let shadow_pixels = lit
        .pixels()
        .zip(no_shadow.pixels())
        .filter(|(shadow, plain)| {
            gray(plain)
                && shadow[0].abs_diff(shadow[1]) <= 3
                && shadow[0].abs_diff(shadow[2]) <= 3
                && i16::from(plain[0]) - i16::from(shadow[0]) >= 20
        })
        .count();
    assert!(
        shadow_pixels >= 6,
        "no visible cast shadow on ground: {shadow_pixels}"
    );
    assert_eq!(
        colored(&lit, 0),
        colored(&no_shadow, 0),
        "shadow control changed cube geometry"
    );
    println!(
        "PASS actual PBR light and cast-shadow effects: {shadow_pixels} ground pixels darkened"
    );
}
