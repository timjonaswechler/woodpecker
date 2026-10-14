#[cfg(feature = "automation")]
mod automation;
mod blueprint;
mod camera;
mod coins;
mod course;
mod expedition;
mod game;
mod input;
mod interactions;
mod items;
mod level;
mod navigation;
mod options;
mod player;
#[cfg(test)]
mod playtests;
mod scenery;
mod settings;
mod shooting;
mod smoke;
mod stations;
mod ui;
mod zoo;

use bevy::{
    input_focus::tab_navigation::TabNavigationPlugin, prelude::*, window::WindowResolution,
};

fn main() {
    if std::env::args().any(|arg| arg == "--zoo") {
        zoo::run();
        return;
    }
    let smoke_test = std::env::args().any(|arg| arg == "--smoke-test");
    let offscreen = smoke_test && std::env::args().any(|arg| arg == "--offscreen");
    let fixed_window = std::env::args().any(|arg| arg == "--fixed-window");
    let scale = options::value::<f32>("--scale-factor");
    assert!(
        scale.is_none_or(|value| value.is_finite() && value > 0.0),
        "invalid scale factor"
    );
    let resolution = match scale.or(offscreen.then_some(1.0)) {
        Some(scale) => WindowResolution::new(1440, 900).with_scale_factor_override(scale),
        None => WindowResolution::new(1440, 900),
    };
    let mut app = App::new();
    #[cfg(feature = "automation")]
    let log_plugin = automation::log_plugin();
    #[cfg(not(feature = "automation"))]
    let log_plugin = bevy::log::LogPlugin::default();
    app.insert_resource(ClearColor(Color::srgb(0.63, 0.66, 0.63)))
        .insert_resource(if smoke_test {
            settings::Settings::default()
        } else {
            settings::Settings::load()
        })
        .init_resource::<player::Player>()
        .init_resource::<camera::ViewState>()
        .init_resource::<coins::CoinRun>()
        .init_resource::<expedition::Expedition>()
        .init_resource::<ui::UiState>()
        .init_resource::<level::LevelLoad>()
        .init_resource::<ui::Actions>()
        .init_resource::<input::InputMode>()
        .init_resource::<shooting::Practice>()
        .init_resource::<level::LevelAssets>()
        .insert_resource(level::layout())
        .add_plugins(
            DefaultPlugins
                .set(log_plugin)
                .set(AssetPlugin {
                    file_path: format!("{}/asset", env!("CARGO_MANIFEST_DIR")),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Signalstation | Räume und Entdeckungen".into(),
                        focused: !std::env::args().any(|arg| arg == "--start-unfocused"),
                        resizable: !fixed_window,
                        visible: !offscreen,
                        resolution,
                        resize_constraints: if fixed_window {
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
                course::spawn_course,
                stations::spawn_stations,
                shooting::spawn_range,
                ui::spawn_ui,
            ),
        )
        .add_observer(shooting::tag_scene_parts)
        .add_observer(player::pose_player)
        .add_observer(course::hide_planning_markers)
        .add_systems(
            Update,
            (
                (
                    level::check_assets,
                    ui::apply_font,
                    ui::buttons,
                    ui::sync_text_fields,
                    game::keyboard_shortcuts,
                    game::handle_actions,
                    course::update_course,
                    stations::update_stations,
                )
                    .chain(),
                (
                    camera::mouse_look,
                    player::move_player,
                    coins::tick_run,
                    expedition::tick_expedition,
                    shooting::tick_practice,
                    shooting::shoot,
                    player::click_to_move,
                    camera::sync_cursor,
                    camera::update_camera,
                    player::update_model,
                    coins::update_coins,
                    level::update_lighting,
                    shooting::update_visuals,
                    ui::update_ui,
                )
                    .chain(),
            )
                .chain(),
        );
    if smoke_test {
        smoke::prepare_artifacts();
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(0.05),
        ))
        .add_systems(Update, smoke::step.before(level::check_assets));
    }
    if offscreen {
        app.insert_resource(bevy::winit::WinitSettings::continuous());
        app.add_systems(
            Startup,
            smoke::setup_offscreen
                .after(level::spawn_level)
                .after(shooting::spawn_range)
                .after(ui::spawn_ui),
        );
    }
    #[cfg(feature = "automation")]
    automation::install(&mut app);
    app.run();
}
