//! Object placement, interaction extents and physical colliders.
use crate::{level::mission::Interaction, level::navigation::Obstacle};
use bevy::prelude::*;

#[derive(Clone, Copy)]
pub struct ObjectDefinition {
    pub name: &'static str,
    pub action: Interaction,
    pub center: Vec3,
    pub half: Vec3,
    pub floor: f32,
}
impl ObjectDefinition {
    pub(crate) const fn new(
        name: &'static str,
        action: Interaction,
        base: Vec3,
        size: Vec3,
    ) -> Self {
        Self {
            name,
            action,
            center: Vec3::new(base.x, base.y + size.y / 2., base.z),
            half: Vec3::new(size.x / 2., size.y / 2., size.z / 2.),
            floor: base.y,
        }
    }
}

pub use crate::level::definition::{CHEST_CENTER, CHEST_SIZE, INTERACTIVE_OBJECTS, PICKUP_TABLES};

pub fn all() -> impl Iterator<Item = ObjectDefinition> {
    INTERACTIVE_OBJECTS.into_iter().chain(PICKUP_TABLES)
}

pub fn colliders() -> impl Iterator<Item = Obstacle> {
    INTERACTIVE_OBJECTS
        .into_iter()
        .chain(PICKUP_TABLES)
        .map(|target| {
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
            Obstacle::new(target.name.into(), center.to_array(), half.to_array())
                .expect("valid obstacle")
        })
}
