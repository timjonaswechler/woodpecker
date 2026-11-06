use bevy::{prelude::*, window::PrimaryWindow};

/// Logical input identity. Headless installs metadata only, never a native handle.
pub(super) fn primary(world: &World) -> Option<(Entity, &Window)> {
    let mut windows = world.iter_entities().filter_map(|entity| {
        entity
            .contains::<PrimaryWindow>()
            .then(|| entity.get::<Window>().map(|window| (entity.id(), window)))
            .flatten()
    });
    let first = windows.next()?;
    windows.next().is_none().then_some(first)
}

pub(super) fn target(world: &World) -> Option<bevy::camera::NormalizedRenderTarget> {
    #[cfg(feature = "offscreen")]
    if let Some(image) = world.get_resource::<super::offscreen::OffscreenTarget>() {
        let target = world
            .get_resource::<super::headless::Viewport>()
            .map_or_else(
                || bevy::camera::RenderTarget::Image(image.0.clone().into()),
                |viewport| viewport.target(),
            );
        return target.normalize(None);
    }
    let (entity, _) = primary(world)?;
    bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(entity)).normalize(None)
}
