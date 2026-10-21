//! Frame-specific proof: loaded CPU assets alone cannot acknowledge EngineReady.
use super::{Diagnostic, State};
use bevy::{
    asset::{AssetId, RenderAssetUsages},
    prelude::*,
    render::{
        Extract, ExtractSchedule, Render, RenderApp, RenderSystems,
        camera::ExtractedCamera,
        mesh::RenderMesh,
        render_asset::RenderAssets,
        render_resource::{CachedPipelineState, PipelineCache},
        renderer::RenderQueue,
        sync_world::MainEntity,
        texture::GpuImage,
        view::{ViewTarget, window::ExtractedWindow},
    },
};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Clone)]
pub(super) struct Ticket {
    window: Entity,
    meshes: Vec<AssetId<Mesh>>,
    images: Vec<AssetId<Image>>,
    result: Arc<OnceLock<Result<(), Diagnostic>>>,
    waiting: Arc<Mutex<String>>,
}

impl Ticket {
    pub(super) fn new(world: &World, window: Entity) -> Self {
        let meshes = world
            .get_resource::<Assets<Mesh>>()
            .map(|assets| {
                assets
                    .iter()
                    .filter(|(_, mesh)| mesh.asset_usage.contains(RenderAssetUsages::RENDER_WORLD))
                    .map(|(id, _)| id)
                    .collect()
            })
            .unwrap_or_default();
        let images = world
            .get_resource::<Assets<Image>>()
            .map(|assets| {
                assets
                    .iter()
                    .filter(|(_, image)| {
                        image.asset_usage.contains(RenderAssetUsages::RENDER_WORLD)
                    })
                    .map(|(id, _)| id)
                    .collect()
            })
            .unwrap_or_default();
        Self {
            window,
            meshes,
            images,
            result: Arc::new(OnceLock::new()),
            waiting: Arc::new(Mutex::new("render extraction".into())),
        }
    }
    pub(super) fn waiting(&self) -> String {
        self.waiting.lock().unwrap().clone()
    }
    fn wait(&self, message: impl Into<String>) {
        *self.waiting.lock().unwrap() = message.into();
    }
    pub(super) fn result(&self) -> Option<Result<(), Diagnostic>> {
        self.result.get().cloned()
    }
}

#[derive(Resource)]
struct Installed;

#[derive(Resource, Default)]
struct Frame {
    ticket: Option<Ticket>,
    prepared: bool,
    consecutive: u8,
    submitted: bool,
}

pub(super) fn install(app: &mut App) {
    let Some(render) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render
        .init_resource::<Frame>()
        .add_systems(ExtractSchedule, extract)
        .add_systems(
            Render,
            prepare
                .after(RenderSystems::Prepare)
                .before(RenderSystems::Render),
        )
        .add_systems(
            Render,
            submitted
                .after(RenderSystems::Render)
                .before(RenderSystems::Cleanup),
        );
    app.insert_resource(Installed);
}

pub(super) fn available(world: &World) -> bool {
    world.contains_resource::<Installed>()
}

fn extract(state: Extract<Res<State>>, mut frame: ResMut<Frame>) {
    let ticket = state.frame.clone();
    let same = match (&ticket, &frame.ticket) {
        (Some(a), Some(b)) => Arc::ptr_eq(&a.result, &b.result),
        _ => false,
    };
    if !same {
        *frame = Frame {
            ticket,
            ..default()
        };
    }
    frame.prepared = false;
}

fn prepare(
    mut frame: ResMut<Frame>,
    windows: Query<(MainEntity, &ExtractedWindow)>,
    cache: Res<PipelineCache>,
    meshes: Option<Res<RenderAssets<RenderMesh>>>,
    images: Option<Res<RenderAssets<GpuImage>>>,
    cameras: Query<(&ExtractedCamera, &ViewTarget)>,
) {
    let Some(ticket) = &frame.ticket else {
        return;
    };
    if frame.submitted || ticket.result.get().is_some() {
        return;
    }
    // Retryable shader loads remain in the cache's waiting set.
    if cache.waiting_pipelines().next().is_some() {
        ticket.wait(format!(
            "{} queued render pipelines",
            cache.waiting_pipelines().count()
        ));
        frame.consecutive = 0;
        return;
    }
    for pipeline in cache.pipelines() {
        if let CachedPipelineState::Err(error) = &pipeline.state {
            let _ = ticket
                .result
                .set(Err(Diagnostic::new("startup_pipeline_failed", error)));
            return;
        }
        if !matches!(pipeline.state, CachedPipelineState::Ok(_)) {
            frame.consecutive = 0;
            return;
        }
    }
    let assets_ready = ticket.meshes.iter().all(|id| {
        meshes
            .as_ref()
            .is_some_and(|assets| assets.get(*id).is_some())
    }) && ticket.images.iter().all(|id| {
        images
            .as_ref()
            .is_some_and(|assets| assets.get(*id).is_some())
    });
    let surface = windows.iter().any(|(entity, window)| {
        entity == ticket.window
            && window.swap_chain_texture_view.is_some()
            && window.swap_chain_texture_view_format.is_some()
    });
    let camera = cameras.iter().any(|(camera, _)| matches!(camera.target,
        Some(bevy::camera::NormalizedRenderTarget::Window(window)) if window.entity() == ticket.window));
    ticket.wait(format!(
        "GPU meshes/images ready={assets_ready}, window surface={surface}, camera={camera}"
    ));
    frame.prepared = assets_ready && surface && camera;
    if !frame.prepared {
        frame.consecutive = 0;
    }
}

fn submitted(mut frame: ResMut<Frame>, queue: Res<RenderQueue>) {
    if !frame.prepared || frame.submitted {
        return;
    }
    frame.consecutive += 1;
    // Require a subsequent prepared frame too: render assets becoming available can
    // queue new pipeline specializations during the next render iteration.
    if frame.consecutive < 2 {
        return;
    }
    let ticket = frame.ticket.as_ref().unwrap().clone();
    frame.submitted = true;
    ticket.wait("GPU completion of prepared window frame");
    queue.on_submitted_work_done(move || {
        let _ = ticket.result.set(Ok(()));
    });
}
