//! Exercise the exported level with the same navigation and body used by the game.
mod assets;

use crate::{
    expedition::{CellState, Expedition, Interaction},
    level,
    level::layout::{self as map, DATA, point},
    level::navigation::{BODY_HEIGHT, Navigation},
    player::Player,
};
use bevy::prelude::*;

#[test]
fn level_player_coins_and_camera_spawn_through_their_own_modules() {
    use crate::{items::coins, player, player::camera};
    use bevy::world_serialization::WorldAsset;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<WorldAsset>()
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .init_resource::<level::LevelAssets>()
        .init_resource::<crate::settings::Settings>();
    let world = app.world_mut();

    world.run_system_cached(level::spawn_level).unwrap();
    assert_eq!(world.query::<&WorldAssetRoot>().iter(world).count(), 1);
    assert_eq!(world.query::<&player::PlayerModel>().iter(world).count(), 0);
    assert_eq!(world.query::<&coins::Coin>().iter(world).count(), 0);
    assert_eq!(world.query::<&camera::GameCamera>().iter(world).count(), 0);

    world.run_system_cached(player::spawn_player).unwrap();
    let transform = world
        .query_filtered::<&Transform, With<player::PlayerModel>>()
        .single(world)
        .unwrap();
    assert_eq!(transform.translation, map::START);
    assert_eq!(transform.scale, Vec3::splat(player::MODEL_SCALE));

    world.run_system_cached(coins::spawn_coins).unwrap();
    let mut pickups: Vec<_> = world
        .query::<(&coins::Coin, &Transform)>()
        .iter(world)
        .map(|(coin, transform)| (coin.0, transform.translation))
        .collect();
    pickups.sort_by_key(|(index, _)| *index);
    assert_eq!(pickups.len(), DATA.coins.len());
    for (index, position) in pickups {
        assert_eq!(position, Vec3::from(DATA.coins[index]) + Vec3::Y);
    }
    assert_eq!(world.resource::<level::LevelAssets>().0.len(), 26);

    world.run_system_cached(camera::spawn_camera).unwrap();
    let transform = world
        .query_filtered::<&Transform, With<camera::GameCamera>>()
        .single(world)
        .unwrap();
    assert_eq!(transform.translation, map::START + Vec3::new(10., 32., 25.));
}

#[test]
fn reflected_resource_paths_match_the_session_contract() {
    use bevy::reflect::TypePath;

    assert_eq!(
        [
            level::LevelLoad::type_path(),
            Player::type_path(),
            crate::player::camera::ViewState::type_path(),
            Expedition::type_path(),
            crate::practice::Practice::type_path(),
            crate::items::coins::CoinRun::type_path(),
            crate::ui::UiState::type_path(),
            crate::settings::Settings::type_path(),
        ],
        [
            "test_app::level::LevelLoad",
            "test_app::player::Player",
            "test_app::player::camera::ViewState",
            "test_app::expedition::Expedition",
            "test_app::practice::Practice",
            "test_app::items::coins::CoinRun",
            "test_app::ui::UiState",
            "test_app::settings::Settings",
        ]
    );
}

