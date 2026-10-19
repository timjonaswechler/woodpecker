//! Builds the playable application and owns the order of its systems.
use crate::level;
pub(crate) mod flow;
pub mod launch;
pub(crate) mod schedule;
pub(crate) mod settings;
use crate::{
    app::launch::Options,
    input,
    player::{self},
    ui,
};
use bevy::{
    input_focus::tab_navigation::TabNavigationPlugin, prelude::*, window::WindowResolution,
};

pub fn install(app: &mut App, options: &Options, log_plugin: bevy::log::LogPlugin) {
    settings::install(app, options.smoke_test);
    schedule::install(app);
    let resolution = match options.scale_factor.or(options.offscreen.then_some(1.0)) {
        Some(scale) => WindowResolution::new(1440, 900).with_scale_factor_override(scale),
        None => WindowResolution::new(1440, 900),
    };
    app.insert_resource(ClearColor(Color::srgb(0.63, 0.66, 0.63)))
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
                            // Winit interprets minimum bounds in desktop points even
                            // with a scale override. Keep them below the pixel size;
                            // `resizable: false` prevents interactive resizing.
                            bevy::window::WindowResizeConstraints {
                                min_width: 1.0,
                                min_height: 1.0,
                                max_width: 1440.0,
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
        .add_plugins(TabNavigationPlugin);
    flow::install(app);
    input::install(app);
    player::install(app);
    level::install(app);
    ui::install(app);
}
