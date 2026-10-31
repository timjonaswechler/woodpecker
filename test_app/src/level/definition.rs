//! Authoritative construction, task placement and passages exported from Blender.
use super::{
    mission::{CellId, Interaction, PartId, ScanId, ValveId},
    objects::definitions::ObjectDefinition,
    passages::{CutawayDefinition, PassageDefinition},
};
use crate::level::navigation::{Obstacle, Surface};
use bevy::prelude::*;
use serde::Deserialize;
use std::{collections::BTreeMap, sync::LazyLock};

#[derive(Deserialize)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub bounds: [f32; 4],
}
#[derive(Deserialize)]
pub(super) struct StaticModule {
    pub name: String,
    pub asset: String,
    pub origin: [f32; 3],
    pub quarter_turns: u8,
    pub exit: [f32; 3],
}
#[derive(Deserialize, PartialEq)]
pub(super) struct StationPlacement {
    pub base: [f32; 3],
    pub size: [f32; 3],
    pub approach: [f32; 3],
}
impl StationPlacement {
    fn object(&self, name: &'static str, action: Interaction) -> ObjectDefinition {
        ObjectDefinition::new(name, action, self.base.into(), self.size.into())
    }
}
#[derive(Deserialize)]
pub(super) struct Workshop {
    pub part: StationPlacement,
    pub craft: StationPlacement,
    pub output: StationPlacement,
}
pub struct LevelDefinition {
    pub(super) rooms: Vec<Room>,
    pub(super) surfaces: Vec<Surface>,
    pub(super) obstacles: Vec<Obstacle>,
    pub(super) coins: [[f32; 3]; 24],
    pub(super) blueprints: [[f32; 3]; 6],
    pub(super) gears: [[f32; 3]; 2],
    pub(super) modules: Vec<StaticModule>,
    pub(super) stations: BTreeMap<String, StationPlacement>,
    pub(super) passages: [PassageDefinition; 3],
    pub(super) cutaways: Vec<CutawayDefinition>,
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
pub(super) fn station(name: &str) -> &'static StationPlacement {
    DATA.stations.get(name).expect("checked task station")
}
fn approach(name: &str) -> Vec3 {
    station(name).approach.into()
}
pub const START: Vec3 = point(7., 5., 0.);
pub static CHECKPOINT: LazyLock<Vec3> = LazyLock::new(|| approach("checkpoint"));
pub static BENCH: LazyLock<Vec3> = LazyLock::new(|| approach("craft"));
pub static RACK: LazyLock<Vec3> = LazyLock::new(|| approach("rack"));
pub static VALVES: LazyLock<[Vec3; 2]> = LazyLock::new(|| [approach("valve0"), approach("valve1")]);
pub static TABLETS: LazyLock<[Vec3; 3]> =
    LazyLock::new(|| [approach("scan0"), approach("scan1"), approach("scan2")]);
pub static PART_APPROACH: LazyLock<[Vec3; 2]> =
    LazyLock::new(|| [approach("part0"), approach("part1")]);
pub static CELL_ITEMS: LazyLock<[Vec3; 3]> = LazyLock::new(|| {
    ["cell0", "cell1", "cell2"].map(|s| Vec3::from(station(s).base) + Vec3::Y * 1.32)
});
pub static REWARDS: LazyLock<[Vec3; 3]> =
    LazyLock::new(|| [approach("cell0"), approach("cell1"), approach("cell2")]);
pub static GENERATOR: LazyLock<Vec3> = LazyLock::new(|| approach("generator"));
pub static SIGNAL: LazyLock<Vec3> = LazyLock::new(|| approach("signal"));
pub const RANGE_ENTRY: Vec3 = point(35., 41., 0.);
pub static CHEST_CENTER: LazyLock<Vec3> =
    LazyLock::new(|| Vec3::from(station("chest").base) + Vec3::Y * 0.4);
pub const CHEST_SIZE: Vec3 = Vec3::new(1.6, 0.8, 1.2);

pub static INTERACTIVE_OBJECTS: LazyLock<[ObjectDefinition; 12]> = LazyLock::new(|| {
    [
        ("board", "Station board", Interaction::Board),
        ("craft", "Station craft", Interaction::Craft),
        ("rack", "Station rack", Interaction::Rack),
        (
            "valve0",
            "Station valve 0",
            Interaction::Valve(ValveId::ALL[0]),
        ),
        (
            "valve1",
            "Station valve 1",
            Interaction::Valve(ValveId::ALL[1]),
        ),
        ("scan0", "Station scan 0", Interaction::Scan(ScanId::ALL[0])),
        ("scan1", "Station scan 1", Interaction::Scan(ScanId::ALL[1])),
        ("scan2", "Station scan 2", Interaction::Scan(ScanId::ALL[2])),
        ("generator", "Station generator", Interaction::Install),
        ("signal", "Station sender", Interaction::Signal),
        ("checkpoint", "Station checkpoint", Interaction::Checkpoint),
        ("chest", "Blueprint reward chest", Interaction::Chest),
    ]
    .map(|(key, name, action)| station(key).object(name, action))
});
pub static PICKUP_TABLES: LazyLock<[ObjectDefinition; 5]> = LazyLock::new(|| {
    [
        ("part0", "Table part 0", Interaction::Part(PartId::ALL[0])),
        ("part1", "Table part 1", Interaction::Part(PartId::ALL[1])),
        ("cell0", "Table cell 0", Interaction::Cell(CellId::ALL[0])),
        ("cell1", "Table cell 1", Interaction::Cell(CellId::ALL[1])),
        ("cell2", "Table cell 2", Interaction::Cell(CellId::ALL[2])),
    ]
    .map(|(key, name, action)| station(key).object(name, action))
});
pub const RANGE_TARGETS: [Vec3; 3] = [
    point(29., 51., 1.85),
    point(35., 51., 1.85),
    point(41., 51., 1.85),
];
pub(super) const RACK_COLLIDER: &str = "Movable storage rack";
