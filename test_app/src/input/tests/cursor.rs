use super::*;
use crate::{player::camera::GameCamera, ui::UiBlock};
use bevy::{
    camera::{CameraProjection, ComputedCameraValues, RenderTargetInfo},
    picking::pointer::Location,
    window::{WindowRef, WindowResolution},
};

fn scene(mode: InputMode) -> (App, Entity) {
    let (mut app, window) = app(mode, mode == InputMode::Native);
    app.init_resource::<bevy::picking::pointer::PointerMap>();
    app.world_mut()
        .resource_mut::<Player>()
        .respawn(Vec2::ZERO, 0.0);
    let mut win = app.world_mut().get_mut::<Window>(window).unwrap();
    win.resolution = WindowResolution::new(800, 800).with_scale_factor_override(2.0);
    win.set_cursor_position(Some(Vec2::new(100.0, 200.0)));
    app.world_mut().spawn((
        GameCamera,
        Camera {
            computed: ComputedCameraValues {
                clip_from_view: PerspectiveProjection {
                    fov: std::f32::consts::FRAC_PI_2,
                    aspect_ratio: 1.0,
                    near: 0.1,
                    ..default()
                }
                .get_clip_from_view(),
                target_info: Some(RenderTargetInfo {
                    physical_size: UVec2::splat(800),
                    scale_factor: 2.0,
                }),
                ..default()
            },
            ..default()
        },
        GlobalTransform::from(
            Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Z),
        ),
    ));
    (app, window)
}

fn location(window: Entity, position: Vec2) -> PointerLocation {
    PointerLocation::new(Location {
        target: NormalizedRenderTarget::Window(WindowRef::Entity(window).normalize(None).unwrap()),
        position,
    })
}

fn route_click(app: &mut App) {
    let mut buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    buttons.reset_all();
    buttons.press(MouseButton::Left);
    app.world_mut()
        .run_system_cached(crate::input::devices::click_to_move)
        .unwrap();
}

#[test]
fn virtual_route_and_ui_hit_test_share_logical_coordinates_without_moving_os_cursor() {
    let (mut app, window) = scene(CONTROLLED);
    let native = app.world().get::<Window>(window).unwrap().internal;
    app.world_mut()
        .spawn((VIRTUAL_ID, location(window, Vec2::new(300.0, 200.0))));
    // The real mouse is over a panel. The virtual pointer targets the floor to its right.
    let panel = app
        .world_mut()
        .spawn((
            UiBlock,
            ComputedNode {
                size: Vec2::splat(60.0),
                ..default()
            },
            UiGlobalTransform::from_translation(Vec2::new(200.0, 400.0)),
        ))
        .id();
    route_click(&mut app);
    let target = *app.world().resource::<Player>().route().back().unwrap();
    assert!(
        target.distance(Vec3::new(5.0, 0.0, 0.0)) < 0.001,
        "{target:?}"
    );
    assert_eq!(app.world().get::<Window>(window).unwrap().internal, native);
    assert!(!app.world().get::<Window>(window).unwrap().focused);

    // Cover the virtual cursor instead. A panel click must not create a route or bind FPV.
    *app.world_mut().get_mut::<UiGlobalTransform>(panel).unwrap() =
        UiGlobalTransform::from_translation(Vec2::new(600.0, 400.0));
    app.world_mut().resource_mut::<Player>().clear_route();
    let text = app.world_mut().spawn(EditableText::new("notes")).id();
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(text, FocusCause::Pressed);
    route_click(&mut app);
    assert!(app.world().resource::<Player>().route().is_empty());
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(text));
    app.world_mut()
        .resource_mut::<ViewState>()
        .set_mode(if true {
            crate::player::camera::ViewMode::FirstPerson
        } else {
            crate::player::camera::ViewMode::Overview
        });
    route_click(&mut app);
    assert!(
        !app.world()
            .resource::<crate::input::PointerCapture>()
            .active()
    );
    // No native position is required to click back into the game with the virtual pointer.
    app.world_mut().despawn(panel);
    app.world_mut()
        .get_mut::<Window>(window)
        .unwrap()
        .set_cursor_position(None);
    route_click(&mut app);
    assert!(
        app.world()
            .resource::<crate::input::PointerCapture>()
            .active()
    );
    assert!(app.world().resource::<InputFocus>().get().is_none());
}

#[test]
fn invalid_virtual_locations_never_fall_back_to_the_native_mouse() {
    let (mut app, window) = scene(CONTROLLED);
    let pointer = app
        .world_mut()
        .spawn((VIRTUAL_ID, PointerLocation::default()))
        .id();
    // An unrelated device must not be mistaken for Woodpecker's configured pointer.
    app.world_mut()
        .spawn((PointerId::Mouse, location(window, Vec2::new(300.0, 200.0))));
    let other_window = app.world_mut().spawn(Window::default()).id();
    for invalid in [
        PointerLocation::default(),
        location(other_window, Vec2::new(300.0, 200.0)),
        location(window, Vec2::new(400.0, 200.0)),
        location(window, Vec2::new(-1.0, 200.0)),
        location(window, Vec2::new(f32::NAN, 200.0)),
    ] {
        *app.world_mut().get_mut::<PointerLocation>(pointer).unwrap() = invalid;
        route_click(&mut app);
        assert!(app.world().resource::<Player>().route().is_empty());
    }
    app.world_mut().despawn(pointer);
    route_click(&mut app);
    assert!(app.world().resource::<Player>().route().is_empty());
    // Standalone play still uses the native position, even with custom pointers present.
    app.world_mut().insert_resource(InputMode::Native);
    app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
    route_click(&mut app);
    let target = *app.world().resource::<Player>().route().back().unwrap();
    assert!(
        target.distance(Vec3::new(-5.0, 0.0, 0.0)) < 0.001,
        "{target:?}"
    );
}

#[test]
fn the_same_pointer_id_has_independent_positions_in_separate_sessions() {
    let (mut first, window) = scene(CONTROLLED);
    let (mut second, other_window) = scene(CONTROLLED);
    first
        .world_mut()
        .spawn((VIRTUAL_ID, location(window, Vec2::new(300.0, 200.0))));
    second
        .world_mut()
        .spawn((VIRTUAL_ID, location(other_window, Vec2::new(200.0, 300.0))));
    route_click(&mut first);
    route_click(&mut second);
    let a = *first.world().resource::<Player>().route().back().unwrap();
    let b = *second.world().resource::<Player>().route().back().unwrap();
    assert!(a.distance(Vec3::new(5.0, 0.0, 0.0)) < 0.001, "{a:?}");
    assert!(b.distance(Vec3::new(0.0, 0.0, 5.0)) < 0.001, "{b:?}");
}
