//! Station placement, interaction extents and physical colliders.
use crate::{
    expedition::Interaction,
    level::{layout as map, navigation::Obstacle},
};
use bevy::prelude::*;

pub const CHEST_CENTER: Vec3 = map::point(46., 51., 0.4);
pub const CHEST_SIZE: Vec3 = Vec3::new(1.6, 0.8, 1.2);

#[derive(Clone, Copy)]
pub struct Target {
    pub name: &'static str,
    pub action: Interaction,
    pub center: Vec3,
    pub half: Vec3,
    pub floor: f32,
}
impl Target {
    const fn new(name: &'static str, action: Interaction, base: Vec3, size: Vec3) -> Self {
        Self {
            name,
            action,
            center: Vec3::new(base.x, base.y + size.y / 2., base.z),
            half: Vec3::new(size.x / 2., size.y / 2., size.z / 2.),
            floor: base.y,
        }
    }
}

pub const STATIONS: [Target; 12] = [
    Target::new(
        "Station board",
        Interaction::Board,
        map::point(4., 13., 0.),
        Vec3::new(1.4, 1.9, 0.6),
    ),
    Target::new(
        "Station craft",
        Interaction::Craft,
        map::point(6.6, 33., 0.),
        Vec3::new(1.3, 1.5, 0.9),
    ),
    Target::new(
        "Station rack",
        Interaction::Rack,
        map::point(59., 27., 0.),
        Vec3::new(0.9, 1.5, 0.8),
    ),
    Target::new(
        "Station valve 0",
        Interaction::Valve(0),
        map::point(64., 43., 0.),
        Vec3::new(1.0, 1.5, 0.55),
    ),
    Target::new(
        "Station valve 1",
        Interaction::Valve(1),
        map::point(73., 49., 0.),
        Vec3::new(1.0, 1.5, 0.55),
    ),
    Target::new(
        "Station scan 0",
        Interaction::Scan(0),
        map::point(4., 50., 0.),
        Vec3::new(1.3, 1.75, 0.9),
    ),
    Target::new(
        "Station scan 1",
        Interaction::Scan(1),
        map::point(16., 51., 0.),
        Vec3::new(1.3, 1.75, 0.9),
    ),
    Target::new(
        "Station scan 2",
        Interaction::Scan(2),
        map::point(20., 40., 0.),
        Vec3::new(1.3, 1.75, 0.9),
    ),
    Target::new(
        "Station generator",
        Interaction::Install,
        map::point(61., 7., 0.),
        Vec3::new(1.3, 1.5, 0.9),
    ),
    Target::new(
        "Station sender",
        Interaction::Signal,
        map::point(73., 13., 3.),
        Vec3::new(0.9, 1.25, 0.65),
    ),
    Target::new(
        "Station checkpoint",
        Interaction::Checkpoint,
        map::point(24.4, 22., 0.),
        Vec3::new(0.5, 1.4, 0.5),
    ),
    Target::new(
        "Blueprint reward chest",
        Interaction::Chest,
        map::point(46., 51., 0.),
        Vec3::new(1.6, 1.2, 1.2),
    ),
];
// Every table has one purpose: take the visible object. Assembly has its own switch.
pub const TABLES: [Target; 5] = [
    Target::new(
        "Table part 0",
        Interaction::Part(0),
        map::point(5., 28., 0.),
        Vec3::new(1.1, 1.6, 1.1),
    ),
    Target::new(
        "Table part 1",
        Interaction::Part(1),
        map::point(66., 26., 0.),
        Vec3::new(1.1, 1.6, 1.1),
    ),
    Target::new(
        "Table cell 0",
        Interaction::Cell(0),
        map::point(4., 33., 0.),
        Vec3::new(4., 1.75, 2.),
    ),
    Target::new(
        "Table cell 1",
        Interaction::Cell(1),
        map::point(71., 42., 0.),
        Vec3::new(1.8, 1.75, 1.3),
    ),
    Target::new(
        "Table cell 2",
        Interaction::Cell(2),
        map::point(17., 48., 0.),
        Vec3::new(1.6, 1.75, 1.1),
    ),
];

pub fn all() -> impl Iterator<Item = Target> {
    STATIONS.into_iter().chain(TABLES)
}

pub fn colliders() -> impl Iterator<Item = Obstacle> {
    STATIONS.into_iter().chain(TABLES).map(|target| {
        let (center, half) = if target.action == Interaction::Chest {
            (CHEST_CENTER, CHEST_SIZE * 0.5)
        } else if matches!(target.action, Interaction::Part(_) | Interaction::Cell(_)) {
            let height = if matches!(target.action, Interaction::Part(_)) {
                0.84
            } else {
                1.
            };
            (
                Vec3::new(target.center.x, target.floor + height / 2., target.center.z),
                Vec3::new(target.half.x, height / 2., target.half.z),
            )
        } else {
            (target.center, target.half)
        };
        Obstacle {
            name: target.name.into(),
            center: center.to_array(),
            half: half.to_array(),
        }
    })
}
