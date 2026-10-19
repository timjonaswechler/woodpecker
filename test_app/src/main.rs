#[cfg(feature = "automation")]
use test_app::woodpecker;
use test_app::{app, dev};

fn main() {
    let options = app::launch::Options::from_env();
    let mut app = bevy::prelude::App::new();
    #[cfg(feature = "automation")]
    let log_plugin = woodpecker::log_plugin();
    #[cfg(not(feature = "automation"))]
    let log_plugin = bevy::log::LogPlugin::default();

    app::install(&mut app, &options, log_plugin);
    dev::install(&mut app, &options);
    #[cfg(feature = "automation")]
    woodpecker::install(&mut app);
    app.run();
}
