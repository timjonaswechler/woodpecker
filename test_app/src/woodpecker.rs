//! Optional Woodpecker wiring; gameplay owns its loading states and transitions.
use bevy::prelude::*;

/// Counts application Last-schedule runs, independently of Woodpecker's private counter.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
struct SimulationTicks {
    ticks: u64,
}

fn count_simulation_tick(mut count: ResMut<SimulationTicks>) {
    count.ticks = count
        .ticks
        .checked_add(1)
        .expect("simulation tick counter exhausted");
}

pub fn enabled() -> bool {
    std::env::var_os("WOODPECKER_ARTIFACT_DIR").is_some()
}

pub fn log_plugin() -> bevy::log::LogPlugin {
    let missing_layer_probe = std::env::args().any(|arg| arg == "--woodpecker-diagnostics")
        && std::env::args().any(|arg| arg == "--woodpecker-no-tracing-layer");
    if enabled() && !missing_layer_probe {
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
    app.init_resource::<SimulationTicks>()
        .register_type::<SimulationTicks>()
        .add_systems(Last, count_simulation_tick)
        .register_type::<crate::level::LevelLoad>()
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
    crate::diagnostics::install(app);
    if std::env::args().any(|arg| arg == "--woodpecker-diagnostics")
        && std::env::args().any(|arg| arg == "--woodpecker-process-probe")
    {
        let root = std::path::PathBuf::from(std::env::var_os("WOODPECKER_ARTIFACT_DIR").unwrap());
        std::fs::write(
            root.join("process-probe.json"),
            serde_json::json!({"pid": std::process::id()}).to_string(),
        )
        .unwrap();
        app.add_systems(Last, write_process_probe.after(count_simulation_tick));
    }
}

/// Existing independent counter, observed without adding a schedule run or input.
/// Atomic replacement lets a client read while a paced Warp is active.
fn write_process_probe(count: Res<SimulationTicks>) {
    let root = std::path::PathBuf::from(std::env::var_os("WOODPECKER_ARTIFACT_DIR").unwrap());
    std::fs::write(
        root.join("simulation-ticks.next"),
        serde_json::json!({"ticks": count.ticks}).to_string(),
    )
    .unwrap();
    std::fs::rename(
        root.join("simulation-ticks.next"),
        root.join("simulation-ticks.json"),
    )
    .unwrap();
}
