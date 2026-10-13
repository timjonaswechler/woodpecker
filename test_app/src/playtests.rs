//! Exercise the exported level with the same navigation and body used by the game.
use crate::{
    blueprint::{self as map, DATA, point},
    expedition::{CellState, Expedition, Interaction},
    level,
    navigation::{BODY_HEIGHT, Navigation},
    player::Player,
};
use bevy::prelude::*;

#[test]
fn visible_character_cannot_cross_a_wall_with_its_arms_or_head() {
    // Measure the real glTF hierarchy with exactly the pose and scale used at spawn.
    let model: serde_json::Value =
        serde_json::from_str(include_str!("../asset/Assets/gltf/Dummy_Base.gltf")).unwrap();
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
        vec![crate::navigation::Obstacle {
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
                p.x.abs() <= crate::navigation::BODY_HALF
                    && p.z.abs() <= crate::navigation::BODY_HALF
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
        vec![crate::navigation::Surface {
            center: [0., 0.],
            half: [2., 2.],
            height: 3.,
            bottom: 2.7,
        }],
        vec![crate::navigation::Obstacle {
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
    for p in [map::BOARD, DATA.gears[0].into(), map::RACK] {
        walk(&mut player, &nav, p);
    }
    nav.obstacles
        .iter_mut()
        .find(|o| o.name == "Movable storage rack")
        .unwrap()
        .center[2] -= 3.;
    nav.rebuild();
    for p in [Vec3::from(DATA.gears[1]), map::BENCH]
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
    assert!(!mission.finished && !mission.parts[1] && !mission.chest_open);
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
    use crate::{coins::CoinRun, items::ItemKind};
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
fn body_sweeps_stop_at_walls_and_land_on_boxes() {
    let nav = Navigation::new(
        Vec::new(),
        vec![crate::navigation::Obstacle {
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
