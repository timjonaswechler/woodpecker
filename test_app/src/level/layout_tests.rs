//! Whole-level proofs through the real body, navigation and mission interfaces.
use super::mission::Interaction;
use super::{definition as map, mission::*, *};
use crate::player::{Player, camera::ViewState};

#[test]
fn all_six_stairs_join_their_floors_and_allow_body_travel_both_ways() {
    let nav = navigation_world();
    assert_eq!(map::DATA.modules.len(), 6);
    for stair in &map::DATA.modules {
        let bottom = Vec3::from(stair.origin);
        let top = Vec3::from(stair.exit);
        let forward = (top - bottom).xz().normalize();
        let start = bottom - ground(forward, 0.);
        let end = top + ground(forward, 0.);
        assert!(nav.path(start, end).is_some(), "up route: {}", stair.name);
        assert!(nav.path(end, start).is_some(), "down route: {}", stair.name);
        let mut player = Player::default();
        player.respawn(start.xz(), start.y);
        for _ in 0..125 {
            player.advance(&nav, forward * 0.08, 1. / 60., false);
        }
        assert!(
            player.feet().distance(end) < 0.02,
            "up {}: {:?} != {end:?}",
            stair.name,
            player.feet()
        );
        for _ in 0..125 {
            player.advance(&nav, -forward * 0.08, 1. / 60., false);
        }
        assert!(
            player.feet().distance(start) < 0.02,
            "down {}: {:?}",
            stair.name,
            player.feet()
        );
    }
}

#[test]
fn every_task_approach_is_clear_and_reachable_without_a_secret() {
    let nav = navigation_world();
    for (name, station) in &map::DATA.stations {
        let point = Vec3::from(station.approach);
        assert!(nav.walkable(point), "blocked {name}: {point:?}");
        assert!(nav.path(map::START, point).is_some(), "unreachable {name}");
    }
    assert!(
        nav.clear_segment(map::point(35., 44., 0.), map::point(35., 49., 0.)),
        "under G2"
    );
    assert!(
        nav.clear_segment(map::point(23., 33.5, 3.), map::point(69., 33.5, 3.)),
        "B/C/D upper route"
    );
    assert!(
        nav.path(map::point(63., 33., 3.), map::point(55., 42., 3.))
            .is_some(),
        "D/E upper route"
    );
}

#[test]
fn world_pick_stops_at_a_foreground_post_instead_of_the_floor_behind_it() {
    let nav = navigation_world();
    let direction = Dir3::new(Vec3::new(-0.3, -1., -0.8)).unwrap();
    let hidden = map::point(5.5, 46.5, 0.);
    let ray = Ray3d::new(hidden + Vec3::new(3., 10., 8.), direction);
    let blocker = nav
        .obstacles()
        .iter()
        .filter_map(|o| {
            navigation::box_hit(
                ray.origin,
                *ray.direction,
                o.center().into(),
                o.half().into(),
            )
            .map(|d| (d, o.name()))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .unwrap();
    assert!(blocker.1.starts_with("Gallery rail post"), "{blocker:?}");
    assert!(nav.pick(ray).is_none());
    let visible = map::point(5., 46.5, 0.);
    let ray = Ray3d::new(visible + Vec3::new(3., 10., 8.), direction);
    assert!(nav.pick(ray).unwrap().distance(visible) < 0.01);
}

#[test]
fn three_cells_and_signal_work_in_two_orders_without_optional_content() {
    for order in [[2, 1, 0], [1, 0, 2]] {
        let mut nav = navigation_world();
        let mut mission = Mission::default();
        let mut player = Player::default();
        let view = ViewState::default();
        let visit = |mission: &mut Mission,
                     nav: &mut navigation::Navigation,
                     player: &mut Player,
                     key: &str,
                     action: Interaction| {
            let point = Vec3::from(map::station(key).approach);
            assert!(nav.path(player.feet(), point).is_some(), "route to {key}");
            player.respawn(point.xz(), point.y);
            assert_eq!(
                mission.interaction(player, nav, &view),
                action,
                "interaction at {key}"
            );
            mission.try_apply(action).unwrap();
            objects::sync_collision(mission, nav, player);
        };
        for cell in order {
            match cell {
                0 => {
                    visit(
                        &mut mission,
                        &mut nav,
                        &mut player,
                        "part0",
                        Interaction::Part(PartId::ALL[0]),
                    );
                    visit(
                        &mut mission,
                        &mut nav,
                        &mut player,
                        "rack",
                        Interaction::Rack,
                    );
                    visit(
                        &mut mission,
                        &mut nav,
                        &mut player,
                        "part1",
                        Interaction::Part(PartId::ALL[1]),
                    );
                    visit(
                        &mut mission,
                        &mut nav,
                        &mut player,
                        "craft",
                        Interaction::Craft,
                    );
                }
                1 => {
                    for i in 0..2 {
                        visit(
                            &mut mission,
                            &mut nav,
                            &mut player,
                            if i == 0 { "valve0" } else { "valve1" },
                            Interaction::Valve(ValveId::ALL[i]),
                        );
                    }
                }
                _ => {
                    for i in 0..3 {
                        visit(
                            &mut mission,
                            &mut nav,
                            &mut player,
                            ["scan0", "scan1", "scan2"][i],
                            Interaction::Scan(ScanId::ALL[i]),
                        );
                    }
                }
            }
            visit(
                &mut mission,
                &mut nav,
                &mut player,
                ["cell0", "cell1", "cell2"][cell],
                Interaction::Cell(CellId::ALL[cell]),
            );
        }
        assert_eq!(mission.carried_cells(), 3);
        visit(
            &mut mission,
            &mut nav,
            &mut player,
            "generator",
            Interaction::Install,
        );
        visit(
            &mut mission,
            &mut nav,
            &mut player,
            "signal",
            Interaction::Signal,
        );
        assert!(mission.finished());
        assert_eq!(mission.secrets(), [false; 3]);
        assert_eq!(mission.portals(), [false; 6]);
        assert!(!mission.practice_reward());
    }
}