#[test]
fn visible_character_cannot_cross_a_wall_with_its_arms_or_head() {
    // Measure the real glTF hierarchy with exactly the pose and scale used at spawn.
    let model: serde_json::Value =
        serde_json::from_str(include_str!("../asset/meshes/characters/Dummy_Base.gltf")).unwrap();
    fn bounds(model: &serde_json::Value, index: usize, parent: Mat4, vertices: &mut Vec<Vec3>) {
        let node = &model["nodes"][index];
        let translation = node["translation"].as_array().map_or(Vec3::ZERO, |v| {
            Vec3::new(
                v[0].as_f64().unwrap() as f32,
                v[1].as_f64().unwrap() as f32,
                v[2].as_f64().unwrap() as f32,
            )
        });
        let matrix = parent
            * Mat4::from_rotation_translation(
                crate::player::limb_rotation(node["name"].as_str().unwrap_or("")),
                translation,
            );
        if let Some(mesh) = node["mesh"].as_u64() {
            for primitive in model["meshes"][mesh as usize]["primitives"]
                .as_array()
                .unwrap()
            {
                let a = &model["accessors"]
                    [primitive["attributes"]["POSITION"].as_u64().unwrap() as usize];
                for x in 0..2 {
                    for y in 0..2 {
                        for z in 0..2 {
                            let value = |side, axis| {
                                a[if side == 0 { "min" } else { "max" }][axis]
                                    .as_f64()
                                    .unwrap() as f32
                            };
                            vertices.push(matrix.transform_point3(Vec3::new(
                                value(x, 0),
                                value(y, 1),
                                value(z, 2),
                            )));
                        }
                    }
                }
            }
        }
        if let Some(children) = node["children"].as_array() {
            for child in children {
                bounds(model, child.as_u64().unwrap() as usize, matrix, vertices);
            }
        }
    }
    let mut vertices = Vec::new();
    bounds(
        &model,
        5,
        Mat4::from_scale(Vec3::splat(crate::player::MODEL_SCALE)),
        &mut vertices,
    );
    let nav = Navigation::new(
        Vec::new(),
        vec![crate::level::navigation::Obstacle {
            name: "wall".into(),
            center: [0., 1.5, 0.],
            half: [0.1, 1.5, 3.],
        }],
        Vec2::splat(-8.),
        Vec2::splat(8.),
    );
    let mut player = Player::default();
    player.respawn(Vec2::new(-4., 0.), 0.);
    player.advance(&nav, Vec2::new(8., 0.), 0.05, false);
    for angle in 0..72 {
        let rotation = Quat::from_rotation_y(angle as f32 * std::f32::consts::TAU / 72.);
        for &vertex in &vertices {
            let p = rotation * vertex;
            assert!(player.position.x + p.x <= -0.1, "visible mesh crossed wall");
            assert!(
                p.x.abs() <= crate::level::navigation::BODY_HALF
                    && p.z.abs() <= crate::level::navigation::BODY_HALF
            );
            assert!(
                p.y + crate::player::MODEL_BOB <= BODY_HEIGHT,
                "head outside collision box"
            );
        }
    }
}

#[test]
fn box_corners_and_deck_edges_are_solid() {
    let nav = Navigation::new(
        vec![crate::level::navigation::Surface {
            center: [0., 0.],
            half: [2., 2.],
            height: 3.,
            bottom: 2.7,
        }],
        vec![crate::level::navigation::Obstacle {
            name: "corner".into(),
            center: [5., 1.5, 5.],
            half: [0.1, 1.5, 0.1],
        }],
        Vec2::splat(-10.),
        Vec2::splat(10.),
    );
    assert!(
        !nav.can_occupy(Vec3::new(4.4, 0., 4.4)),
        "box corner overlaps wall corner"
    );
    assert!(
        !nav.can_occupy(Vec3::new(2.3, 2., 0.)),
        "shoulder crosses side of deck"
    );
    assert!(
        nav.can_occupy(Vec3::new(0., 0., 0.)),
        "room below deck remains usable"
    );
}

fn walk(player: &mut Player, nav: &Navigation, target: Vec3) {
    assert!(nav.walkable(target), "blocked destination {target:?}");
    let route = nav
        .path(player.feet(), target)
        .unwrap_or_else(|| panic!("no route from {:?} to {target:?}", player.feet()));
    for waypoint in route {
        for _ in 0..2400 {
            if player.feet().distance(waypoint) < 0.035 {
                break;
            }
            player.advance(
                nav,
                (waypoint.xz() - player.position).clamp_length_max(0.24),
                0.05,
                false,
            );
        }
        assert!(
            player.feet().distance(waypoint) < 0.035,
            "body stuck at {:?}, heading to {waypoint:?}",
            player.feet()
        );
    }
    assert!(player.feet().distance(target) < 0.04);
}

