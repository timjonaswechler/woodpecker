//! Builds the playable application and owns the order of its systems.
use crate::level;
pub(crate) mod flow;
#[cfg(feature = "automation")]
mod headless;
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
    #[cfg(feature = "automation")]
    let headless_requested = options.headless.is_some();
    #[cfg(not(feature = "automation"))]
    let headless_requested = false;
    assert!(
        cfg!(feature = "native") != headless_requested,
        "native launches require default native features; headless requires --no-default-features --features automation and an explicit launch.headless request"
    );
    settings::install(app);
    schedule::install(app);
    let resolution = match options.scale_factor {
        Some(scale) => WindowResolution::new(1440, 900).with_scale_factor_override(scale),
        None => WindowResolution::new(1440, 900),
    };
    let plugins = DefaultPlugins
        .set(log_plugin)
        .set(AssetPlugin {
            file_path: crate::ASSET_DIR.into(),
            ..default()
        })
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Signalstation | Räume und Entdeckungen".into(),
                focused: !options.start_unfocused,
                present_mode: if options.uncapped {
                    bevy::window::PresentMode::AutoNoVsync
                } else {
                    bevy::window::PresentMode::default()
                },
                position: options.window_position.map_or(
                    bevy::window::WindowPosition::Automatic,
                    bevy::window::WindowPosition::At,
                ),
                resizable: !options.fixed_window,
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
        });
    #[cfg(feature = "automation")]
    let plugins = headless::plugins(plugins, options.headless.as_ref());
    app.insert_resource(ClearColor(Color::srgb(0.63, 0.66, 0.63)))
        .add_plugins(plugins)
        .add_plugins(TabNavigationPlugin);
    #[cfg(feature = "automation")]
    if let Some(configuration) = &options.headless {
        app.add_plugins(woodpecker::session::headless::Plugin(configuration.clone()));
    }
    flow::install(app);
    input::install(app);
    player::install(app);
    level::install(app);
    ui::install(app);
}
