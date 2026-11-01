//! Explicit diagnostic actions for controlled acceptance runs, never ordinary play.
use bevy::prelude::*;

mod clock;
mod reflection;
mod reports;
mod startup;

pub(crate) fn install(app: &mut App) {
    if !crate::woodpecker::enabled()
        || !std::env::args().any(|argument| argument == "--woodpecker-diagnostics")
    {
        return;
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
