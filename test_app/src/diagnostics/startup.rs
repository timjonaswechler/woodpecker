//! Application-side prerequisites for the public engine-startup failure cases.
use bevy::prelude::*;
use woodpecker::session::startup::Assets as StartupAssets;

#[derive(Resource)]
struct MissingAsset(Handle<Image>);

pub(super) fn install(app: &mut App, mode: &str) {
    match mode {
        "pending" => {
            app.insert_resource(StartupAssets(|_| None));
        }
        "missing-scope" => {
            app.world_mut().remove_resource::<StartupAssets>();
        }
        "missing-asset" => {
            let handle = app
                .world()
                .resource::<AssetServer>()
                .load("__woodpecker_missing_startup__.png");
            app.insert_resource(MissingAsset(handle))
                .insert_resource(StartupAssets(|world| {
                    Some(vec![world.resource::<MissingAsset>().0.clone().untyped()])
                }));
        }
        "no-camera" => {
            // Keep the renderer and real window; only the application's cameras
            // are inactive. The session must not claim a completed camera frame.
            app.add_systems(PostStartup, disable_cameras);
        }
        _ => panic!(
            "invalid --woodpecker-startup: {mode}; expected pending, missing-scope, missing-asset or no-camera"
        ),
    }
}

fn disable_cameras(mut cameras: Query<&mut Camera>) {
    for mut camera in &mut cameras {
        camera.is_active = false;
    }
}
