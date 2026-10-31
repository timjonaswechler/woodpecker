//! Convert the exported file representation into one checked level definition.
use super::{
    definition::{LevelDefinition, Room, StaticModule, Workshop},
    navigation::{Obstacle, Surface},
};
use serde::Deserialize;
use std::collections::HashSet;

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
            || !room.bounds.iter().all(|n| n.is_finite())
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
    ] {
        if !build_anchor(&station.base)
            || !build_anchor(&station.approach)
            || station.size.iter().any(|v| !v.is_finite() || *v <= 0.)
        {
            return Err("invalid workshop station".into());
        }
    }
    if raw.gears[0] != raw.workshop.part.base {
        return Err("workshop part and table positions differ".into());
    }
    Ok(LevelDefinition {
        rooms: raw.rooms,
        surfaces: raw.surfaces,
        obstacles: raw.obstacles,
        coins: raw.coins,
        blueprints: raw.blueprints,
        gears: raw.gears,
        modules: raw.modules,
        workshop: raw.workshop,
    })
}
