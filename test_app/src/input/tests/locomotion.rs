use super::*;

fn assert_stride(app: &mut App, sprinting: bool) {
    let before = app.world().resource::<Player>().position;
    tick(app);
    let player = app.world().resource::<Player>();
    assert_eq!(
        player.sprinting, sprinting,
        "locomotion state must match the actual speed"
    );
    let distance = player.position.distance(before);
    let expected = app.world().resource::<Settings>().speed
        * app.world().resource::<Time>().delta_secs()
        * if sprinting { 1.65 } else { 1. };
    assert!(
        (distance - expected).abs() < 0.00001,
        "sprinting={sprinting}, height={}, vertical_speed={}, moved={distance}, expected={expected}",
        player.height,
        player.vertical_speed,
    );
}

fn walking_app(
    mode: InputMode,
    first_person: bool,
    sprinting: bool,
    shift: KeyCode,
) -> (App, Entity) {
    let (mut app, window) = app(mode, mode == InputMode::Native);
    app.world_mut()
        .resource_mut::<Player>()
        .respawn(Vec2::ZERO, 0.);
    if first_person {
        key(&mut app, window, KeyCode::KeyV);
    }
    key(&mut app, window, KeyCode::ArrowRight);
    if sprinting {
        key(&mut app, window, shift);
    }
    assert_stride(&mut app, sprinting);
    (app, window)
}

fn change_shift(app: &mut App, window: Entity, shift: KeyCode, sprinting_before: bool) {
    key_state(
        app,
        window,
        shift,
        if sprinting_before {
            ButtonState::Released
        } else {
            ButtonState::Pressed
        },
    );
}

fn check_jump_lock(sprinting_at_takeoff: bool) {
    for mode in [InputMode::Native, CONTROLLED] {
        for first_person in [false, true] {
            for shift in [KeyCode::ShiftLeft, KeyCode::ShiftRight] {
                let (mut app, window) =
                    walking_app(mode, first_person, sprinting_at_takeoff, shift);
                key(&mut app, window, KeyCode::Space);
                assert_stride(&mut app, sprinting_at_takeoff);
                assert!(app.world().resource::<Player>().vertical_speed > 0.);
                key_state(&mut app, window, KeyCode::Space, ButtonState::Released);
                change_shift(&mut app, window, shift, sprinting_at_takeoff);
                let mut fell = false;
                let mut landed = false;
                for _ in 0..120 {
                    assert_stride(&mut app, sprinting_at_takeoff);
                    let player = app.world().resource::<Player>();
                    fell |= player.vertical_speed < 0.;
                    if player.height.abs() < 0.025 && player.vertical_speed <= 0. {
                        landed = true;
                        break;
                    }
                }
                assert!(
                    fell && landed,
                    "test must include ascent, descent and landing"
                );
                // Only now may the current key state change walking/running.
                assert_stride(&mut app, !sprinting_at_takeoff);
            }
        }
    }
}

#[test]
fn sprint_cannot_start_during_a_jump_and_is_applied_after_landing() {
    check_jump_lock(false);
}

#[test]
fn sprint_cannot_stop_during_a_jump_and_is_released_after_landing() {
    check_jump_lock(true);
}

#[test]
fn airborne_sprint_state_is_locked_even_at_the_apex_or_when_falling_without_a_jump() {
    for sprinting in [false, true] {
        for (height, vertical_speed) in [(1., 3.), (1., 0.), (1., -3.), (0., 3.)] {
            let (mut app, window) = walking_app(CONTROLLED, true, sprinting, KeyCode::ShiftLeft);
            {
                let mut player = app.world_mut().resource_mut::<Player>();
                player.height = height;
                player.vertical_speed = vertical_speed;
            }
            change_shift(&mut app, window, KeyCode::ShiftLeft, sprinting);
            assert_stride(&mut app, sprinting);
        }
    }
}

#[test]
fn respawn_resets_sprint_state_before_an_airborne_start() {
    assert!(!Player::default().sprinting);
    let (mut app, _) = walking_app(CONTROLLED, true, true, KeyCode::ShiftLeft);
    app.world_mut()
        .resource_mut::<Player>()
        .respawn(Vec2::ZERO, 1.);
    assert!(!app.world().resource::<Player>().sprinting);
    // Shift is still held, but spawning in the air must not enable running.
    assert_stride(&mut app, false);
}

#[test]
fn sprint_can_change_on_a_supported_upper_floor() {
    let (mut app, window) = walking_app(CONTROLLED, false, false, KeyCode::ShiftRight);
    let center = app.world().resource::<Player>().position.to_array();
    app.insert_resource(Navigation::new(
        vec![crate::level::navigation::Surface {
            center,
            half: [10., 10.],
            height: 3.,
            bottom: 2.7,
        }],
        Vec::new(),
        Vec2::splat(-39.),
        Vec2::splat(39.),
    ));
    app.world_mut().resource_mut::<Player>().height = 3.;
    key(&mut app, window, KeyCode::ShiftRight);
    assert_stride(&mut app, true);
    assert_eq!(app.world().resource::<Player>().height, 3.);
    key_state(&mut app, window, KeyCode::ShiftRight, ButtonState::Released);
    assert_stride(&mut app, false);
}
