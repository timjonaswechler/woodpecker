//! Locomotion consumes one frame of semantic input.
use super::*;
pub fn move_player(
    time: Res<Time>,
    frame: Res<crate::input::intents::FrameInput>,
    nav: Res<Navigation>,
    settings: Res<Settings>,
    mut player: ResMut<Player>,
    view: Res<ViewState>,
) {
    let Some(intent) = frame.movement() else {
        return;
    };
    let dt = time.delta_secs().min(0.05);
    let axis = intent.axis;
    // Use the same ground-contact rule as jumping and body bob. Shift changes
    // neither walking to running nor running to walking while airborne.
    if player.is_grounded(&nav) {
        player.sprinting = intent.sprint;
    }
    let speed = settings.speed() * if player.sprinting { 1.65 } else { 1.0 };
    let before = player.position;
    let mut delta = Vec2::ZERO;
    if axis != Vec2::ZERO {
        player.route.clear();
        let mut direction = axis.normalize();
        if view.first_person() {
            direction = (Quat::from_rotation_y(player.yaw) * ground(direction, 0.0)).xz();
        }
        delta = direction * speed * dt;
    } else if let Some(&target) = player.route.front() {
        delta = (target.xz() - player.position).clamp_length_max(speed * dt);
    }
    player.advance(&nav, delta, dt, intent.jump);
    if player.route.front().is_some_and(|target| {
        target.xz().distance(player.position) < 0.04 && (target.y - player.height).abs() < 0.05
    }) {
        player.route.pop_front();
    }
    let travelled = player.position - before;
    player.walked += travelled.length();
    if !view.first_person() && travelled.length_squared() > 0.000001 {
        player.yaw = (-travelled.x).atan2(-travelled.y);
    }
}
