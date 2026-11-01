//! Target geometry and shot intersections.
use crate::level::mission::TargetId;
use bevy::prelude::*;
pub const ENTRY: Vec2 = Vec2::new(
    crate::level::definition::RANGE_ENTRY.x,
    crate::level::definition::RANGE_ENTRY.z,
);
pub use crate::level::definition::RANGE_TARGETS as TARGETS;
pub const MIN_TARGET_DISTANCE: f32 = 6.0;
pub(super) const TARGET_RADIUS: f32 = 0.8;
const TARGET_FRONT: f32 = 0.32;

/// A solid disk handles centre-line hits independently of glTF triangle seams.
/// The closest scene intersection still stops shots at walls and other props.
pub(super) fn target_hit(
    origin: Vec3,
    direction: Vec3,
    limit: f32,
    destroyed: &[bool; 3],
) -> Option<(TargetId, Vec3)> {
    if direction.z >= -0.00001 {
        return None;
    }
    TARGETS
        .iter()
        .enumerate()
        .filter_map(|(i, &center)| {
            let distance = (center.z + TARGET_FRONT - origin.z) / direction.z;
            let point = origin + direction * distance;
            (!destroyed[i]
                && distance > 0.
                && distance <= limit + 0.001
                && (point - center).xy().length_squared() <= TARGET_RADIUS * TARGET_RADIUS)
                .then_some((TargetId::ALL[i], point, distance))
        })
        .min_by(|a, b| a.2.total_cmp(&b.2))
        .map(|(i, p, _)| (i, p))
}
pub fn in_range(p: Vec2) -> bool {
    p.x > -15. && p.x < 11. && p.y > -27. && p.y < -11.
}

pub fn stand_boxes(center: Vec3) -> [(Vec3, Vec3); 3] {
    [
        (
            Vec3::new(1.3, 0.2, 1.0),
            Vec3::new(center.x, 0.1, center.z - 0.35),
        ),
        (
            Vec3::new(0.22, 1.45, 0.22),
            Vec3::new(center.x, 0.825, center.z - 0.35),
        ),
        (
            Vec3::new(0.8, 0.16, 0.8),
            Vec3::new(center.x, 0.85, center.z - 0.1),
        ),
    ]
}
