//! Narrow image-target adapter. Native capture does not use this module's state.
use super::protocol::Diagnostic;
use bevy::{
    asset::{AssetId, RenderAssetUsages, handle_internal_asset_events},
    prelude::*,
    render::{
        Extract, ExtractSchedule, RenderApp,
        camera::ExtractedCamera,
        error_handler::{RenderError, RenderErrorHandler, RenderErrorPolicy},
        pipelined_rendering::PipelinedRenderingPlugin,
        render_asset::RenderAssets,
        render_resource::{CachedPipelineState, PipelineCache},
        renderer::{RenderAdapterInfo, RenderDevice, RenderGraph, RenderGraphSystems, RenderQueue},
        texture::GpuImage,
        view::ViewTarget,
    },
    shader::{Shader, ShaderCacheError},
};
use std::{
    collections::HashSet,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

/// Opt into image capture instead of primary-window capture with the `offscreen` feature.
/// Insert before [`super::Plugin`]. The application owns the image and camera setup:
/// no window/surface, Metal hardware, non-pipelined renderer, an active Camera2d
/// or basic Camera3d targeting this image, single-sampled D2 Rgba8UnormSrgb with
/// COPY_SRC usage. The qualified 3D profile uses asynchronous pipeline compilation.
/// Rendering starts only after the first explicitly counted tick. This is not
/// engine-startup readiness or general 3D-effects/UI/asset-loading support.
#[derive(Resource, Clone)]
pub struct OffscreenTarget(pub Handle<Image>);

#[derive(Resource, Clone, Default)]
pub(super) struct Status {
    fatal: Arc<OnceLock<Diagnostic>>,
    stopped: Arc<AtomicBool>,
}

impl Status {
    pub(super) fn error(&self) -> Option<Diagnostic> {
        self.fatal.get().cloned()
    }
    fn fail(&self, error: Diagnostic) {
        let _ = self.fatal.set(error);
    }
    pub(super) fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
    }
    pub(super) fn guard(&self, since: Instant) -> Guard {
        Guard {
            status: self.clone(),
            since,
        }
    }
}

#[derive(Clone)]
pub(super) struct Guard {
    status: Status,
    since: Instant,
}
impl Guard {
    pub(super) fn check(&self) -> Result<(), Diagnostic> {
        if let Some(error) = self.status.error() {
            return Err(error);
        }
        if self.status.stopped.load(Ordering::SeqCst) {
            return Err(Diagnostic::new(
                "screenshot_cancelled",
                "session is terminating",
            ));
        }
        if self.since.elapsed() >= Duration::from_secs(30) {
            return Err(Diagnostic::new(
                "screenshot_failed",
                "GPU capture deadline expired",
            ));
        }
        Ok(())
    }
}

struct Ticket {
    entity: Entity,
    image: Handle<Image>,
    copied: AtomicBool,
    surface: Arc<OnceLock<bool>>,
}

type Delivery = (Entity, Result<Image, Diagnostic>);
#[derive(Resource)]
struct Reader {
    ticket: Option<Arc<Ticket>>,
    receiver: Mutex<mpsc::Receiver<Delivery>>,
}
#[derive(Resource, Default)]
struct Frame {
    ticket: Option<Arc<Ticket>>,
    stable: u8,
}
#[derive(Resource, Default)]
struct Shaders(HashSet<AssetId<Shader>>);

pub(super) fn install(app: &mut App) {
    if !app.world().contains_resource::<OffscreenTarget>() {
        return;
    }
    app.init_resource::<Status>();
    let admitted = app.get_sub_app(RenderApp).is_some()
        && !app.is_plugin_added::<PipelinedRenderingPlugin>()
        && app
            .world()
            .get_resource::<RenderAdapterInfo>()
            .is_some_and(|info| {
                info.backend == wgpu::Backend::Metal
                    && matches!(
                        info.device_type,
                        wgpu::DeviceType::IntegratedGpu | wgpu::DeviceType::DiscreteGpu
                    )
            });
    if !admitted {
        app.world().resource::<Status>().fail(Diagnostic::new(
            "render_profile_unsupported",
            "offscreen requires a non-pipelined Metal hardware renderer",
        ));
        return;
    }
    let (sender, receiver) = mpsc::channel();
    app.insert_resource(RenderErrorHandler(render_error))
        .insert_resource(Reader {
            ticket: None,
            receiver: Mutex::new(receiver),
        })
        .init_resource::<Shaders>();
    let status = app.world().resource::<Status>().clone();
    app.sub_app_mut(RenderApp)
        .insert_resource(status)
        .insert_resource(ReadbackSender(sender))
        .init_resource::<Frame>()
        .add_systems(ExtractSchedule, extract)
        .add_systems(
            RenderGraph,
            copy.after(RenderGraphSystems::Submit)
                .before(RenderGraphSystems::Finish),
        );
}