#[test]
fn all_task_stations_and_optional_finds_are_reachable() {
    let mut nav = level::layout();
    let mut player = Player::default();
    for p in [map::BOARD, map::PART_APPROACH[0], map::RACK] {
        walk(&mut player, &nav, p);
    }
    nav.obstacles
        .iter_mut()
        .find(|o| o.name == "Movable storage rack")
        .unwrap()
        .center[2] -= 3.;
    nav.rebuild();
    for p in [map::PART_APPROACH[1], map::BENCH]
        .into_iter()
        .chain(map::VALVES)
        .chain(map::TABLETS)
        .chain(map::REWARDS)
        .chain([
            map::GENERATOR,
            map::SIGNAL,
            map::CHECKPOINT,
            map::PRACTICE,
            point(46., 49.5, 0.),
        ])
        .chain(DATA.blueprints[..5].iter().copied().map(Vec3::from))
        .chain(
            DATA.coins
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != 8)
                .map(|(_, p)| Vec3::from(*p)),
        )
    {
        walk(&mut player, &nav, p);
    }
}

#[test]
fn bridge_has_separate_lower_and_upper_routes_and_a_solid_underside() {
    let nav = level::layout();
    let lower = point(37., 33., 0.);
    let upper = point(37., 33., 3.);
    let mut player = Player::default();
    walk(&mut player, &nav, lower);
    for frame in 0..80 {
        player.advance(&nav, Vec2::ZERO, 0.02, frame == 0);
        assert!(player.height + BODY_HEIGHT <= 2.721, "head crossed deck");
    }
    assert_eq!(player.height, 0.);
    assert!(!nav.clear_segment(lower, upper));
    walk(&mut player, &nav, upper);
    walk(&mut player, &nav, lower);
}

#[test]
fn every_staircase_works_up_and_down_with_the_player_body() {
    let nav = level::layout();
    let mut player = Player::default();
    for (x, bottom, top) in [
        (12., 23., 31.),
        (42., 23., 31.),
        (71., 23., 31.),
        (59., 53.5, 47.5),
        (8., 51., 44.),
        (69., 3., 10.),
    ] {
        let direction: f32 = if top > bottom { 1. } else { -1. };
        let a = if x == 59. {
            point(57., 52.9, 0.)
        } else {
            point(x, bottom - direction * 0.8, 0.)
        };
        let b = point(x, top + direction * 0.5, 3.);
        walk(&mut player, &nav, a);
        walk(&mut player, &nav, b);
        walk(&mut player, &nav, a);
    }
}

#[test]
fn parkour_can_be_jumped_to_the_gallery() {
    let nav = level::layout();
    let mut player = Player::default();
    walk(&mut player, &nav, point(26., 14., 0.));
    for target in [
        point(26., 16., 0.6),
        point(27., 19., 1.2),
        point(29., 22., 1.8),
        point(31., 25., 2.4),
        point(32., 28., 3.),
        point(32., 32., 3.),
    ] {
        assert!(nav.walkable(target), "blocked parkour {target:?}");
        for frame in 0..90 {
            player.advance(
                &nav,
                (target.xz() - player.position).clamp_length_max(0.1),
                0.02,
                frame == 0,
            );
            if frame > 10 && player.feet().distance(target) < 0.04 && player.vertical_speed == 0. {
                break;
            }
        }
        assert!(
            player.feet().distance(target) < 0.04,
            "parkour landing {:?}, expected {target:?}",
            player.feet()
        );
    }
}

#[test]
fn cells_require_tasks_and_allow_any_order_and_partial_delivery() {
    use Interaction::*;
    let mut mission = Expedition::default();
    for action in [Signal, Craft, Cell(0), Part(1), Chest] {
        mission.apply(action);
    }
    assert!(!mission.finished && !mission.parts[1] && !mission.chest_claimed);
    assert_eq!(mission.cells, [CellState::Locked; 3]);
    for action in [Scan(2), Scan(0), Scan(1), Cell(2), Install, Signal] {
        mission.apply(action);
    }
    assert_eq!(mission.installed_cells(), 1);
    assert!(!mission.finished);
    for action in [
        Valve(1),
        Valve(0),
        Cell(1),
        Rack,
        Part(1),
        Part(0),
        Craft,
        Cell(0),
        Install,
        Signal,
    ] {
        mission.apply(action);
    }
    assert!(mission.finished);
    assert_eq!(mission.installed_cells(), 3);
    for action in [Scan(1), Valve(0), Craft, Cell(1)] {
        mission.apply(action);
    }
    assert_eq!(
        mission.installed_cells(),
        3,
        "repeated interactions must not duplicate cells"
    );
    mission.practice_reward = true;
    mission.apply(Chest);
    assert!(
        mission.blueprints[5],
        "optional exploration continues after the signal"
    );
}

