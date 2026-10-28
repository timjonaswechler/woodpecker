//! Convert the exported file representation into one checked level definition.
use super::{
    definition::{LevelDefinition, Room},
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
    Ok(LevelDefinition {
        rooms: raw.rooms,
        surfaces: raw.surfaces,
        obstacles: raw.obstacles,
        coins: raw.coins,
        blueprints: raw.blueprints,
        gears: raw.gears,
    })
}
