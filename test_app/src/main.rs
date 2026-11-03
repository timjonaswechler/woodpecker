use test_app::app;
#[cfg(feature = "automation")]
use test_app::woodpecker;

fn main() {
    let options = app::launch::Options::from_env();
    let mut app = bevy::prelude::App::new();
    #[cfg(feature = "automation")]
    let log_plugin = woodpecker::log_plugin();
    #[cfg(not(feature = "automation"))]
    let log_plugin = bevy::log::LogPlugin::default();

    app::install(&mut app, &options, log_plugin);
    #[cfg(feature = "automation")]
    woodpecker::install(&mut app);
    let controlled = std::env::var_os("WOODPECKER_ARTIFACT_DIR").is_some();
    test_app::performance::install(
        &mut app,
        options.performance_overlay,
        options.performance_log,
        controlled,
    );
    app.run();
}