#[test]
fn resource_inventory_consumes_parts_and_delivered_cells() {
    use crate::{items::ItemKind, items::coins::CoinRun};
    use Interaction::*;
    let mut mission = Expedition::default();
    let mut coins = CoinRun::default();
    let counts = |m: &Expedition, c: &CoinRun| ItemKind::ALL.map(|kind| kind.count(c, m));
    assert_eq!(counts(&mission, &coins), [0, 0, 0, 0]);
    mission.apply(Part(0));
    coins.collected[0] = true;
    assert_eq!(counts(&mission, &coins), [1, 1, 0, 0]);
    mission.apply(Rack);
    mission.apply(Part(1));
    assert_eq!(counts(&mission, &coins), [1, 2, 0, 0]);
    mission.apply(Craft);
    assert_eq!(
        counts(&mission, &coins),
        [1, 0, 0, 0],
        "cell still on workbench"
    );
    mission.apply(Cell(0));
    assert_eq!(counts(&mission, &coins), [1, 0, 1, 0]);
    mission.apply(Install);
    assert_eq!(counts(&mission, &coins), [1, 0, 0, 0]);
    mission.practice_reward = true;
    mission.apply(Chest);
    mission.apply(Chest);
    assert_eq!(counts(&mission, &coins), [1, 0, 0, 1]);
}

#[test]
fn archive_scanner_can_be_used_from_every_accessible_side() {
    let nav = level::layout();
    let mission = Expedition::default();
    let center = point(4., 50., 0.);
    for offset in [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z] {
        let position = center + offset * 1.6;
        assert!(nav.walkable(position), "test approach must be accessible");
        let mut player = Player::default();
        player.respawn(position.xz(), 0.);
        assert_eq!(
            mission.interaction(&player, &nav, &crate::player::camera::ViewState::default()),
            Interaction::Scan(0),
            "scanner unavailable from {offset:?}"
        );
        let direction = (center + Vec3::Y - (player.feet() + Vec3::Y * 1.5)).normalize();
        player.yaw = (-direction.x).atan2(-direction.z);
        let view = crate::player::camera::ViewState {
            first_person: true,
            captured: true,
            pitch: direction.y.asin(),
            eye: Some(
                Transform::from_translation(player.feet() + Vec3::Y * 1.5)
                    .looking_to(direction, Vec3::Y),
            ),
            ..default()
        };
        assert_eq!(
            mission.interaction(&player, &nav, &view),
            Interaction::Scan(0)
        );
    }
}

