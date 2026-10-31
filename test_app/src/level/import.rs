//! Convert the exported file representation into one checked level definition.
use super::{
    definition::{LevelDefinition, Room, StaticModule, StationPlacement, Workshop},
    navigation::{Obstacle, Surface},
    passages::{CutawayDefinition, PassageDefinition},
};
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};

#[derive(Deserialize)]
struct RawLevel {
    rooms: Vec<Room>,
    surfaces: Vec<Surface>,
    obstacles: Vec<Obstacle>,
    coins: [[f32; 3]; 24],
    blueprints: [[f32; 3]; 6],
    gears: [[f32; 3]; 2],
    modules: Vec<StaticModule>,
    workshop: Workshop,
    stations: BTreeMap<String, StationPlacement>,
    passages: [PassageDefinition; 3],
    cutaways: Vec<CutawayDefinition>,
}

// The one-metre construction grid applies to layout anchors, not mesh extents
// or decoration colliders. Those retain their original continuous coordinates.
fn build_anchor(point: &[f32; 3]) -> bool {
    point
        .iter()
        .all(|v| v.is_finite() && (v - v.round()).abs() < 0.00001)
}

pub(super) fn from_gltf(document: &str) -> Result<LevelDefinition, String> {
    let mut gltf: serde_json::Value =
        serde_json::from_str(document).map_err(|e| format!("glTF: {e}"))?;
    let metadata = gltf
        .pointer_mut("/extras/signalstation/layout")
        .ok_or("missing extras.signalstation.layout")?
        .take();
    let raw: RawLevel =
        serde_json::from_value(metadata).map_err(|e| format!("level layout: {e}"))?;
    let mut ids = HashSet::new();
    if raw.rooms.is_empty() || raw.surfaces.is_empty() {
        return Err("level needs rooms and walkable surfaces".into());
    }
    for room in &raw.rooms {
        let [x0, y0, x1, y1] = room.bounds;
        if room.id.trim().is_empty()
            || room.name.trim().is_empty()
            || !ids.insert(&room.id)
            || !room
                .bounds
                .iter()
                .all(|n| n.is_finite() && (n - n.round()).abs() < 0.00001)
            || x0 >= x1
            || y0 >= y1
        {
            return Err(format!("invalid or duplicate room {}", room.id));
        }
    }
    if !raw
        .obstacles
        .iter()
        .any(|o| o.name() == super::definition::RACK_COLLIDER)
    {
        return Err("missing movable rack collider".into());
    }
    let mut obstacles = HashSet::new();
    for obstacle in &raw.obstacles {
        if !obstacles.insert(obstacle.name()) {
            return Err(format!("duplicate obstacle {}", obstacle.name()));
        }
    }
    for (kind, positions) in [
        ("coins", raw.coins.as_slice()),
        ("blueprints", raw.blueprints.as_slice()),
        ("gears", raw.gears.as_slice()),
    ] {
        for (i, position) in positions.iter().enumerate() {
            if !position.iter().all(|n| n.is_finite()) {
                return Err(format!("{kind}[{i}] must have finite coordinates"));
            }
        }
    }
    let mut module_names = HashSet::new();
    for module in &raw.modules {
        if module.name.trim().is_empty()
            || module.asset.trim().is_empty()
            || !module_names.insert(&module.name)
            || module.quarter_turns > 3
            || !build_anchor(&module.origin)
            || !build_anchor(&module.exit)
            || module.origin == module.exit
        {
            return Err(format!("invalid module {}", module.name));
        }
    }
    for station in [
        &raw.workshop.part,
        &raw.workshop.craft,
        &raw.workshop.output,
    ]
    .into_iter()
    .chain(raw.stations.values())
    {
        if !build_anchor(&station.base)
            || !build_anchor(&station.approach)
            || station.size.iter().any(|v| !v.is_finite() || *v <= 0.)
        {
            return Err("invalid task station".into());
        }
    }
    if raw.gears[0] != raw.workshop.part.base {
        return Err("workshop part and table positions differ".into());
    }
    let keys = [
        "board",
        "craft",
        "rack",
        "part0",
        "part1",
        "valve0",
        "valve1",
        "scan0",
        "scan1",
        "scan2",
        "cell0",
        "cell1",
        "cell2",
        "generator",
        "signal",
        "checkpoint",
        "chest",
    ];
    if raw.stations.len() != keys.len() || keys.iter().any(|k| !raw.stations.contains_key(*k)) {
        return Err("missing task station placements".into());
    }
    if raw.stations["part0"] != raw.workshop.part
        || raw.stations["craft"] != raw.workshop.craft
        || raw.stations["cell0"] != raw.workshop.output
        || raw.gears[1] != raw.stations["part1"].base
    {
        return Err("task and pickup placements differ".into());
    }
    for (i, passage) in raw.passages.iter().enumerate() {
        if passage.id != format!("G{}", i + 1)
            || !matches!(passage.floor, 0. | 3.)
            || passage.volumes.is_empty()
            || passage.volumes.iter().any(|b| {
                !b.iter()
                    .all(|v| v.is_finite() && (v - v.round()).abs() < 0.00001)
                    || b[0] >= b[2]
                    || b[1] >= b[3]
            })
        {
            return Err("invalid secret passage volumes".into());
        }
        for portal in &passage.portals {
            let normal = bevy::prelude::Vec3::from(portal.inward);
            if !build_anchor(&portal.anchor)
                || !build_anchor(&portal.slide)
                || !portal.center.iter().all(|v| v.is_finite())
                || !portal.half.iter().all(|v| v.is_finite() && *v > 0.)
                || ![
                    bevy::prelude::Vec3::X,
                    bevy::prelude::Vec3::NEG_X,
                    bevy::prelude::Vec3::Z,
                    bevy::prelude::Vec3::NEG_Z,
                ]
                .contains(&normal)
                || bevy::prelude::Vec3::from(portal.slide).length() < 1.
                || normal.dot(portal.slide.into()).abs() > 0.00001
                || (portal.center[1] - portal.half[1] - passage.floor).abs() > 0.00001
                || !raw.obstacles.iter().any(|o| {
                    o.name() == portal.name
                        && o.center() == portal.center
                        && o.half() == portal.half
                })
            {
                return Err("invalid secret portal or missing collider".into());
            }
        }
    }
    let mut cuts = HashSet::new();
    let node_names: HashSet<_> = gltf["nodes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|n| n["name"].as_str())
        .collect();
    for cut in &raw.cutaways {
        if cut.secret >= 3
            || cut.portal < -1
            || cut.portal >= 6
            || (cut.portal >= 0 && cut.portal as usize / 2 != cut.secret)
            || !cuts.insert(cut.name.as_str())
            || !node_names.contains(cut.name.as_str())
            || cut
                .surface
                .is_some_and(|b| b.iter().any(|v| !v.is_finite()) || b[0] >= b[2] || b[1] >= b[3])
        {
            return Err("invalid passage cutaway".into());
        }
    }
    if (0..3).any(|i| !raw.cutaways.iter().any(|c| c.secret == i)) {
        return Err("missing passage covers".into());
    }
    Ok(LevelDefinition {
        rooms: raw.rooms,
        surfaces: raw.surfaces,
        obstacles: raw.obstacles,
        coins: raw.coins,
        blueprints: raw.blueprints,
        gears: raw.gears,
        modules: raw.modules,
        stations: raw.stations,
        passages: raw.passages,
        cutaways: raw.cutaways,
    })
}
