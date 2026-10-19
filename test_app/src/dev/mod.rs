//! Optional development tools; normal gameplay does not depend on their state.
mod smoke;

use crate::{app::launch::Options, level, player::camera, ui};
use bevy::prelude::*;

pub fn install(app: &mut App, options: &Options) {
    if options.smoke_test {
        smoke::prepare_artifacts();
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(0.05),
        ))
        .add_systems(
            Update,
            smoke::step.before(crate::app::schedule::Phase::Input),
        );
    }
    if options.offscreen {
        app.insert_resource(bevy::winit::WinitSettings::continuous())
            .add_systems(
                Startup,
                smoke::setup_offscreen
                    .after(camera::spawn_camera)
                    .after(level::objects::shooting_range::spawn_range)
                    .after(crate::player::blaster::spawn)
                    .after(ui::spawn_ui),
            );
    }
}
