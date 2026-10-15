//! Object-sized interaction targets. Walking clearance is not an interaction ray.
use crate::{
    camera::ViewState,
    game::items::blueprint as map,
    player::Player,
    player::actions::expedition::{Expedition, Interaction},
    player::actions::navigation::Navigation,
};
use bevy::prelude::*;

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

pub fn targets() -> impl Iterator<Item = Target> {
    STATIONS.into_iter().chain(TABLES)
}

/// Distance to a box along a ray, including rays parallel to a face.
fn box_hit(origin: Vec3, direction: Vec3, center: Vec3, half: Vec3) -> Option<f32> {
    let mut entry: f32 = 0.;
    let mut exit = f32::INFINITY;
    for i in 0..3 {
        let relative = origin[i] - center[i];
        if direction[i].abs() < 0.00001 {
            if relative.abs() > half[i] {
                return None;
            }
        } else {
            let a = (-half[i] - relative) / direction[i];
            let b = (half[i] - relative) / direction[i];
            entry = entry.max(a.min(b));
            exit = exit.min(a.max(b));
        }
    }
    (exit >= entry).then_some(entry)
}

fn unobstructed(nav: &Navigation, eye: Vec3, point: Vec3, owner: &str) -> bool {
    let delta = point - eye;
    let distance = delta.length();
    let direction = delta.normalize_or_zero();
    !nav.obstacles.iter().filter(|o| o.name != owner).any(|o| {
        box_hit(eye, direction, o.center.into(), o.half.into())
            .is_some_and(|hit| hit < distance - 0.02)
    }) && !nav.surfaces.iter().any(|s| {
        box_hit(
            eye,
            direction,
            Vec3::new(s.center[0], (s.height + s.bottom) / 2., s.center[1]),
            Vec3::new(s.half[0], (s.height - s.bottom) / 2., s.half[1]),
        )
        .is_some_and(|hit| hit < distance - 0.02)
    })
}

pub fn select(
    mission: &Expedition,
    player: &Player,
    nav: &Navigation,
    view: &ViewState,
) -> Interaction {
    if view.first_person && !view.captured {
        return Interaction::None;
    }
    let eye = player.feet() + Vec3::Y * 1.5;
    let forward = Quat::from_euler(EulerRot::YXZ, player.yaw, view.pitch, 0.) * Vec3::NEG_Z;
    targets()
        .filter(|t| mission.available(t.action))
        .filter_map(|target| {
            if (player.height - target.floor).abs() > 0.4 {
                return None;
            }
            let nearest = player.position.clamp(
                target.center.xz() - target.half.xz(),
                target.center.xz() + target.half.xz(),
            );
            let reach = player.position.distance(nearest);
            if reach > 1.75 {
                return None;
            }
            let (score, point) = if view.first_person {
                let hit = box_hit(eye, forward, target.center, target.half)?;
                (hit, eye + forward * hit)
            } else {
                (reach, target.center)
            };
            unobstructed(nav, eye, point, target.name).then_some((score, target.action))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map_or(Interaction::None, |(_, action)| action)
}
