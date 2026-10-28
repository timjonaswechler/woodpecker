use super::*;
use crate::command::input::keyboard::Key;
use bevy::{
    input::{
        ButtonState, InputPlugin,
        keyboard::Key as LogicalKey,
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit},
        touch::{TouchPhase, Touches},
    },
    window::{PrimaryWindow, WindowCloseRequested, WindowResized},
};

#[test]
fn native_input_cannot_enter_controlled_ticks_or_release_virtual_holds() {
    // Software-only input schedules: no Session, Winit, renderer or OS events.
    let mut app = App::new();
    app.add_plugins(InputPlugin);
    app.add_message::<WindowEvent>()
        .add_message::<CursorMoved>()
        .add_message::<CursorEntered>()
        .add_message::<CursorLeft>()
        .add_message::<Ime>()
        .add_message::<PointerInput>();
    keyboard::install(&mut app);
    let window = app
        .world_mut()
        .spawn((Window::default(), PrimaryWindow))
        .id();
    let key = KeyboardInput {
        key_code: KeyCode::KeyA,
        logical_key: LogicalKey::Character("a".into()),
        state: ButtonState::Pressed,
        text: Some("a".into()),
        repeat: false,
        window,
    };
    let native = vec![
        WindowEvent::KeyboardInput(key.clone()),
        WindowEvent::KeyboardFocusLost(KeyboardFocusLost),
        WindowEvent::MouseButtonInput(MouseButtonInput {
            button: MouseButton::Left,
            state: ButtonState::Pressed,
            window,
        }),
        WindowEvent::MouseMotion(MouseMotion {
            delta: Vec2::new(7., -3.),
        }),
        WindowEvent::MouseWheel(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 2.,
            y: -4.,
            window,
            phase: TouchPhase::Moved,
        }),
        WindowEvent::CursorMoved(CursorMoved {
            window,
            position: Vec2::new(19., 23.),
            delta: Some(Vec2::ONE),
        }),
        WindowEvent::CursorEntered(CursorEntered { window }),
        WindowEvent::CursorLeft(CursorLeft { window }),
        WindowEvent::Ime(Ime::Commit {
            window,
            value: "native IME".into(),
        }),
        WindowEvent::TouchInput(TouchInput {
            phase: TouchPhase::Started,
            position: Vec2::new(5., 6.),
            window,
            force: None,
            id: 42,
        }),
        WindowEvent::PinchGesture(PinchGesture(1.25)),
        WindowEvent::RotationGesture(RotationGesture(-0.5)),
        WindowEvent::DoubleTapGesture(DoubleTapGesture),
        WindowEvent::PanGesture(PanGesture(Vec2::ONE)),
    ];
    for event in native {
        // Winit normally writes both typed and aggregate messages; reproduce both.
        match &event {
            WindowEvent::KeyboardInput(v) => {
                app.world_mut().write_message(v.clone());
            }
            WindowEvent::KeyboardFocusLost(v) => {
                app.world_mut().write_message(v.clone());
            }
            WindowEvent::MouseButtonInput(v) => {
                app.world_mut().write_message(*v);
            }
            WindowEvent::MouseMotion(v) => {
                app.world_mut().write_message(*v);
            }
            WindowEvent::MouseWheel(v) => {
                app.world_mut().write_message(*v);
            }
            WindowEvent::CursorMoved(v) => {
                app.world_mut().write_message(v.clone());
            }
            WindowEvent::CursorEntered(v) => {
                app.world_mut().write_message(v.clone());
            }
            WindowEvent::CursorLeft(v) => {
                app.world_mut().write_message(v.clone());
            }
            WindowEvent::Ime(v) => {
                app.world_mut().write_message(v.clone());
            }
            WindowEvent::TouchInput(v) => {
                app.world_mut().write_message(*v);
            }
            WindowEvent::PinchGesture(v) => {
                app.world_mut().write_message(*v);
            }
            WindowEvent::RotationGesture(v) => {
                app.world_mut().write_message(*v);
            }
            WindowEvent::DoubleTapGesture(v) => {
                app.world_mut().write_message(*v);
            }
            WindowEvent::PanGesture(v) => {
                app.world_mut().write_message(*v);
            }
            _ => unreachable!(),
        }
        app.world_mut().write_message(event);
    }
    app.world_mut()
        .write_message(WindowEvent::WindowResized(WindowResized {
            window,
            width: 800.,
            height: 600.,
        }));
    app.world_mut()
        .write_message(WindowEvent::WindowCloseRequested(WindowCloseRequested {
            window,
        }));
    let location = bevy::picking::pointer::Location {
        target: bevy::camera::NormalizedRenderTarget::Window(
            bevy::window::WindowRef::Entity(window)
                .normalize(Some(window))
                .unwrap(),
        ),
        position: Vec2::new(12., 13.),
    };
    app.world_mut().write_message(PointerInput::new(
        bevy::picking::pointer::PointerId::Mouse,
        location,
        bevy::picking::pointer::PointerAction::Press(
            bevy::picking::pointer::PointerButton::Primary,
        ),
    ));
    let mut gate = Gate::default();
    let mut virtual_keys = keyboard::State::default();
    virtual_keys
        .key(app.world(), &Key::ShiftLeft, true)
        .unwrap();
    gate.capture(app.world_mut());
    macro_rules! empty {($($t:ty),*)=>{$(assert!(app.world().resource::<Messages<$t>>().is_empty(),stringify!($t));)*};}
    empty!(
        KeyboardInput,
        KeyboardFocusLost,
        MouseButtonInput,
        MouseMotion,
        MouseWheel,
        CursorMoved,
        CursorEntered,
        CursorLeft,
        Ime,
        TouchInput,
        PinchGesture,
        RotationGesture,
        DoubleTapGesture,
        PanGesture,
        PointerInput
    );
    gate.swap_window_events(app.world_mut());
    let retained: Vec<_> = app
        .world()
        .resource::<Messages<WindowEvent>>()
        .iter_current_update_messages()
        .collect();
    assert_eq!(retained.len(), 2);
    assert!(matches!(retained[0], WindowEvent::WindowResized(_)));
    assert!(matches!(retained[1], WindowEvent::WindowCloseRequested(_)));
    virtual_keys.flush(app.world_mut());
    app.update(); // controlled tick 1
    gate.swap_window_events(app.world_mut());
    let physical = app.world().resource::<ButtonInput<KeyCode>>();
    assert!(physical.pressed(KeyCode::ShiftLeft));
    assert!(!physical.pressed(KeyCode::KeyA));
    assert!(
        !app.world()
            .resource::<ButtonInput<MouseButton>>()
            .pressed(MouseButton::Left)
    );
    assert_eq!(
        app.world().resource::<AccumulatedMouseMotion>().delta,
        Vec2::ZERO
    );
    assert_eq!(
        app.world().resource::<AccumulatedMouseScroll>().delta,
        Vec2::ZERO
    );
    assert_eq!(app.world().resource::<Touches>().iter().count(), 0);
    // Focus loss plus an actual native release of the virtual key, during idle.
    let release = KeyboardInput {
        key_code: KeyCode::ShiftLeft,
        logical_key: LogicalKey::Shift,
        state: ButtonState::Released,
        text: None,
        repeat: false,
        window,
    };
    app.world_mut().write_message(release.clone());
    app.world_mut()
        .write_message(WindowEvent::KeyboardInput(release));
    app.world_mut().write_message(KeyboardFocusLost);
    app.world_mut()
        .write_message(WindowEvent::KeyboardFocusLost(KeyboardFocusLost));
    gate.capture(app.world_mut());
    gate.swap_window_events(app.world_mut());
    virtual_keys.flush(app.world_mut());
    app.update(); // controlled tick 2
    gate.swap_window_events(app.world_mut());
    assert!(
        app.world()
            .resource::<ButtonInput<KeyCode>>()
            .pressed(KeyCode::ShiftLeft)
    );
    assert!(
        !app.world()
            .resource::<ButtonInput<KeyCode>>()
            .just_released(KeyCode::ShiftLeft)
    );
    assert!(
        app.world()
            .resource::<ButtonInput<LogicalKey>>()
            .pressed(LogicalKey::Shift)
    );

    // Independent native baseline: no Woodpecker installation or Gate.
    let mut native = App::new();
    native.add_plugins(InputPlugin);
    native.world_mut().write_message(key);
    native.world_mut().write_message(MouseButtonInput {
        button: MouseButton::Left,
        state: ButtonState::Pressed,
        window,
    });
    native.world_mut().write_message(TouchInput {
        phase: TouchPhase::Started,
        position: Vec2::ONE,
        window,
        force: None,
        id: 42,
    });
    native.update(); // native tick 1: original Bevy keyboard system accepts A.
    assert!(
        native
            .world()
            .resource::<ButtonInput<KeyCode>>()
            .pressed(KeyCode::KeyA)
    );
    assert!(
        native
            .world()
            .resource::<ButtonInput<MouseButton>>()
            .pressed(MouseButton::Left)
    );
    assert_eq!(native.world().resource::<Touches>().iter().count(), 1);
    native.world_mut().write_message(KeyboardFocusLost);
    native.update(); // native tick 2: normal focus-loss behavior is retained.
    assert!(
        !native
            .world()
            .resource::<ButtonInput<KeyCode>>()
            .pressed(KeyCode::KeyA)
    );
}
