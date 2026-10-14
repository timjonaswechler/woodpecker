use super::*;
mod cursor;
use crate::{
    camera::{self, ViewState},
    coins::{self, CoinRun},
    expedition::{self, Expedition},
    game,
    level::LevelLoad,
    navigation::Navigation,
    player::{self, Player},
    settings::Settings,
    shooting::{self, Fragments, Practice},
    ui::{Actions, UiState},
};
use bevy::{
    app::AppExit,
    input::{
        ButtonState, InputPlugin,
        keyboard::{Key, KeyboardInput},
        mouse::{MouseButtonInput, MouseMotion},
    },
    input_focus::{FocusCause, InputFocus},
    text::EditableText,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use std::time::Duration;

const VIRTUAL_ID: PointerId = PointerId::Custom(bevy::asset::uuid::Uuid::from_u128(42));
const CONTROLLED: InputMode = InputMode::Controlled(VIRTUAL_ID);

fn app(mode: InputMode, focused: bool) -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins(InputPlugin)
        .add_message::<AppExit>()
        .insert_resource(mode)
        .init_resource::<Time>()
        .init_resource::<InputFocus>()
        .init_resource::<Navigation>()
        .init_resource::<Settings>()
        .init_resource::<Player>()
        .init_resource::<ViewState>()
        .init_resource::<UiState>()
        .init_resource::<Actions>()
        .init_resource::<CoinRun>()
        .init_resource::<Expedition>()
        .init_resource::<Practice>()
        .init_resource::<Fragments>()
        .init_resource::<Assets<Mesh>>()
        .insert_resource(LevelLoad::Ready)
        .add_systems(
            Update,
            (
                game::keyboard_shortcuts,
                game::handle_actions,
                camera::mouse_look,
                player::move_player,
                coins::tick_run,
                expedition::tick_expedition,
                shooting::tick_practice,
                shooting::shoot,
                camera::sync_cursor,
            )
                .chain(),
        );
    let window = app
        .world_mut()
        .spawn((
            Window {
                focused,
                ..default()
            },
            PrimaryWindow,
            CursorOptions::default(),
        ))
        .id();
    (app, window)
}

fn key(app: &mut App, window: Entity, key_code: KeyCode) {
    app.world_mut().write_message(KeyboardInput {
        key_code,
        logical_key: Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified),
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window,
    });
}

fn tick(app: &mut App) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f64(1.0 / 60.0));
    app.world_mut().run_schedule(PreUpdate);
    app.world_mut().run_schedule(Update);
    // Match the message lifecycle without creating a window or renderer.
    app.world_mut()
        .resource_mut::<Messages<KeyboardInput>>()
        .update();
    app.world_mut()
        .resource_mut::<Messages<MouseMotion>>()
        .update();
    app.world_mut()
        .resource_mut::<Messages<MouseButtonInput>>()
        .update();
}

fn click(app: &mut App, window: Entity) {
    app.world_mut().write_message(MouseButtonInput {
        window,
        button: MouseButton::Left,
        state: ButtonState::Pressed,
    });
}

#[test]
fn controlled_background_games_move_look_and_shoot_independently() {
    let (mut first, window) = app(CONTROLLED, false);
    let (mut second, other_window) = app(CONTROLLED, false);
    let origin = first.world().resource::<Player>().position;
    key(&mut first, window, KeyCode::KeyV);
    key(&mut first, window, KeyCode::KeyW);
    first.world_mut().write_message(MouseMotion {
        delta: Vec2::new(100.0, -40.0),
    });
    tick(&mut first);
    tick(&mut second);
    assert!(first.world().resource::<ViewState>().captured);
    assert!(first.world().resource::<ViewState>().pitch > 0.0);
    assert!(first.world().resource::<Player>().yaw < 0.0);
    assert!(first.world().resource::<Player>().position.distance(origin) > 0.05);
    assert_eq!(second.world().resource::<Player>().position, origin);
    assert_eq!(second.world().resource::<Player>().yaw, 0.0);
    assert!(first.world().resource::<Expedition>().elapsed > 0.0);
    click(&mut first, window);
    tick(&mut first);
    assert_eq!(first.world().resource::<Practice>().shots, 1);
    assert_eq!(second.world().resource::<Practice>().shots, 0);
    let first_position = first.world().resource::<Player>().position;
    key(&mut second, other_window, KeyCode::ArrowRight);
    tick(&mut second);
    assert!(second.world().resource::<Player>().position.x > origin.x);
    assert_eq!(first.world().resource::<Player>().position, first_position);
    for (app, window) in [(&first, window), (&second, other_window)] {
        assert!(!app.world().get::<Window>(window).unwrap().focused);
        let cursor = app.world().get::<CursorOptions>(window).unwrap();
        assert!(cursor.visible);
        assert_eq!(cursor.grab_mode, CursorGrabMode::None);
    }
}

#[test]
fn controlled_input_still_respects_menu_text_focus_and_loading() {
    for blocked_by in ["menu", "text", "loading"] {
        let (mut app, window) = app(CONTROLLED, false);
        app.world_mut().resource_mut::<ViewState>().toggle();
        tick(&mut app); // Arm shooting only after entering FPV, as in the game.
        match blocked_by {
            "menu" => app.world_mut().resource_mut::<UiState>().menu = true,
            "text" => {
                let text = app.world_mut().spawn(EditableText::new("notes")).id();
                app.world_mut()
                    .resource_mut::<InputFocus>()
                    .set(text, FocusCause::Pressed);
            }
            "loading" => *app.world_mut().resource_mut::<LevelLoad>() = LevelLoad::Loading,
            _ => unreachable!(),
        }
        let origin = app.world().resource::<Player>().position;
        let yaw = app.world().resource::<Player>().yaw;
        let elapsed = app.world().resource::<Expedition>().elapsed;
        key(&mut app, window, KeyCode::KeyW);
        app.world_mut().write_message(MouseMotion {
            delta: Vec2::splat(100.0),
        });
        click(&mut app, window);
        tick(&mut app);
        assert_eq!(
            app.world().resource::<Player>().position,
            origin,
            "{blocked_by}"
        );
        assert_eq!(app.world().resource::<Player>().yaw, yaw, "{blocked_by}");
        assert_eq!(app.world().resource::<Practice>().shots, 0, "{blocked_by}");
        assert_eq!(
            app.world().resource::<Expedition>().elapsed,
            elapsed,
            "{blocked_by}"
        );
    }
}

#[test]
fn native_play_keeps_focus_gating_and_controlled_play_never_grabs_the_os_cursor() {
    for mode in [InputMode::Native, CONTROLLED] {
        let (mut app, window) = app(mode, true);
        key(&mut app, window, KeyCode::KeyV);
        key(&mut app, window, KeyCode::KeyW);
        tick(&mut app);
        let cursor = app.world().get::<CursorOptions>(window).unwrap();
        assert_eq!(
            cursor.grab_mode,
            if mode == InputMode::Native {
                CursorGrabMode::Locked
            } else {
                CursorGrabMode::None
            }
        );
        let before = app.world().resource::<Player>().position;
        app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
        tick(&mut app);
        assert_eq!(
            app.world().resource::<ViewState>().captured,
            mode == CONTROLLED
        );
        assert_eq!(
            app.world().resource::<Player>().position == before,
            mode == InputMode::Native
        );
    }
}
