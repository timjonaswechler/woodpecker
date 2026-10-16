//! Builds the playable application and owns the order of its systems.
use crate::{
    commands,
    expedition::{self, props, stations},
    input,
    items::coins,
    launch::Options,
    level,
    player::{self, camera},
    practice,
    settings::Settings,
    ui,
};
use bevy::{
    input_focus::tab_navigation::TabNavigationPlugin, prelude::*, window::WindowResolution,
};

pub fn install(app: &mut App, options: &Options, log_plugin: bevy::log::LogPlugin) {
    let resolution = match options.scale_factor.or(options.offscreen.then_some(1.0)) {
        Some(scale) => WindowResolution::new(1440, 900).with_scale_factor_override(scale),
        None => WindowResolution::new(1440, 900),
    };
    app.insert_resource(ClearColor(Color::srgb(0.63, 0.66, 0.63)))
        .insert_resource(if options.smoke_test {
            Settings::default()
        } else {
            Settings::load()
        })
        .init_resource::<player::Player>()
        .init_resource::<camera::ViewState>()
        .init_resource::<coins::CoinRun>()
        .init_resource::<expedition::Expedition>()
        .init_resource::<ui::UiState>()
        .init_resource::<level::LevelLoad>()
        .init_resource::<commands::Actions>()
        .init_resource::<input::InputMode>()
        .init_resource::<practice::Practice>()
        .init_resource::<level::LevelAssets>()
        .insert_resource(level::layout())
        .add_plugins(
            DefaultPlugins
                .set(log_plugin)
                .set(AssetPlugin {
                    file_path: crate::ASSET_DIR.into(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Signalstation | Räume und Entdeckungen".into(),
                        focused: !options.start_unfocused,
                        resizable: !options.fixed_window,
                        visible: !options.offscreen,
                        resolution,
                        resize_constraints: if options.fixed_window {
                            bevy::window::WindowResizeConstraints {
                                min_width: 1440.0,
                                max_width: 1440.0,
                                min_height: 900.0,
                                max_height: 900.0,
                            }
                        } else {
                            bevy::window::WindowResizeConstraints {
                                min_width: 960.0,
                                min_height: 640.0,
                                ..default()
                            }
                        },
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(TabNavigationPlugin)
        .add_systems(
            Startup,
            (
                level::spawn_level,
                player::spawn_player,
                coins::spawn_coins,
                camera::spawn_camera,
                props::spawn,
                stations::spawn_stations,
                practice::spawn_range,
                ui::spawn_ui,
            ),
        )
        .add_observer(practice::tag_scene_parts)
        .add_observer(player::pose_player)
        .add_observer(level::hide_planning_markers)
        .add_systems(
            Update,
            (
                (
                    level::check_assets,
                    ui::apply_font,
                    ui::buttons,
                    ui::sync_text_fields,
                    input::keyboard_shortcuts,
                    commands::handle_actions,
                    props::update,
                    stations::update_stations,
                )
                    .chain(),
                (
                    camera::mouse_look,
                    player::move_player,
                    coins::tick_run,
                    expedition::tick_expedition,
                    practice::tick_practice,
                    player::update_model,
                    camera::update_camera,
                    practice::shoot,
                    player::click_to_move,
                    camera::sync_cursor,
                    coins::update_coins,
                    level::update_lighting,
                    practice::update_visuals,
                    ui::update_ui,
                )
                    .chain(),
            )
                .chain(),
        );
}
