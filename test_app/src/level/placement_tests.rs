//! Exercise the exported pilot through the game's real navigation and interaction rules.
use super::{definition as map, mission::Mission, *};
use crate::player::{Player, camera::ViewState};

#[test]
fn stair_connects_both_landings_by_route_and_body_movement() {
    let nav = navigation_world();
    let stair = &map::DATA.modules[0];
    let bottom = Vec3::from(stair.origin);
    let top = Vec3::from(stair.exit);
    let forward = (top - bottom).xz().normalize();
    let start = bottom - ground(forward, 0.);
    let end = top + ground(forward, 0.);
    assert!(nav.path(start, end).is_some());
    assert!(nav.path(end, start).is_some());
    let mut feet = start;
    for _ in 0..100 {
        feet = nav.move_body(feet, forward * 0.1, true);
    }
    assert!(feet.distance(end) < 0.01, "upstairs: {feet:?} != {end:?}");
    for _ in 0..100 {
        feet = nav.move_body(feet, -forward * 0.1, true);
    }
    assert!(
        feet.distance(start) < 0.01,
        "downstairs: {feet:?} != {start:?}"
    );
}

#[test]
fn workshop_front_aisle_and_all_three_approaches_are_clear() {
    let nav = navigation_world();
    let workshop = &map::DATA.workshop;
    for station in [&workshop.part, &workshop.craft, &workshop.output] {
        let approach = Vec3::from(station.approach);
        assert!(nav.walkable(approach), "blocked approach: {approach:?}");
        assert!(nav.path(map::START, approach).is_some());
    }
    let start = Vec3::from(workshop.part.approach);
    let end = Vec3::from(workshop.output.approach);
    assert!(nav.clear_segment(start, end));
    assert_eq!(map::DATA.gears()[0], workshop.part.base);
    assert_eq!(
        map::PICKUP_TABLES[0].center.xz(),
        Vec3::from(workshop.part.base).xz()
    );
    assert_eq!(map::PICKUP_TABLES[2].center.xz(), map::CELL_ITEMS[0].xz());
}

#[test]
fn workshop_guidance_reaches_the_actual_interaction_for_each_stage() {
    use mission::{CellId, Interaction, PartId};
    let nav = navigation_world();
    let mut mission = Mission::default();
    let mut player = Player::default();
    let view = ViewState::default();
    let mut visit = |mission: &Mission, goal: Vec3, expected| {
        assert!(nav.walkable(goal));
        player.respawn(goal.xz(), goal.y);
        assert_eq!(mission.interaction(&player, &nav, &view), expected);
    };
    visit(
        &mission,
        mission.destination(),
        Interaction::Part(PartId::ALL[0]),
    );
    mission
        .try_apply(Interaction::Part(PartId::ALL[0]))
        .unwrap();
    mission.try_apply(Interaction::Rack).unwrap();
    mission
        .try_apply(Interaction::Part(PartId::ALL[1]))
        .unwrap();
    assert_eq!(mission.destination(), *map::BENCH);
    visit(&mission, mission.destination(), Interaction::Craft);
    mission.try_apply(Interaction::Craft).unwrap();
    visit(
        &mission,
        mission.destination(),
        Interaction::Cell(CellId::ALL[0]),
    );
    mission
        .try_apply(Interaction::Cell(CellId::ALL[0]))
        .unwrap();
    assert_eq!(mission.carried_cells(), 1);
    mission.reset();
    assert_eq!(mission.destination(), map::PART_APPROACH[0]);
    assert_eq!(mission.carried_cells(), 0);
}

#[test]
fn malformed_pilot_metadata_is_rejected() {
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "../../asset/meshes/environment/courtyard.gltf"
    ))
    .unwrap();
    for (field, value) in [
        ("modules/0/quarter_turns", serde_json::json!(4)),
        ("modules/0/origin/0", serde_json::json!(1e100)),
        ("workshop/output/size/0", serde_json::json!(-1)),
        ("workshop/part/base/0", serde_json::json!(0)),
    ] {
        let mut doc = source.clone();
        *doc.pointer_mut(&format!("/extras/signalstation/layout/{field}"))
            .unwrap() = value;
        assert!(
            super::import::from_gltf(&doc.to_string()).is_err(),
            "{field}"
        );
    }
}
