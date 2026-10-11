use crate::{
    game::{Game, GameCamera, PlayerModel},
    navigation::Navigation,
    settings::Settings,
};
use bevy::{light::CascadeShadowConfigBuilder, prelude::*};
use std::f32::consts::FRAC_PI_2;

pub const START: Vec2 = Vec2::new(0.0, 10.0);
pub const COINS: [Vec2; 6] = [
    Vec2::new(-10.0, 9.0),
    Vec2::new(-10.0, -9.0),
    Vec2::new(0.0, -10.0),
    Vec2::new(10.0, -9.0),
    Vec2::new(10.0, 8.0),
    Vec2::new(0.0, 0.0),
];
pub const TEAL: Color = Color::srgb(0.25, 0.91, 0.77);
pub const GOLD: Color = Color::srgb(1.0, 0.73, 0.25);

#[derive(Component)]
pub struct Coin(pub usize);

#[derive(Resource, Default)]
pub struct LevelAssets(pub Vec<UntypedHandle>);

pub fn ground(point: Vec2, height: f32) -> Vec3 {
    Vec3::new(point.x, height, point.y)
}

// The dimensions below match the glTF mesh bounds. Rendering and collision share this layout.
pub fn layout() -> Navigation {
    let mut nav = Navigation::default();
    nav.min.y = -26.0;
    // The old north wall now has a four-metre entrance to the shooting range.
    for x in [-12.0, -8.0, -4.0, 4.0, 8.0, 12.0] {
        nav.add(Vec2::new(x, -14.0), Vec2::new(2.0, 0.28));
    }
    for x in [-4.0, 4.0] {
        nav.add(Vec2::new(x, -22.0), Vec2::new(0.28, 4.0));
    }
    for x in [-8.0, 0.0, 8.0] {
        nav.add(Vec2::new(x, -23.4), Vec2::new(0.65, 0.5));
    }
    for (x, z, rotated) in WALLS {
        // Wall_Decorated extends from -0.25 to +0.663 along its local Z axis.
        nav.add(
            if rotated {
                Vec2::new(x + 0.207, z)
            } else {
                Vec2::new(x, z + 0.207)
            },
            if rotated {
                Vec2::new(0.457, 2.0)
            } else {
                Vec2::new(2.0, 0.457)
            },
        );
    }
    for (index, (x, z)) in CRATES.into_iter().enumerate() {
        nav.add(
            Vec2::new(x, z),
            if index % 2 == 0 {
                Vec2::splat(0.72)
            } else {
                Vec2::new(0.9, 0.62)
            },
        );
    }
    for (x, z) in BARRELS {
        nav.add(Vec2::new(x, z), Vec2::splat(0.52));
    }
    nav
}

const WALLS: [(f32, f32, bool); 10] = [
    (-5.0, 7.0, true),
    (-5.0, 3.0, true),
    (-5.0, -5.0, true),
    (-5.0, -9.0, true),
    (5.0, 9.0, true),
    (5.0, 5.0, true),
    (5.0, -3.0, true),
    (5.0, -7.0, true),
    (-3.0, -3.0, false),
    (3.0, 3.0, false),
];
const CRATES: [(f32, f32); 7] = [
    (-9.0, 3.0),
    (-10.5, 3.0),
    (-8.0, -4.0),
    (1.0, -6.0),
    (9.0, 0.0),
    (10.5, 0.0),
    (1.0, 5.0),
];
const BARRELS: [(f32, f32); 5] = [
    (-11.0, 0.0),
    (-8.0, -11.0),
    (3.0, -10.0),
    (11.0, 3.0),
    (-2.0, 6.0),
];

