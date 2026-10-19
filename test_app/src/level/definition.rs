//! Authoritative runtime data exported from the Blender planning model.
use crate::level::navigation::{Obstacle, Surface};
use bevy::prelude::*;
use serde::Deserialize;
use std::sync::LazyLock;
#[derive(Deserialize)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub bounds: [f32; 4],
}
pub struct LevelDefinition {
    pub(super) rooms: Vec<Room>,
    pub(super) surfaces: Vec<Surface>,
    pub(super) obstacles: Vec<Obstacle>,
    pub(super) coins: [[f32; 3]; 24],
    pub(super) blueprints: [[f32; 3]; 6],
    pub(super) gears: [[f32; 3]; 2],
}
impl LevelDefinition {
    pub fn rooms(&self) -> &[Room] {
        &self.rooms
    }
    pub fn surfaces(&self) -> &[Surface] {
        &self.surfaces
    }
    pub fn obstacles(&self) -> &[Obstacle] {
        &self.obstacles
    }
    pub fn coins(&self) -> [[f32; 3]; 24] {
        self.coins
    }
    pub fn blueprints(&self) -> [[f32; 3]; 6] {
        self.blueprints
    }
    pub fn gears(&self) -> [[f32; 3]; 2] {
        self.gears
    }
}
pub static DATA: LazyLock<LevelDefinition> = LazyLock::new(|| {
    super::import::from_gltf(include_str!(
        "../../asset/meshes/environment/courtyard.gltf"
    ))
    .expect("valid courtyard level definition")
});
pub const fn point(x: f32, y: f32, height: f32) -> Vec3 {
    Vec3::new(x - 39., height, 27. - y)
}
pub fn room(position: Vec2) -> Option<&'static Room> {
    let x = position.x + 39.;
    let y = 27. - position.y;
    DATA.rooms()
        .iter()
        .find(|r| x >= r.bounds[0] && x <= r.bounds[2] && y >= r.bounds[1] && y <= r.bounds[3])
}
pub const START: Vec3 = point(7., 5., 0.);
pub const CHECKPOINT: Vec3 = point(23., 22., 0.);
pub const BENCH: Vec3 = point(7.3, 31.5, 0.);
pub const RACK: Vec3 = point(59., 25.5, 0.);
pub const VALVES: [Vec3; 2] = [point(64., 41.7, 0.), point(73., 47.7, 0.)];
pub const TABLETS: [Vec3; 3] = [
    point(4., 48.5, 0.),
    point(16., 49.5, 0.),
    point(18.5, 40., 0.),
];
pub const PART_APPROACH: [Vec3; 2] = [point(6.7, 28., 0.), point(66., 24.4, 0.)];
pub const CELL_ITEMS: [Vec3; 3] = [
    point(4., 33., 1.32),
    point(71., 42., 1.32),
    point(17., 48., 1.32),
];
pub const REWARDS: [Vec3; 3] = [
    point(2.8, 31.3, 0.),
    point(71., 40.5, 0.),
    point(17., 46.5, 0.),
];
pub const GENERATOR: Vec3 = point(61., 5.5, 0.);
pub const SIGNAL: Vec3 = point(73., 11.8, 3.);
pub const RANGE_ENTRY: Vec3 = point(35., 41., 0.);
#[cfg(test)]
pub const BOARD: Vec3 = point(4., 11.5, 0.);

use crate::level::{mission::Interaction, objects::definitions::ObjectDefinition};
pub const CHEST_CENTER: Vec3 = point(46., 51., 0.4);
pub const CHEST_SIZE: Vec3 = Vec3::new(1.6, 0.8, 1.2);

