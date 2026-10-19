use super::*;
#[test]
fn cells_require_tasks_and_allow_any_order_and_partial_delivery() {
    use crate::level::mission::ActionRejected;
    use Interaction::*;
    let mut mission = Mission::default();
    for action in [
        Signal,
        Craft,
        Cell(crate::level::mission::CellId::ALL[0]),
        Part(crate::level::mission::PartId::ALL[1]),
        Chest,
    ] {
        assert_eq!(mission.try_apply(action), Err(ActionRejected::Unavailable));
    }
    assert!(!mission.finished() && !mission.parts()[1] && !mission.chest_claimed());
    assert_eq!(mission.cells(), [CellState::Locked; 3]);
    for action in [
        Scan(crate::level::mission::ScanId::ALL[2]),
        Scan(crate::level::mission::ScanId::ALL[0]),
        Scan(crate::level::mission::ScanId::ALL[1]),
        Cell(crate::level::mission::CellId::ALL[2]),
        Install,
        Signal,
    ] {
        let _ = mission.try_apply(action);
    }
    assert_eq!(mission.installed_cells(), 1);
    assert!(!mission.finished());
    for action in [
        Valve(crate::level::mission::ValveId::ALL[1]),
        Valve(crate::level::mission::ValveId::ALL[0]),
        Cell(crate::level::mission::CellId::ALL[1]),
        Rack,
        Part(crate::level::mission::PartId::ALL[1]),
        Part(crate::level::mission::PartId::ALL[0]),
        Craft,
        Cell(crate::level::mission::CellId::ALL[0]),
        Install,
        Signal,
    ] {
        mission.try_apply(action).unwrap();
    }
    assert!(mission.finished());
    assert_eq!(mission.installed_cells(), 3);
    for action in [
        Scan(crate::level::mission::ScanId::ALL[1]),
        Valve(crate::level::mission::ValveId::ALL[0]),
        Craft,
        Cell(crate::level::mission::CellId::ALL[1]),
    ] {
        assert_eq!(mission.try_apply(action), Err(ActionRejected::Unavailable));
    }
    assert_eq!(
        mission.installed_cells(),
        3,
        "repeated interactions must not duplicate cells"
    );
    {
        let mut range = crate::level::objects::shooting_range::ShootingRange::default();
        for id in crate::level::mission::TargetId::ALL {
            range.hit(id, 0.0, 8.0).unwrap();
        }
        mission.complete_range(range.completion().unwrap());
    }
    mission.try_apply(Chest).unwrap();
    assert!(
        mission.blueprints()[5],
        "optional exploration continues after the signal"
    );
}

#[test]
fn resource_inventory_consumes_parts_and_delivered_cells() {
    use crate::level::mission::{ActionRejected, Outcome};
    use crate::{level::mission::inventory::CoinRun, level::mission::inventory::ItemKind};
    use Interaction::*;
    let mut mission = Mission::default();
    let mut coins = CoinRun::default();
    let counts = |m: &Mission, c: &CoinRun| ItemKind::ALL.map(|kind| kind.count(c, m));
    assert_eq!(counts(&mission, &coins), [0, 0, 0, 0]);
    mission
        .try_apply(Part(crate::level::mission::PartId::ALL[0]))
        .unwrap();
    coins.collect(crate::level::mission::CoinId::ALL[0]);
    assert_eq!(counts(&mission, &coins), [1, 1, 0, 0]);
    mission.try_apply(Rack).unwrap();
    mission
        .try_apply(Part(crate::level::mission::PartId::ALL[1]))
        .unwrap();
    assert_eq!(counts(&mission, &coins), [1, 2, 0, 0]);
    assert_eq!(mission.try_apply(Craft), Ok(Outcome::CellProduced));
    assert_eq!(
        counts(&mission, &coins),
        [1, 0, 0, 0],
        "cell still on workbench"
    );
    mission
        .try_apply(Cell(crate::level::mission::CellId::ALL[0]))
        .unwrap();
    assert_eq!(counts(&mission, &coins), [1, 0, 1, 0]);
    mission.try_apply(Install).unwrap();
    assert_eq!(counts(&mission, &coins), [1, 0, 0, 0]);
    {
        let mut range = crate::level::objects::shooting_range::ShootingRange::default();
        for id in crate::level::mission::TargetId::ALL {
            range.hit(id, 0.0, 8.0).unwrap();
        }
        mission.complete_range(range.completion().unwrap());
    }
    assert_eq!(mission.try_apply(Chest), Ok(Outcome::ChestClaimed));
    assert_eq!(mission.try_apply(Chest), Err(ActionRejected::Unavailable));
    assert_eq!(counts(&mission, &coins), [1, 0, 0, 1]);
}
