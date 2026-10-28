//! Visual effects derived from range state.
use super::*;
use crate::app::flow::GameFlow;
pub fn update_visuals(
    mut commands: Commands,
    time: Res<Time>,
    shooting_range: Res<ShootingRange>,
    flow: Res<GameFlow>,
    mut targets: Query<(&Target, &mut Visibility)>,
    mut debris: Query<(Entity, &mut Transform, &mut Debris)>,
) {
    for (target, mut visibility) in &mut targets {
        *visibility = if shooting_range.targets_hit[target.0.index()] {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
    let dt = time.delta_secs().min(0.05);
    for (entity, mut transform, mut piece) in &mut debris {
        if flow.menu() {
            continue;
        }
        piece.life -= dt;
        piece.velocity.y -= 6.0 * dt;
        transform.translation += piece.velocity * dt;
        transform.rotate_local_x(dt * 2.8);
        if piece.life <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