pub fn spawn_level(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut tracked: ResMut<LevelAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<Settings>,
) {
    let mut model = |name: &str, transform: Transform, commands: &mut Commands| {
        let handle =
            assets.load(GltfAssetLabel::Scene(0).from_asset(format!("Assets/gltf/{name}.gltf")));
        tracked.0.push(handle.clone().untyped());
        commands
            .spawn((
                Name::new(name.to_owned()),
                WorldAssetRoot(handle),
                transform,
            ))
            .id()
    };
    for z in -6..=3 {
        for x in -3..=3 {
            model(
                "Floor",
                Transform::from_xyz(x as f32 * 4.0, -0.5, z as f32 * 4.0),
                &mut commands,
            );
        }
    }
    for (x, z, rotated) in WALLS {
        model(
            "Wall_Decorated",
            Transform::from_xyz(x, 0.0, z)
                .with_scale(Vec3::new(1.0, 0.5, 1.0))
                .with_rotation(Quat::from_rotation_y(if rotated { FRAC_PI_2 } else { 0.0 })),
            &mut commands,
        );
    }
    for i in -3..=3 {
        let p = i as f32 * 4.0;
        for side in [-1.0, 1.0] {
            if side > 0.0 || i != 0 {
                model(
                    "Wall",
                    Transform::from_xyz(p, 0.0, side * 14.0).with_scale(Vec3::new(1.0, 0.22, 1.0)),
                    &mut commands,
                );
            }
            model(
                "Wall",
                Transform::from_xyz(side * 14.0, 0.0, p)
                    .with_scale(Vec3::new(1.0, 0.22, 1.0))
                    .with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
                &mut commands,
            );
        }
    }
    for x in -3..=3 {
        model(
            "Wall_Target",
            Transform::from_xyz(x as f32 * 4.0, 0.0, -26.0).with_scale(Vec3::new(1.0, 0.75, 1.0)),
            &mut commands,
        );
    }
    for z in [-16.0, -20.0, -24.0] {
        for x in [-14.0, 14.0] {
            model(
                "Wall",
                Transform::from_xyz(x, 0.0, z)
                    .with_scale(Vec3::new(1.0, 0.5, 1.0))
                    .with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
                &mut commands,
            );
        }
    }
    for x in [-4.0, 4.0] {
        for z in [-20.0, -24.0] {
            model(
                "Wall",
                Transform::from_xyz(x, 0.0, z)
                    .with_scale(Vec3::new(1.0, 0.5, 1.0))
                    .with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
                &mut commands,
            );
        }
    }
    for (i, (x, z)) in CRATES.into_iter().enumerate() {
        model(
            if i % 2 == 0 { "Box_A" } else { "Box_B" },
            Transform::from_xyz(x, 0.0, z).with_scale(Vec3::splat(3.0)),
            &mut commands,
        );
    }
    for (x, z) in BARRELS {
        model("Barrel_A", Transform::from_xyz(x, 0.5, z), &mut commands);
    }
    let player = model(
        "Dummy_Base",
        Transform::from_translation(ground(START, 0.0)).with_scale(Vec3::splat(1.2)),
        &mut commands,
    );
    commands.entity(player).insert(PlayerModel);

    for (i, position) in COINS.into_iter().enumerate() {
        let entity = model(
            "Coin_A",
            Transform::from_translation(ground(position, 1.0)).with_scale(Vec3::splat(0.65)),
            &mut commands,
        );
        commands.entity(entity).insert(Coin(i));
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.75, 0.035))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: GOLD,
                ..default()
            })),
            Transform::from_translation(ground(position, 0.02)),
        ));
    }
    commands.spawn((
        Name::new("Zielplattform"),
        Mesh3d(meshes.add(Cylinder::new(1.45, 0.05))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: TEAL,
            ..default()
        })),
        Transform::from_translation(ground(START, 0.03)),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(28.5, 0.8, 40.5))),
        MeshMaterial3d(materials.add(Color::srgb(0.13, 0.19, 0.22))),
        Transform::from_xyz(0.0, -0.9, -6.0),
    ));
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.8, 0.88, 1.0),
        brightness: 350.0,
        ..default()
    });
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadow_maps_enabled: settings.shadows,
            ..default()
        },
        bevy::camera::visibility::RenderLayers::from_layers(&[0, 1]),
        CascadeShadowConfigBuilder {
            maximum_distance: 70.0,
            ..default()
        }
        .build(),
        Transform::from_xyz(-12.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Name::new("Spielkamera"),
        GameCamera,
        Camera3d::default(),
        bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted,
        Projection::Perspective(PerspectiveProjection {
            fov: 50_f32.to_radians(),
            near: 0.05,
            far: 150.0,
            ..default()
        }),
        Transform::from_translation(ground(START, 0.0) + Vec3::new(0.0, 18.0, 14.0))
            .looking_at(ground(START, 0.0), Vec3::Y),
        Msaa::Sample4,
    ));
}

pub fn check_assets(assets: Res<AssetServer>, tracked: Res<LevelAssets>, mut game: ResMut<Game>) {
    if game.ready || game.asset_error {
        return;
    }
    for handle in &tracked.0 {
        if let Some(bevy::asset::LoadState::Failed(error)) = assets.get_load_state(handle.id()) {
            game.status = format!("Asset konnte nicht geladen werden: {error}");
            game.asset_error = true;
            error!("{}", game.status);
            return;
        }
    }
    if !tracked.0.is_empty()
        && tracked
            .0
            .iter()
            .all(|h| assets.is_loaded_with_dependencies(h.id()))
    {
        game.ready = true;
        game.status = "Sammle die sechs Münzen. Danach zurück zur grünen Plattform.".into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_collectible_and_return_route_is_reachable() {
        let nav = layout();
        for point in COINS {
            assert!(
                nav.path(START, point).is_some(),
                "unreachable coin {point:?}"
            );
            assert!(
                nav.path(point, START).is_some(),
                "cannot return from {point:?}"
            );
        }
    }

    #[test]
    fn range_entrance_and_all_three_firing_positions_are_reachable() {
        let nav = layout();
        for target in [
            crate::shooting::ENTRY,
            Vec2::new(-8.0, -17.0),
            Vec2::new(8.0, -17.0),
        ] {
            let route = nav
                .path(START, target)
                .expect("route through north entrance");
            let mut from = START;
            for to in route {
                assert!(nav.clear_segment(from, to));
                from = to;
            }
        }
        assert!(
            !nav.walkable(Vec2::new(8.0, -14.0)),
            "old north wall still blocks movement"
        );
        assert!(
            !nav.walkable(Vec2::new(4.0, -21.0)),
            "lane divider blocks movement"
        );
    }
}
