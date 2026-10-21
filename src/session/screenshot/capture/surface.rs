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
