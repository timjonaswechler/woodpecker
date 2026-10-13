//! Authoritative runtime data exported from the Blender planning model.
use crate::navigation::{Obstacle, Surface};
use bevy::prelude::*;
use serde::Deserialize;
use std::sync::LazyLock;
#[derive(Deserialize)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub bounds: [f32; 4],
}
#[derive(Deserialize)]
pub struct Blueprint {
    pub rooms: Vec<Room>,
    pub surfaces: Vec<Surface>,
    pub obstacles: Vec<Obstacle>,
    pub coins: [[f32; 3]; 24],
    pub blueprints: [[f32; 3]; 6],
    pub gears: [[f32; 3]; 2],
}
pub static DATA: LazyLock<Blueprint> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../asset/custom/courtyard/layout.json"))
        .expect("valid exported courtyard layout")
});
pub const fn point(x: f32, y: f32, height: f32) -> Vec3 {
    Vec3::new(x - 39., height, 27. - y)
}
pub fn room(position: Vec2) -> Option<&'static Room> {
    let x = position.x + 39.;
    let y = 27. - position.y;
    DATA.rooms
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
pub const REWARDS: [Vec3; 3] = [BENCH, point(71., 40.5, 0.), point(17., 46.5, 0.)];
pub const GENERATOR: Vec3 = point(61., 5.5, 0.);
pub const SIGNAL: Vec3 = point(73., 11.8, 3.);
pub const PRACTICE: Vec3 = point(35., 41., 0.);
pub const BOARD: Vec3 = point(4., 11.5, 0.);
