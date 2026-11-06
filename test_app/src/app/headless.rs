//! App-owned graphics installation. No Winit or system clipboard in this build.
use bevy::{
    app::{PluginGroupBuilder, ScheduleRunnerPlugin},
    prelude::*,
    render::{
        RenderPlugin,
        pipelined_rendering::PipelinedRenderingPlugin,
        settings::{Backends, RenderCreation, WgpuSettings},
    },
    window::ExitCondition,
};
use woodpecker::session::launch::Headless;

pub(super) fn plugins(
    plugins: PluginGroupBuilder,
    configuration: Option<&Headless>,
) -> PluginGroupBuilder {
    if configuration.is_none() {
        return plugins;
    }
    // Native features are excluded at Cargo resolution, not merely hidden at runtime.
    // Keep the game's normal GPU features, MSAA, tonemapping and materials.
    plugins
        .set(WindowPlugin {
            primary_window: None,
            exit_condition: ExitCondition::DontExit,
            ..default()
        })
        .set(RenderPlugin {
            render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
                backends: Some(Backends::METAL),
                ..default()
            })),
            synchronous_pipeline_compilation: false,
            ..default()
        })
        .disable::<PipelinedRenderingPlugin>()
        .add(ScheduleRunnerPlugin::run_loop(
            std::time::Duration::from_millis(1),
        ))
}
