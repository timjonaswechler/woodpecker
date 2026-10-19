//! Optional Woodpecker wiring; gameplay owns its loading states and transitions.
use bevy::prelude::*;

pub fn enabled() -> bool {
    std::env::var_os("WOODPECKER_ARTIFACT_DIR").is_some()
}

pub fn log_plugin() -> bevy::log::LogPlugin {
    if enabled() {
        bevy::log::LogPlugin {
            custom_layer: woodpecker::session::tracing_error_layer,
            ..default()
        }
    } else {
        default()
    }
}

pub fn install(app: &mut App) {
    if !enabled() {
        return;
    }
    app.register_type::<crate::level::LevelLoad>()
        .register_type::<crate::player::Player>()
        .register_type::<crate::player::camera::ViewState>()
        .register_type::<crate::player::camera::ViewMode>()
        .register_type::<crate::input::PointerCapture>()
        .register_type::<crate::app::flow::GameFlow>()
        .register_type::<crate::level::mission::Mission>()
        .register_type::<crate::level::objects::shooting_range::ShootingRange>()
        .register_type::<crate::level::mission::inventory::CoinRun>()
        .register_type::<crate::ui::UiState>()
        .register_type::<crate::app::settings::Settings>()
        .register_type::<ButtonInput<KeyCode>>()
        .register_type::<ButtonInput<MouseButton>>()
        .insert_resource(woodpecker::session::startup::Assets(|world| {
            world
                .get_resource::<crate::level::LevelAssets>()
                .map(|assets| assets.handles().to_vec())
        }))
        .insert_resource(crate::input::InputMode::Controlled(
            woodpecker::session::VIRTUAL_POINTER_ID,
        ))
        .insert_resource(bevy::winit::WinitSettings::continuous())
        .add_plugins(woodpecker::session::Plugin);
}
