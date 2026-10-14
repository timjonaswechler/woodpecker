use super::*;
use bevy::window::PrimaryWindow;

fn world() -> (World, Entity) {
    let mut world = World::new();
    let window = world
        .spawn((
            Window {
                resolution: bevy::window::WindowResolution::new(200, 100)
                    .with_scale_factor_override(2.0),
                ..default()
            },
            PrimaryWindow,
        ))
        .id();
    (world, window)
}

#[test]
fn coordinates_are_logical_finite_and_rejection_preserves_accepted_state() {
    let (world, window) = world();
    let mut state = State::default();
    for position in [[f32::NAN, 0.], [0., f32::INFINITY]] {
        assert_eq!(
            state.move_to(&world, position).unwrap_err().code,
            "invalid_arguments"
        );
    }
    state.move_to(&world, [10., 20.]).unwrap();
    state.move_by(&world, [2., -3.]).unwrap();
    for position in [[-1., 0.], [0., -1.], [100., 0.], [0., 50.]] {
        assert_eq!(
            state.move_to(&world, position).unwrap_err().code,
            "pointer_position_out_of_bounds"
        );
        assert_eq!(
            state.cursor.location(&world).unwrap(),
            (window, Vec2::new(12., 17.))
        );
        assert_eq!(state.pending.len(), 2);
    }
    for delta in [[f32::NAN, 0.], [0., f32::INFINITY], [f32::NEG_INFINITY, 0.]] {
        assert_eq!(
            state.move_by(&world, delta).unwrap_err().code,
            "invalid_arguments"
        );
        assert_eq!(
            state.cursor.location(&world).unwrap(),
            (window, Vec2::new(12., 17.))
        );
        assert_eq!(state.pending.len(), 2);
    }
    assert!(
        world
            .get::<Window>(window)
            .unwrap()
            .cursor_position()
            .is_none()
    );
}

#[test]
fn relative_motion_starts_at_center_and_clamps_after_resize_without_overflow() {
    let (mut world, window) = world();
    let mut state = State::default();
    state.move_by(&world, [5., -3.]).unwrap();
    assert_eq!(
        state.cursor.location(&world).unwrap(),
        (window, Vec2::new(55., 22.))
    );
    state.move_by(&world, [f32::MAX, -f32::MAX]).unwrap();
    assert_eq!(
        state.cursor.location(&world).unwrap(),
        (window, Vec2::new(100_f32.next_down(), 0.))
    );
    state.move_by(&world, [f32::MAX, -f32::MAX]).unwrap();
    assert_eq!(
        state.cursor.location(&world).unwrap(),
        (window, Vec2::new(100_f32.next_down(), 0.))
    );
    world
        .get_mut::<Window>(window)
        .unwrap()
        .resolution
        .set(10., 10.);
    state.move_by(&world, [0., 2.]).unwrap();
    assert_eq!(
        state.cursor.location(&world).unwrap(),
        (window, Vec2::new(10_f32.next_down(), 2.))
    );
    state.button(&world, "left", true).unwrap();
}

#[test]
fn buttons_scroll_windows_and_isolation() {
    let (mut world, window) = world();
    let mut state = State::default();
    assert_eq!(
        state.button(&world, "other", true).unwrap_err().code,
        "invalid_pointer_button"
    );
    assert_eq!(
        state.button(&world, "left", true).unwrap_err().code,
        "pointer_location_unavailable"
    );
    assert_eq!(
        state.scroll(&world, [0., 0.]).unwrap_err().code,
        "pointer_location_unavailable"
    );
    state.move_to(&world, [12., 17.]).unwrap();
    for button in ["left", "right", "middle"] {
        assert_eq!(
            state.button(&world, button, false).unwrap_err().code,
            "pointer_button_not_pressed"
        );
        state.button(&world, button, true).unwrap();
        assert_eq!(
            state.button(&world, button, true).unwrap_err().code,
            "pointer_button_already_pressed"
        );
        state.button(&world, button, false).unwrap();
    }
    state.scroll(&world, [-2., 3.]).unwrap();
    state.scroll(&world, [0., 0.]).unwrap();
    let len = state.pending.len();
    assert_eq!(
        state.scroll(&world, [f32::INFINITY, 0.]).unwrap_err().code,
        "invalid_arguments"
    );
    assert_eq!(state.pending.len(), len);
    let PointerAction::Scroll { unit, x, y, .. } = &state.pending[len - 2].pointer.action else {
        panic!("scroll")
    };
    assert_eq!(*unit, MouseScrollUnit::Line);
    assert_eq!([*x, *y], [-2., 3.]);
    assert_eq!(
        State::default()
            .button(&world, "left", true)
            .unwrap_err()
            .code,
        "pointer_location_unavailable"
    );
    let second = world.spawn((Window::default(), PrimaryWindow)).id();
    assert_eq!(
        state.move_to(&world, [0., 0.]).unwrap_err().code,
        "pointer_window_unavailable"
    );
    assert_eq!(
        state.scroll(&world, [0., 0.]).unwrap_err().code,
        "pointer_window_unavailable"
    );
    assert_eq!(
        state.button(&world, "left", true).unwrap_err().code,
        "pointer_location_unavailable"
    );
    world.despawn(second);
    world
        .get_mut::<Window>(window)
        .unwrap()
        .resolution
        .set(1., 1.);
    assert_eq!(
        state.button(&world, "left", true).unwrap_err().code,
        "pointer_location_unavailable"
    );
    world.despawn(window);
    assert_eq!(
        state.move_to(&world, [0., 0.]).unwrap_err().code,
        "pointer_window_unavailable"
    );
    let replacement = world.spawn((Window::default(), PrimaryWindow)).id();
    state.move_by(&world, [0., 0.]).unwrap();
    let size = world.get::<Window>(replacement).unwrap();
    assert_eq!(
        state.cursor.location(&world).unwrap(),
        (replacement, Vec2::new(size.width(), size.height()) * 0.5)
    );
}
