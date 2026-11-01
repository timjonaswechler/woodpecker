//! Object-sized interaction targets. Walking clearance is not an interaction ray.
use crate::{
    level::mission::{Interaction, Mission},
    level::navigation::Navigation,
    level::objects::definitions,
    player::Player,
    player::camera::ViewState,
};
use bevy::prelude::*;

use crate::level::navigation::box_hit;

fn unobstructed(nav: &Navigation, eye: Vec3, point: Vec3, owner: &str) -> bool {
    let delta = point - eye;
    let distance = delta.length();
    let direction = delta.normalize_or_zero();
    !nav.obstacles()
        .iter()
        .filter(|o| o.name() != owner)
        .any(|o| {
            box_hit(eye, direction, o.center().into(), o.half().into())
                .is_some_and(|hit| hit < distance - 0.02)
        })
        && !nav.surfaces().iter().any(|s| {
            box_hit(
                eye,
                direction,
                Vec3::new(s.center()[0], (s.height() + s.bottom()) / 2., s.center()[1]),
                Vec3::new(s.half()[0], (s.height() - s.bottom()) / 2., s.half()[1]),
            )
            .is_some_and(|hit| hit < distance - 0.02)
        })
}

pub fn select(
    mission: &Mission,
    player: &Player,
    nav: &Navigation,
    view: &ViewState,
) -> Interaction {
    let (eye, forward) = if view.first_person() {
        let Some(eye) = view.eye() else {
            return Interaction::None;
        };
        (eye.translation, eye.forward().as_vec3())
    } else {
        (player.feet() + Vec3::Y * 1.5, Vec3::NEG_Z)
    };
    definitions::all()
        .chain(super::passages::targets())
        .filter(|t| mission.available(t.action))
        .filter(|t| match t.action {
            Interaction::Portal(id) => super::passages::accessible(id, player.feet()),
            _ => true,
        })
        .filter_map(|target| {
            if (player.height() - target.floor).abs() > 0.4 {
                return None;
            }
            let nearest = player.position().clamp(
                target.center.xz() - target.half.xz(),
                target.center.xz() + target.half.xz(),
            );
            let reach = player.position().distance(nearest);
            if reach > 1.75 {
                return None;
            }
            let (score, point) = if view.first_person() {
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