// wgpu loss notification can lag a GPU operation that already panics inside
// Bevy. End this offscreen session rather than resuming a partially run schedule.
// This does not catch aborts or promise recovery of either world.
pub(super) fn protect(status: Option<Status>, run: impl FnOnce()) {
    let Some(status) = status else {
        run();
        return;
    };
    if let Err(payload) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(run)) {
        let message = payload
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| payload.downcast_ref::<&str>().copied())
            .unwrap_or("render schedule panicked");
        status.fail(Diagnostic::new("render_panicked", message));
    }
}

fn render_error(error: &RenderError, main: &mut World, _: &mut World) -> RenderErrorPolicy {
    let code = match error.ty {
        bevy::render::error_handler::ErrorType::DeviceLost => "render_device_lost",
        bevy::render::error_handler::ErrorType::OutOfMemory => "render_out_of_memory",
        bevy::render::error_handler::ErrorType::Validation => "render_validation",
        _ => "render_internal",
    };
    main.resource::<Status>()
        .fail(Diagnostic::new(code, &error.description));
    RenderErrorPolicy::StopRendering
}

/// Poll only technical asset progress. Do not execute application schedules or
/// publish synthetic shader events to application MessageReaders. Ordinary shader
/// modification/removal events still follow Bevy's extraction after explicit ticks.
pub(super) fn shaders(main: &mut World, render: &mut World) {
    if !main.contains_resource::<Reader>() {
        return;
    }
    bevy::tasks::tick_global_task_pools_on_main_thread();
    handle_internal_asset_events(main);
    let ids: Vec<_> = main.resource::<Assets<Shader>>().ids().collect();
    for id in ids {
        if main.resource_mut::<Shaders>().0.insert(id) {
            let shader = main.resource::<Assets<Shader>>().get(id).unwrap().clone();
            render
                .resource_mut::<PipelineCache>()
                .set_shader(id, shader);
        }
    }
}

pub(super) fn target(world: &World) -> Result<Handle<Image>, Diagnostic> {
    let handle = world.resource::<OffscreenTarget>().0.clone();
    let valid = world
        .get_resource::<Assets<Image>>()
        .and_then(|images| images.get(&handle))
        .is_some_and(|image| {
            let d = &image.texture_descriptor;
            d.dimension == wgpu::TextureDimension::D2
                && d.size.depth_or_array_layers == 1
                && d.size.width > 0
                && d.size.height > 0
                && d.sample_count == 1
                && d.format == wgpu::TextureFormat::Rgba8UnormSrgb
                && d.usage.contains(
                    wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
                )
        });
    if !valid {
        return Err(Diagnostic::new(
            "render_target_unavailable",
            "expected a single-sampled D2 RGBA8-sRGB render/copy image",
        ));
    }
    Ok(handle)
}

pub(super) fn request(
    world: &mut World,
    entity: Entity,
    image: Handle<Image>,
    surface: Arc<OnceLock<bool>>,
) {
    world.resource_mut::<Reader>().ticket = Some(Arc::new(Ticket {
        entity,
        image,
        surface,
        copied: AtomicBool::new(false),
    }));
}
pub(super) fn deliveries(world: &World) -> Vec<Delivery> {
    world
        .get_resource::<Reader>()
        .map(|reader| reader.receiver.lock().unwrap().try_iter().collect())
        .unwrap_or_default()
}

fn extract(reader: Extract<Res<Reader>>, mut frame: ResMut<Frame>) {
    let same = match (&reader.ticket, &frame.ticket) {
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    };
    if !same {
        frame.stable = 0;
    }
    frame.ticket = reader.ticket.clone();
}

fn pipeline_failure(state: &CachedPipelineState) -> Option<Diagnostic> {
    match state {
        CachedPipelineState::Err(
            ShaderCacheError::ShaderNotLoaded(_) | ShaderCacheError::ShaderImportNotYetAvailable,
        ) => None,
        CachedPipelineState::Err(error) => Some(Diagnostic::new("render_pipeline_failed", error)),
        _ => None,
    }
}

