use super::protocol::{Diagnostic, Message};
use bevy::prelude::*;

#[cfg(feature = "screenshot")]
mod capture;
#[cfg(feature = "screenshot")]
mod destination;

pub(super) fn install(app: &mut App, root: &std::path::Path) {
    #[cfg(feature = "screenshot")]
    capture::install(app, root);
    #[cfg(not(feature = "screenshot"))]
    let _ = (app, root);
}

pub(super) fn available(world: &World) -> bool {
    #[cfg(feature = "screenshot")]
    return world.contains_resource::<capture::Service>();
    #[cfg(not(feature = "screenshot"))]
    {
        let _ = world;
        false
    }
}

pub(super) fn pending(world: &World, id: u64) -> bool {
    #[cfg(feature = "screenshot")]
    return world
        .get_resource::<capture::Service>()
        .is_some_and(|s| s.pending(id));
    #[cfg(not(feature = "screenshot"))]
    {
        let _ = (world, id);
        false
    }
}

pub(super) fn start(world: &mut World, id: u64, path: String) -> Result<(), Diagnostic> {
    validate(&path)?;
    #[cfg(feature = "screenshot")]
    {
        if available(world) && super::plugin::render_waiting(world) {
            return Err(Diagnostic::new(
                "screenshot_window_unavailable",
                "primary window has no rendered surface before the first explicit tick",
            ));
        }
        capture::start(world, id, path)
    }
    #[cfg(not(feature = "screenshot"))]
    {
        let _ = (world, id);
        Err(Diagnostic::new(
            "screenshot_unavailable",
            "enable the screenshot feature in the application",
        ))
    }
}

pub(super) fn poll(world: &mut World) -> Vec<Message> {
    #[cfg(feature = "screenshot")]
    return capture::poll(world);
    #[cfg(not(feature = "screenshot"))]
    {
        let _ = world;
        Vec::new()
    }
}

fn validate(path: &str) -> Result<(), Diagnostic> {
    if path.is_empty()
        || !path.ends_with(".png")
        || path.contains(['\\', '\0'])
        || path.as_bytes().get(1) == Some(&b':')
        || path.split('/').any(|part| matches!(part, "" | "." | ".."))
    {
        return Err(Diagnostic::new(
            "invalid_screenshot_path",
            "expected a normalized relative .png path",
        ));
    }
    Ok(())
}