#[test]
fn interaction_requires_aim_reach_same_floor_and_no_wall() {
    let mut nav = level::layout();
    let mut mission = Expedition::default();
    let mut player = Player::default();
    player.respawn(point(4., 48.4, 0.).xz(), 0.);
    let mut view = crate::player::camera::ViewState {
        first_person: true,
        captured: true,
        pitch: -0.3,
        eye: Some(
            Transform::from_translation(player.feet() + Vec3::Y * 1.5)
                .with_rotation(Quat::from_rotation_x(-0.3)),
        ),
        ..default()
    };
    assert_eq!(
        mission.interaction(&player, &nav, &view),
        Interaction::Scan(0)
    );
    player.yaw = std::f32::consts::PI;
    view.eye.as_mut().unwrap().rotation =
        Quat::from_euler(EulerRot::YXZ, player.yaw, view.pitch, 0.);
    assert_eq!(mission.interaction(&player, &nav, &view), Interaction::None);
    player.yaw = 0.;
    view.eye.as_mut().unwrap().rotation = Quat::from_rotation_x(view.pitch);
    player.height = 3.;
    assert_eq!(mission.interaction(&player, &nav, &view), Interaction::None);
    player.height = 0.;
    player.position = point(4., 46., 0.).xz();
    assert_eq!(mission.interaction(&player, &nav, &view), Interaction::None);
    player.position = point(4., 48.4, 0.).xz();
    view.captured = false;
    assert_eq!(mission.interaction(&player, &nav, &view), Interaction::None);
    view.captured = true;
    nav.obstacles.push(crate::level::navigation::Obstacle {
        name: "test wall".into(),
        center: point(4., 49.2, 1.5).to_array(),
        half: [2., 1.5, 0.10],
    });
    assert_eq!(mission.interaction(&player, &nav, &view), Interaction::None);
    nav.obstacles.pop();
    mission.apply(Interaction::Scan(0));
    assert_eq!(
        mission.interaction(&player, &nav, &view),
        Interaction::None,
        "used scanner must not offer the same action again"
    );
    mission = Expedition::default();
    assert_eq!(
        mission.interaction(&player, &nav, &view),
        Interaction::Scan(0)
    );
}

#[test]
fn first_person_interactions_use_the_actual_head_camera_ray() {
    let nav = level::layout();
    let mission = Expedition::default();
    let mut player = Player::default();
    player.respawn(point(4., 48.4, 0.).xz(), 0.);
    let mut view = crate::player::camera::ViewState {
        first_person: true,
        captured: true,
        ..default()
    };
    assert_eq!(mission.interaction(&player, &nav, &view), Interaction::None);
    // A level ray above the scanner must miss, even though the old fixed
    // 1.5-metre origin and player's yaw/pitch would have selected it.
    view.eye = Some(Transform::from_translation(player.feet() + Vec3::Y * 2.));
    assert_eq!(mission.interaction(&player, &nav, &view), Interaction::None);
    view.eye.as_mut().unwrap().translation.y = 0.9;
    assert_eq!(
        mission.interaction(&player, &nav, &view),
        Interaction::Scan(0)
    );
    view.eye.as_mut().unwrap().rotation = Quat::from_rotation_y(std::f32::consts::PI);
    assert_eq!(mission.interaction(&player, &nav, &view), Interaction::None);
}

#[test]
fn body_sweeps_stop_at_walls_and_land_on_boxes() {
    let nav = Navigation::new(
        Vec::new(),
        vec![crate::level::navigation::Obstacle {
            name: "box".into(),
            center: [0., 0.5, 0.],
            half: [1., 0.5, 1.],
        }],
        Vec2::splat(-8.),
        Vec2::splat(8.),
    );
    let mut player = Player::default();
    player.respawn(Vec2::new(-4., 0.), 0.);
    player.advance(&nav, Vec2::new(8., 0.), 0.05, false);
    assert!(
        player.position.x < -1.37,
        "large movement tunneled through a box"
    );
    assert!(!nav.clear_segment(Vec3::new(-3., 0., 0.), Vec3::new(3., 0., 0.)));
    for dt in [0.05, 1. / 60.] {
        player.respawn(Vec2::ZERO, 2.5);
        for _ in 0..120 {
            player.advance(&nav, Vec2::ZERO, dt, false);
        }
        assert!(
            (player.height - 1.).abs() < 0.001,
            "player fell inside a box"
        );
        for _ in 0..120 {
            player.advance(&nav, Vec2::X * dt * 4., dt, false);
        }
        assert_eq!(player.height, 0., "player must drop back to the ground");
    }
}

#[test]
fn clicks_pick_the_visible_deck_and_reject_a_vertical_shortcut() {
    let nav = level::layout();
    let ray = Ray3d::new(point(37., 33., 10.), Dir3::NEG_Y);
    let target = nav.pick(ray).unwrap();
    assert!((target.y - 3.).abs() < 0.001);
    let route = nav.path(point(37., 33., 0.), target).unwrap();
    assert!(
        route.iter().any(|p| p.xz().distance(target.xz()) > 3.),
        "route must use a staircase"
    );
}
