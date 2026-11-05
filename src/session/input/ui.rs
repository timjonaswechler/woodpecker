//! Adapt Bevy's legacy Interaction system without exposing the virtual cursor to Winit.
use bevy::{
    camera::{NormalizedRenderTarget, RenderTarget},
    ecs::schedule::ScheduleCleanupPolicy,
    input::InputSystems,
    picking::pointer::{PointerLocation, PointerMap},
    prelude::*,
    ui::{UiPlugin, UiSystems, ui_focus_system},
};

pub(super) fn install(app: &mut App) {
    if app.is_plugin_added::<UiPlugin>() {
        // The original system reads the physical Window cursor. Replace just that
        // system's view, leaving the rest of the UI and native window backend intact.
        app.world_mut()
            .schedule_scope(PreUpdate, |world, schedule| {
                schedule
                    .remove_systems_in_set(
                        ui_focus_system,
                        world,
                        ScheduleCleanupPolicy::RemoveSystemsOnly,
                    )
                    .expect("replace native UI focus system");
            });
        app.add_systems(
            PreUpdate,
            focus
                .in_set(UiSystems::Focus)
                .after(InputSystems)
                .after(bevy::picking::PickingSystems::ProcessInput),
        );
    }
}

fn focus(world: &mut World) {
    let location = world
        .resource::<PointerMap>()
        .get_entity(super::pointer::ID)
        .and_then(|entity| world.get::<PointerLocation>(entity))
        .and_then(|pointer| pointer.location.clone());
    #[cfg(feature = "offscreen")]
    let image = world
        .get_resource::<crate::session::headless::Viewport>()
        .map(|viewport| (viewport.window, viewport.target()));
    #[cfg(not(feature = "offscreen"))]
    let image: Option<(Entity, RenderTarget)> = None;
    let mut positions = Vec::new();
    for (entity, mut window) in world.query::<(Entity, &mut Window)>().iter_mut(world) {
        positions.push((entity, window.internal));
        let virtual_position = location.as_ref().and_then(|location| {
            (matches!(location.target, NormalizedRenderTarget::Window(target) if target.entity() == entity)
                || image.as_ref().is_some_and(|(window, target)| *window == entity
                    && target.normalize(None).as_ref() == Some(&location.target)))
                .then_some(location.position)
        });
        window
            .bypass_change_detection()
            .set_cursor_position(virtual_position);
    }
    // The upstream legacy system accepts Window targets only. For this cached
    // call alone, project the image cameras onto their matching logical Window.
    // Restore targets without Changed<> events before any other system can run.
    let mut targets = Vec::new();
    if let Some((window, image)) = &image {
        for (entity, mut target) in world
            .query_filtered::<(Entity, &mut RenderTarget), With<Camera>>()
            .iter_mut(world)
        {
            if target.normalize(None) == image.normalize(None) {
                targets.push((entity, target.clone()));
                *target.bypass_change_detection() =
                    RenderTarget::Window(bevy::window::WindowRef::Entity(*window));
            }
        }
    }
    let result = world.run_system_cached(ui_focus_system);
    for (entity, target) in targets {
        if let Some(mut current) = world.get_mut::<RenderTarget>(entity) {
            *current.bypass_change_detection() = target;
        }
    }
    // Restore even on a system-parameter error, before any backend system can run.
    for (entity, internal) in positions {
        if let Some(mut window) = world.get_mut::<Window>(entity) {
            window.bypass_change_detection().internal = internal;
        }
    }
    result.expect("virtual UI focus system");
}