pub const INTERACTIVE_OBJECTS: [ObjectDefinition; 12] = [
    ObjectDefinition::new(
        "Station board",
        Interaction::Board,
        point(4., 13., 0.),
        Vec3::new(1.4, 1.9, 0.6),
    ),
    ObjectDefinition::new(
        "Station craft",
        Interaction::Craft,
        point(6.6, 33., 0.),
        Vec3::new(1.3, 1.5, 0.9),
    ),
    ObjectDefinition::new(
        "Station rack",
        Interaction::Rack,
        point(59., 27., 0.),
        Vec3::new(0.9, 1.5, 0.8),
    ),
    ObjectDefinition::new(
        "Station valve 0",
        Interaction::Valve(crate::level::mission::ValveId::ALL[0]),
        point(64., 43., 0.),
        Vec3::new(1.0, 1.5, 0.55),
    ),
    ObjectDefinition::new(
        "Station valve 1",
        Interaction::Valve(crate::level::mission::ValveId::ALL[1]),
        point(73., 49., 0.),
        Vec3::new(1.0, 1.5, 0.55),
    ),
    ObjectDefinition::new(
        "Station scan 0",
        Interaction::Scan(crate::level::mission::ScanId::ALL[0]),
        point(4., 50., 0.),
        Vec3::new(1.3, 1.75, 0.9),
    ),
    ObjectDefinition::new(
        "Station scan 1",
        Interaction::Scan(crate::level::mission::ScanId::ALL[1]),
        point(16., 51., 0.),
        Vec3::new(1.3, 1.75, 0.9),
    ),
    ObjectDefinition::new(
        "Station scan 2",
        Interaction::Scan(crate::level::mission::ScanId::ALL[2]),
        point(20., 40., 0.),
        Vec3::new(1.3, 1.75, 0.9),
    ),
    ObjectDefinition::new(
        "Station generator",
        Interaction::Install,
        point(61., 7., 0.),
        Vec3::new(1.3, 1.5, 0.9),
    ),
    ObjectDefinition::new(
        "Station sender",
        Interaction::Signal,
        point(73., 13., 3.),
        Vec3::new(0.9, 1.25, 0.65),
    ),
    ObjectDefinition::new(
        "Station checkpoint",
        Interaction::Checkpoint,
        point(24.4, 22., 0.),
        Vec3::new(0.5, 1.4, 0.5),
    ),
    ObjectDefinition::new(
        "Blueprint reward chest",
        Interaction::Chest,
        point(46., 51., 0.),
        Vec3::new(1.6, 1.2, 1.2),
    ),
];
// Every table has one purpose: take the visible object. Assembly has its own switch.
pub const PICKUP_TABLES: [ObjectDefinition; 5] = [
    ObjectDefinition::new(
        "Table part 0",
        Interaction::Part(crate::level::mission::PartId::ALL[0]),
        point(5., 28., 0.),
        Vec3::new(1.1, 1.6, 1.1),
    ),
    ObjectDefinition::new(
        "Table part 1",
        Interaction::Part(crate::level::mission::PartId::ALL[1]),
        point(66., 26., 0.),
        Vec3::new(1.1, 1.6, 1.1),
    ),
    ObjectDefinition::new(
        "Table cell 0",
        Interaction::Cell(crate::level::mission::CellId::ALL[0]),
        point(4., 33., 0.),
        Vec3::new(4., 1.75, 2.),
    ),
    ObjectDefinition::new(
        "Table cell 1",
        Interaction::Cell(crate::level::mission::CellId::ALL[1]),
        point(71., 42., 0.),
        Vec3::new(1.8, 1.75, 1.3),
    ),
    ObjectDefinition::new(
        "Table cell 2",
        Interaction::Cell(crate::level::mission::CellId::ALL[2]),
        point(17., 48., 0.),
        Vec3::new(1.6, 1.75, 1.1),
    ),
];

pub const RANGE_TARGETS: [Vec3; 3] = [
    point(29., 51., 1.85),
    point(35., 51., 1.85),
    point(41., 51., 1.85),
];

pub(super) const RACK_COLLIDER: &str = "Movable storage rack";
