//! Pin the window-view prerequisite to the frame that extracts this capture.
//!
//! Bevy skips the window screenshot copy when its swapchain view or format
//! is absent, but still maps the prepared buffer. A later frame's availability
//! must not validate that earlier, unfilled readback.
use super::Service;
use bevy::{
    prelude::*,
    render::{
        Extract, ExtractSchedule, Render, RenderApp, RenderSystems, sync_world::MainEntity,
        view::window::ExtractedWindow,
    },
};
use std::sync::{Arc, OnceLock};

#[derive(Resource, Default)]
pub(super) struct Frame(Option<(Entity, Arc<OnceLock<bool>>)>);

impl Frame {
    pub(super) fn new(window: Entity, result: Arc<OnceLock<bool>>) -> Self {
        Self(Some((window, result)))
    }
}

pub(super) fn install(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .init_resource::<Frame>()
        .add_systems(ExtractSchedule, extract)
        .add_systems(
            Render,
            check
                .after(RenderSystems::Prepare)
                .before(RenderSystems::Render),
        );
}

fn extract(service: Extract<Res<Service>>, mut frame: ResMut<Frame>) {
    *frame = service
        .active
        .as_ref()
        .filter(|active| active.surface.get().is_none())
        .map(|active| Frame::new(active.job.window, active.surface.clone()))
        .unwrap_or_default();
}

pub(super) fn check(frame: Res<Frame>, windows: Query<(MainEntity, &ExtractedWindow)>) {
    let Some((window, result)) = &frame.0 else {
        return;
    };
    let available = windows
        .iter()
        .find(|(main, _)| *main == *window)
        .is_some_and(|(_, window)| {
            window.swap_chain_texture_view.is_some()
                && window.swap_chain_texture_view_format.is_some()
        });
    // This is a per-request, first-frame result, not a mutable global flag.
    let _ = result.set(available);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::SubApp;

    #[test]
    fn extracted_window_without_acquired_view_cannot_validate_readback() {
        use bevy::{
            render::render_resource::TextureFormat,
            window::{CompositeAlphaMode, PresentMode},
        };

        let mut main = World::new();
        let window = main.spawn_empty().id();
        let mut render = World::new();
        // The render-world entity has a different identity from the target.
        render.spawn_empty();
        let render_window = render
            .spawn((
                MainEntity::from(window),
                ExtractedWindow {
                    physical_width: 640,
                    physical_height: 360,
                    present_mode: PresentMode::AutoVsync,
                    desired_maximum_frame_latency: None,
                    swap_chain_texture_view: None,
                    swap_chain_texture: None,
                    swap_chain_texture_format: Some(TextureFormat::Bgra8UnormSrgb),
                    swap_chain_texture_view_format: Some(TextureFormat::Bgra8UnormSrgb),
                    size_changed: false,
                    present_mode_changed: false,
                    alpha_mode: CompositeAlphaMode::Auto,
                    needs_initial_present: false,
                },
            ))
            .id();
        assert_ne!(render_window, window);
        let result = Arc::new(OnceLock::new());
        render.insert_resource(Frame::new(window, result.clone()));
        render.run_system_cached(check).unwrap();
        assert_eq!(result.get(), Some(&false));
    }

    #[test]
    fn surface_guard_observes_preparation_before_render_on_bevys_real_base_schedule() {
        let mut app = App::new();
        let mut render = SubApp::new();
        render.add_schedule(Render::base_schedule());
        app.insert_sub_app(RenderApp, render);
        install(&mut app);
        let result = Arc::new(OnceLock::new());
        let window = app.world_mut().spawn_empty().id();
        let render = app.sub_app_mut(RenderApp);
        render
            .world_mut()
            .insert_resource(Frame::new(window, result.clone()));
        render.add_systems(
            Render,
            (
                (|frame: Res<Frame>| {
                    assert!(
                        frame.0.as_ref().unwrap().1.get().is_none(),
                        "guard ran before preparation finished"
                    );
                })
                .in_set(RenderSystems::Prepare),
                (|frame: Res<Frame>| {
                    assert_eq!(
                        frame.0.as_ref().unwrap().1.get(),
                        Some(&false),
                        "guard did not inspect this frame before rendering"
                    );
                })
                .in_set(RenderSystems::Render),
            ),
        );
        render.world_mut().run_schedule(Render);
        assert_eq!(result.get(), Some(&false));
    }
}
