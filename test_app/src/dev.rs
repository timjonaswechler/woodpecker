//! Optional development tools; normal gameplay does not depend on their state.
mod smoke;

use crate::{launch::Options, level, player::camera, practice, ui};
use bevy::prelude::*;

pub fn install(app: &mut App, options: &Options) {
    if options.smoke_test {
        smoke::prepare_artifacts();
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(0.05),
        ))
        .add_systems(Update, smoke::step.before(level::check_assets));
    }
    if options.offscreen {
        app.insert_resource(bevy::winit::WinitSettings::continuous())
            .add_systems(
                Startup,
                smoke::setup_offscreen
                    .after(camera::spawn_camera)
                    .after(practice::spawn_range)
                    .after(ui::spawn_ui),
            );
    }
}