fn copy(world: &mut World) {
    let device = world.resource::<RenderDevice>().wgpu_device().clone();
    // Nonblocking callback progress, including while the simulation is paused.
    let _ = device.poll(wgpu::PollType::Poll);
    let Some(ticket) = world.resource::<Frame>().ticket.clone() else {
        return;
    };
    if ticket.copied.load(Ordering::SeqCst) || world.resource::<Status>().error().is_some() {
        return;
    }
    let cache = world.resource::<PipelineCache>();
    for pipeline in cache.pipelines() {
        if let Some(error) = pipeline_failure(&pipeline.state) {
            world.resource::<Status>().fail(error);
            return;
        }
    }
    let pipelines_ready = cache.waiting_pipelines().next().is_none()
        && cache
            .pipelines()
            .all(|p| matches!(p.state, CachedPipelineState::Ok(_)));
    let mut cameras = world.query::<(&ExtractedCamera, &ViewTarget)>();
    let views = cameras
        .iter(world)
        .filter(|(camera, view)| matches!(camera.target.as_ref(), Some(bevy::camera::NormalizedRenderTarget::Image(target)) if target.handle == ticket.image) && view.needs_present())
        .count();
    // Bevy UI adds a second view of the same final image. All views have been
    // submitted here; the image must be unique, not its contributing views.
    if !pipelines_ready || views == 0 {
        world.resource_mut::<Frame>().stable = 0;
        return;
    }
    world.resource_mut::<Frame>().stable = world.resource::<Frame>().stable.saturating_add(1);
    if world.resource::<Frame>().stable < 2 {
        return;
    }
    let images = world.resource::<RenderAssets<GpuImage>>();
    let Some(image) = images.get(&ticket.image) else {
        return;
    };
    let descriptor = &image.texture_descriptor;
    if descriptor.format != wgpu::TextureFormat::Rgba8UnormSrgb
        || descriptor.sample_count != 1
        || descriptor.dimension != wgpu::TextureDimension::D2
        || descriptor.size.depth_or_array_layers != 1
    {
        world.resource::<Status>().fail(Diagnostic::new(
            "render_target_unavailable",
            "GPU target no longer matches the capture profile",
        ));
        return;
    }
    let size = descriptor.size;
    let row = size.width * 4;
    let stride = row.div_ceil(256) * 256;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("woodpecker-offscreen-readback"),
        size: u64::from(stride) * u64::from(size.height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("woodpecker-offscreen-copy"),
    });
    encoder.copy_texture_to_buffer(
        image.texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(stride),
                rows_per_image: Some(size.height),
            },
        },
        size,
    );
    world.resource::<RenderQueue>().submit([encoder.finish()]);
    ticket.copied.store(true, Ordering::SeqCst);
    let _ = ticket.surface.set(true);
    // Entity generation and pinned target travel with this exact copy/map. A
    // delayed callback cannot consume another request, even if its numeric id is reused.
    let sender = world.resource::<ReadbackSender>().0.clone();
    let status = world.resource::<Status>().clone();
    let mapped = buffer.clone();
    buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            let result = result
                .map_err(|error| Diagnostic::new("render_map_failed", error))
                .and_then(|()| {
                    let data = mapped
                        .slice(..)
                        .get_mapped_range()
                        .map_err(|error| Diagnostic::new("render_map_failed", error))?;
                    let mut pixels = Vec::with_capacity(row as usize * size.height as usize);
                    for line in data.chunks_exact(stride as usize) {
                        pixels.extend_from_slice(&line[..row as usize]);
                    }
                    drop(data);
                    mapped.unmap();
                    Ok(Image::new(
                        size,
                        wgpu::TextureDimension::D2,
                        pixels,
                        wgpu::TextureFormat::Rgba8UnormSrgb,
                        RenderAssetUsages::MAIN_WORLD,
                    ))
                });
            if let Err(error) = &result {
                status.fail(error.clone());
            }
            let _ = sender.send((ticket.entity, result));
        });
}

#[derive(Resource)]
struct ReadbackSender(mpsc::Sender<Delivery>);

#[cfg(test)]
#[path = "offscreen/tests/test_terminal.rs"]
mod test_terminal;
