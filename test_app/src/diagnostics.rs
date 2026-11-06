//! Explicit diagnostic actions for controlled acceptance runs, never ordinary play.
use bevy::prelude::*;

mod clock;
mod fixture;
mod reflection;
mod reports;
mod startup;

pub(crate) fn install(app: &mut App) {
    if !crate::woodpecker::enabled()
        || !std::env::args().any(|argument| argument == "--woodpecker-diagnostics")
    {
        return;
    }
    if std::env::args().any(|argument| argument == "--woodpecker-clock-snapshot") {
        clock::install_snapshot(app);
    }
    if std::env::args().any(|argument| argument == "--woodpecker-render-loss") {
        app.add_systems(
            Update,
            lose_device.in_set(crate::app::schedule::Phase::WorldActions),
        );
    }
    if std::env::args().any(|argument| argument == "--woodpecker-legacy-ui") {
        app.register_type::<bevy::ui::DeprecatedInteraction>()
            .add_systems(PostStartup, legacy_ui);
    }
    if std::env::args().any(|argument| argument == "--woodpecker-test-fixture") {
        fixture::install(app);
    }
    if let Some(mode) = crate::app::launch::value::<String>("--woodpecker-startup") {
        startup::install(app, &mode);
    }
    if let Some(mode) = crate::app::launch::value::<String>("--woodpecker-clock") {
        clock::install(app, &mode);
    }
    if std::env::args().any(|argument| argument == "--woodpecker-reflection") {
        reflection::install(app);
    }
    if let Some(mode) = crate::app::launch::value::<String>("--woodpecker-report-action") {
        reports::install(app, mode);
    }
    app.add_systems(
        Update,
        trigger.in_set(crate::app::schedule::Phase::WorldActions),
    );
}

// Observe the existing laid-out buttons through both supported UI contracts.
// No synthetic UI, input, ticks or additional application actions are introduced.
fn legacy_ui(mut commands: Commands, buttons: Query<Entity, With<bevy::ui_widgets::Button>>) {
    for entity in &buttons {
        commands
            .entity(entity)
            .insert(bevy::ui::DeprecatedInteraction::None);
    }
}

// Deliberately destroys only this test-owned logical device, never the adapter,
// driver, OS display, another application or a hardware device.
fn lose_device(keys: Res<ButtonInput<KeyCode>>, device: Res<bevy::render::renderer::RenderDevice>) {
    if keys.just_pressed(KeyCode::F10) {
        device.wgpu_device().destroy();
    }
}

fn trigger(keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::F12) {
        error!(
            target: "signalstation::diagnostics",
            "Signalstation diagnostic tracing error"
        );
    }
    if keys.just_pressed(KeyCode::F11) {
        panic!("Signalstation diagnostic panic");
    }
}
