mod game;
mod level;
mod navigation;
mod settings;
mod shooting;
mod smoke;
mod ui;

use bevy::{
    input_focus::tab_navigation::TabNavigationPlugin, prelude::*, window::WindowResolution,
};

fn main() {
    let smoke_test = std::env::args().any(|arg| arg == "--smoke-test");
    let offscreen = smoke_test && std::env::args().any(|arg| arg == "--offscreen");
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.055, 0.09, 0.12)))
        .insert_resource(if smoke_test {
            settings::Settings::default()
        } else {
            settings::Settings::load()
        })
        .init_resource::<game::Game>()
        .init_resource::<ui::Actions>()
        .init_resource::<shooting::Practice>()
        .init_resource::<level::LevelAssets>()
        .insert_resource(level::layout())
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: format!("{}/asset", env!("CARGO_MANIFEST_DIR")),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Münzparcours & Zieltraining | KayKit Demo".into(),
                        resolution: if offscreen {
                            WindowResolution::new(1440, 900).with_scale_factor_override(1.0)
                        } else {
                            WindowResolution::new(1440, 900)
                        },
                        resize_constraints: bevy::window::WindowResizeConstraints {
                            min_width: 960.0,
                            min_height: 640.0,
                            ..default()
                        },
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(TabNavigationPlugin)
        .add_systems(
            Startup,
            (level::spawn_level, shooting::spawn_range, ui::spawn_ui),
        )
        .add_observer(shooting::tag_scene_parts)
        .add_systems(
            Update,
            (
                level::check_assets,
                ui::apply_font,
                ui::buttons,
                ui::sync_name,
                game::keyboard_shortcuts,
                game::handle_actions,
                game::move_player,
                shooting::tick_practice,
                shooting::shoot,
                game::click_to_move,
                game::sync_cursor,
                game::update_scene,
                shooting::update_visuals,
                ui::update_ui,
            )
                .chain(),
        );
    if smoke_test {
        std::fs::create_dir_all(format!("{}/artifacts", env!("CARGO_MANIFEST_DIR")))
            .expect("artifacts directory");
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(0.05),
        ))
        .add_systems(Update, smoke::step.before(level::check_assets));
    }
    if offscreen {
        app.add_systems(
            Startup,
            smoke::setup_offscreen
                .after(level::spawn_level)
                .after(shooting::spawn_range)
                .after(ui::spawn_ui),
        );
    }
    app.run();
}
