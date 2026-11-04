//! FOLLOW-UP STUDY. Copy of our own frozen fixture, modified only in this new directory.
#![allow(dead_code)]
use bevy::{
    app::{AppLabel, MainScheduleOrder},
    camera::{RenderTarget, ScalingMode},
    core_pipeline::tonemapping::Tonemapping,
    ecs::schedule::{ScheduleLabel, SingleThreadedExecutor},
    prelude::*,
    render::{
        Render, RenderApp, RenderPlugin, ExtractSchedule,
        camera::ExtractedCamera,
        error_handler::{RenderError, RenderErrorHandler, RenderErrorPolicy},
        render_asset::RenderAssets,
        renderer::{RenderAdapter, RenderAdapterInfo, RenderDevice, RenderGraph, RenderGraphSystems, RenderInstance, RenderQueue},
        settings::RenderCreation,
        texture::GpuImage,
        view::ViewTarget,
    },
    time::TimeUpdateStrategy,
    window::{ExitCondition, WindowPlugin},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{borrow::Cow, collections::HashSet, path::{Path, PathBuf}, sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc}, time::{Duration, Instant}};

const W: u32 = 65;
const H: u32 = 49;
const STRIDE: u32 = 512; // align_up(65 * 4, 256)
const DT: Duration = Duration::from_nanos(16_666_667);
const MAX_PUMPS: u64 = 64;
fn event(name: &str, data: Value) {
    let _serial=TRACE_LOCK.lock().unwrap();
    let seq = TRACE_SEQ.fetch_add(1, Ordering::SeqCst);
    let ns = TRACE_ORIGIN.get_or_init(Instant::now).elapsed().as_nanos() as u64;
    // The real product writer intentionally owns stdout for its protocol lifetime.
    // Probe instrumentation must use stderr, or it deadlocks behind that lock.
    eprintln!("{}", json!({"event":name,"seq":seq,"t_ns":ns,"data":data}));
}
fn hash(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }

#[derive(Resource, Default)]
struct Counters { ticks: u64, updates: u64, fixed: u64, startup: u64, pending: u64, consumed: u64 }
#[derive(Resource, Default)]
struct Fatal { first: Option<String>, calls: u64 }
#[derive(Resource)]
struct Scene { image: Handle<Image>, moving: Entity, material: Handle<ColorMaterial>, sentinel: Entity }
#[derive(Component)] struct Sentinel;
#[derive(Message)] struct PendingMessage;
#[derive(Resource, Default)] struct Request(Option<Token>);
#[derive(Resource, Default)] struct Copies(Vec<Copy>);
#[derive(Clone, Debug, PartialEq, Eq)]
struct Token { request: u64, generation: u64, frame: u64, tick: u64, state: u64 }
impl Token {
    fn json(&self) -> Value { json!({"request":self.request,"generation":self.generation,"frame":self.frame,"tick":self.tick,"state":self.state}) }
}
struct Copy { token: Token, target: AssetId<Image>, buffer: wgpu::Buffer, submission: wgpu::SubmissionIndex, queue_done: Arc<AtomicBool> }
struct Candidate { token: Token, target: AssetId<Image>, submission: String, bytes: Vec<u8>, queue_done: bool }
struct Probe { app: App, pumps: u64, out: PathBuf, started: Instant, forwarded_shaders: HashSet<AssetId<Shader>> }

fn render_error(error: &RenderError, main: &mut World, _: &mut World) -> RenderErrorPolicy {
    let mut fatal = main.resource_mut::<Fatal>();
    fatal.calls += 1;
    if fatal.first.is_none() { fatal.first = Some(format!("{:?}: {}", error.ty, error.description)); }
    event("public_render_error_handler", json!({"type":format!("{:?}",error.ty),"description":error.description,"calls":fatal.calls}));
    RenderErrorPolicy::StopRendering
}
fn startup(mut count: ResMut<Counters>) { count.startup += 1; }
fn fixed(mut count: ResMut<Counters>) { count.fixed += 1; }
fn simulation(mut count: ResMut<Counters>, scene: Res<Scene>, mut materials: ResMut<Assets<ColorMaterial>>, mut transforms: Query<&mut Transform>) {
    count.updates += 1;
    let pending = count.pending;
    count.pending = 0;
    count.consumed += pending;
    if count.updates == 2 {
        materials.get_mut(&scene.material).unwrap().color = Color::srgb_u8(0,255,0);
        transforms.get_mut(scene.moving).unwrap().translation.x = 12.0;
    }
}

// Executes after the real camera graph's submission, not a texture-clear substitute.
// A final-target allocation alone is not considered sufficient; pixels are checked independently.
fn copy_final(world: &mut World) {
    let Some(token) = world.resource::<Request>().0.clone() else { return; };
    let image = world.resource::<TargetImage>().0.clone();
    let target = RenderTarget::Image(image.clone().into()).normalize(None).unwrap();
    let mut q = world.query::<(&ExtractedCamera, &ViewTarget)>();
    let views = q.iter(world).filter(|(camera, view)| camera.target.as_ref() == Some(&target) && view.needs_present()).count();
    if views != 1 { event("copy_not_prepared",json!({"token":token.json(),"views":views})); return; }
    let images = world.resource::<RenderAssets<GpuImage>>();
    let Some(gpu) = images.get(&image) else { return; };
    assert_eq!(gpu.texture_descriptor.format, wgpu::TextureFormat::Rgba8UnormSrgb);
    let device = world.resource::<RenderDevice>().wgpu_device();
    let buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("proof-staging"), size: u64::from(STRIDE*H), usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label:Some("proof-final-copy") });
    encoder.copy_texture_to_buffer(
        gpu.texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo { buffer:&buffer, layout: wgpu::TexelCopyBufferLayout {offset:0,bytes_per_row:Some(STRIDE),rows_per_image:Some(H)} },
        wgpu::Extent3d {width:W,height:H,depth_or_array_layers:1},
    );
    let queue = world.resource::<RenderQueue>();
    let submission = queue.submit([encoder.finish()]);
    let done = Arc::new(AtomicBool::new(false));
    let callback_done = done.clone();
    let callback_token=token.clone();
    queue.on_submitted_work_done(move || {event("queue_callback",callback_token.json());callback_done.store(true,Ordering::SeqCst);});
    event("graph_final_copy_submitted",json!({"token":token.json(),"submission":format!("{submission:?}"),"views":views,"stride":STRIDE}));
    world.resource_mut::<Copies>().0.push(Copy {token,target:image.id(),buffer,submission,queue_done:done});
}
#[derive(Resource)] struct TargetImage(Handle<Image>);

impl Probe {
    fn new(out: PathBuf) -> Self {Self::new_mode(out,false)}
    fn new_mode(out: PathBuf, product: bool) -> Self {
        // Explicit Metal-only admission. Never create a Surface, Winit or display connection.
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor { backends:wgpu::Backends::METAL, flags:wgpu::InstanceFlags::VALIDATION | wgpu::InstanceFlags::DEBUG, ..wgpu::InstanceDescriptor::new_without_display_handle() });
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference:wgpu::PowerPreference::LowPower, compatible_surface:None, force_fallback_adapter:false, apply_limit_buckets:false })).expect("BLOCKER: no Metal adapter; no fallback permitted");
        let info = adapter.get_info();
        assert!(matches!(info.device_type,wgpu::DeviceType::IntegratedGpu|wgpu::DeviceType::DiscreteGpu),"BLOCKER: not admitted hardware adapter");
        assert_eq!(info.backend,wgpu::Backend::Metal);
        let format = adapter.get_texture_format_features(wgpu::TextureFormat::Rgba8UnormSrgb);
        assert!(format.allowed_usages.contains(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC));
        let (device,queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor { label:Some("throwaway-proof-only"),required_features:wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,required_limits:wgpu::Limits::default(),..Default::default() })).expect("BLOCKER: device request failed");
        event("adapter",json!({"info":format!("{info:#?}"),"adapter_features":format!("{:?}",adapter.features()),"adapter_limits":format!("{:#?}",adapter.limits()),"device_features":format!("{:?}",device.features()),"device_limits":format!("{:#?}",device.limits()),"format":format!("{format:?}"),"surface":false,"native_window":false,"backend":"Metal","manual":true}));
        let creation = RenderCreation::manual(RenderDevice::from(device),RenderQueue::new(queue),RenderAdapterInfo::new(info),RenderAdapter::new(adapter),RenderInstance::new(instance));
        let mut app = App::new();
        app.add_plugins(DefaultPlugins
            .set(bevy::log::LogPlugin {custom_layer:backend_layer,..default()})
            .set(WindowPlugin {primary_window:None,exit_condition:ExitCondition::DontExit,..default()})
            .set(RenderPlugin {render_creation:creation,synchronous_pipeline_compilation:true,..default()})
            .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>());
        // Manual creation must be immediately ready; no App.update warm-up.
        assert_eq!(app.plugins_state(), bevy::app::PluginsState::Ready);
        #[cfg(feature="product-control")]
        if product {app.add_plugins(woodpecker::session::Plugin);}
        #[cfg(not(feature="product-control"))]
        assert!(!product,"product-control feature was not built");
        app.add_plugins(bevy::render::extract_component::ExtractComponentPlugin::<Tracked>::default());
        app.finish();
        app.cleanup();
        app.init_resource::<Counters>().init_resource::<Fatal>().add_message::<PendingMessage>()
            .insert_resource(RenderErrorHandler(render_error))
            .insert_resource(ClearColor(Color::BLACK))
            .add_systems(Startup,startup).add_systems(Update,simulation).add_systems(FixedUpdate,fixed);
        let mut image = Image::new_target_texture(W,H,wgpu::TextureFormat::Rgba8UnormSrgb,None);
        image.texture_descriptor.usage |= wgpu::TextureUsages::COPY_SRC;
        let image = app.world_mut().resource_mut::<Assets<Image>>().add(image);
        let camera = app.world_mut().spawn((Camera2d,Camera {clear_color:ClearColorConfig::Custom(Color::BLACK),..default()},Projection::Orthographic(OrthographicProjection {scaling_mode:ScalingMode::Fixed {width:W as f32,height:H as f32},..OrthographicProjection::default_2d()}),RenderTarget::Image(image.clone().into()),Tonemapping::None,Msaa::Off)).id();
        let rect = app.world_mut().resource_mut::<Assets<Mesh>>().add(Rectangle::new(20.0,16.0));
        let material = app.world_mut().resource_mut::<Assets<ColorMaterial>>().add(Color::srgb_u8(255,0,0));
        let moving = app.world_mut().spawn((Mesh2d(rect),MeshMaterial2d(material.clone()),Transform::from_xyz(-12.0,0.0,0.0),Tracked(0))).id();
        // Asymmetric world geometry: verifies orientation, stride and channels.
        for (x,y,color) in [(-26.0,18.0,[0,0,255]),(26.0,18.0,[255,255,0]),(-26.0,-18.0,[0,255,255]),(26.0,-18.0,[255,0,255])] {
            let mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(Rectangle::new(8.0,8.0));
            let mat = app.world_mut().resource_mut::<Assets<ColorMaterial>>().add(Color::srgb_u8(color[0],color[1],color[2]));
            app.world_mut().spawn((Mesh2d(mesh),MeshMaterial2d(mat),Transform::from_xyz(x,y,0.0)));
        }
        let sentinel = app.world_mut().spawn(Sentinel).id();
        app.insert_resource(Scene {image:image.clone(),moving,material,sentinel});
        let render = app.sub_app_mut(RenderApp);
        render.insert_resource(TargetImage(image)).init_resource::<Request>().init_resource::<Copies>()
            .init_resource::<RenderObservation>()
            .init_resource::<ChainCounters>()
            .add_systems(ExtractSchedule,observe_image_notifications)
            .add_systems(Render,observe_render_invocation.before(bevy::render::RenderSystems::ExtractCommands))
            .add_systems(bevy::render::RenderStartup,observe_render_startup)
            .add_systems(ExtractSchedule,observe_extraction)
            .add_systems(RenderGraph,copy_final.after(RenderGraphSystems::Submit).before(RenderGraphSystems::Finish));
        for label in [Render.intern(),RenderGraph.intern(),ExtractSchedule.intern()] {
            render.get_schedule_mut(label).unwrap().set_executor(SingleThreadedExecutor::new());
        }
        // Run only explicit startup schedules. Normal main schedules are reserved for tick().
        let startup_labels = app.world().resource::<MainScheduleOrder>().startup_labels.clone();
        for label in startup_labels {let _=app.world_mut().try_run_schedule(label);}
        event("startup",json!({"camera":format!("{camera:?}"),"mesh_count":5,"camera_count":1,"image":[W,H],"format":"Rgba8UnormSrgb","pipelined":false,"winit_compiled":false,"clipboard_compiled":false}));
        Self {app,pumps:0,out,started:Instant::now(),forwarded_shaders:HashSet::new()}
    }
    fn device(&self) -> wgpu::Device {self.app.world().resource::<RenderDevice>().wgpu_device().clone()}
    fn tick(&mut self) {
        let world = self.app.world_mut();
        if world.resource::<Time<Real>>().last_update().is_none() {world.resource_mut::<Time<Real>>().update_with_duration(Duration::ZERO);}
        world.insert_resource(TimeUpdateStrategy::ManualDuration(DT));
        let labels = world.resource::<MainScheduleOrder>().labels.clone();
        for label in labels {let _=world.try_run_schedule(label);}
        world.resource_mut::<Counters>().ticks += 1;
        event("explicit_tick",self.snapshot());
    }
    fn snapshot(&self) -> Value {
        let w = self.app.world(); let c=w.resource::<Counters>();
        let s=w.resource::<Scene>();
        let marker=w.entity(s.sentinel).get_ref::<Sentinel>();
        json!({"ticks":c.ticks,"updates":c.updates,"fixed":c.fixed,"startup":c.startup,"pending_input":c.pending,"consumed_input":c.consumed,"real_ns":w.resource::<Time<Real>>().elapsed().as_nanos() as u64,"virtual_ns":w.resource::<Time<Virtual>>().elapsed().as_nanos() as u64,"fixed_ns":w.resource::<Time<Fixed>>().elapsed().as_nanos() as u64,"real_delta_ns":w.resource::<Time<Real>>().delta().as_nanos() as u64,"marker_changed":marker.as_ref().map(|m|m.last_changed().get()),"removed":w.removed::<Sentinel>().map(|e|format!("{e:?}")).collect::<Vec<_>>(),"pending_messages":w.resource::<Messages<PendingMessage>>().len(),"moving_x":w.get::<Transform>(s.moving).unwrap().translation.x,"material":format!("{:?}",w.resource::<Assets<ColorMaterial>>().get(&s.material).unwrap().color)})
    }
    fn seed_idle_oracles(&mut self) {
        self.app.world_mut().resource_mut::<Counters>().pending = 7;
        self.app.world_mut().write_message(PendingMessage);
        let removed=self.app.world_mut().spawn(Sentinel).id();
        self.app.world_mut().entity_mut(removed).remove::<Sentinel>();
    }
    fn asset_and_clock_pump(&mut self) {
        // Explicit probe-only CPU asset progress. No First/PreUpdate/PostUpdate or Time update.
        // The normal private asset_events system cannot be called independently. For this
        // fixed shader-only loading scope, forward Added for genuinely loaded new shaders
        // through the public AssetEvent stream; this does NOT qualify product asset lifecycles.
        bevy::tasks::tick_global_task_pools_on_main_thread();
        bevy::asset::handle_internal_asset_events(self.app.world_mut());
        let ids: Vec<_> = self.app.world().resource::<Assets<Shader>>().ids().collect();
        let added: Vec<_> = ids.iter().copied().filter(|id| self.forwarded_shaders.insert(*id)).map(|id| AssetEvent::<Shader>::Added {id}).collect();
        let count=added.len();
        self.app.world_mut().write_message_batch(added);
        let drained=self.app.world().resource::<bevy::time::TimeReceiver>().0.try_iter().count();
        event("explicit_asset_and_clock_pump",json!({"loaded_shaders":ids.len(),"forwarded_new_shaders":count,"clock_samples_drained":drained,"simulation_schedules":0,"time_updates":0}));
    }
    fn extract_only(&mut self) {
        event("normal_extract_enter",json!({"snapshot":self.snapshot()}));
        let subs=self.app.sub_apps_mut();
        subs.sub_apps.get_mut(&RenderApp.intern()).unwrap().extract(subs.main.world_mut());
        event("normal_extract_return",json!({"fatal":self.app.world().resource::<Fatal>().first,"render":self.render_observation()}));
    }
    fn pump(&mut self, request:u64, state:u64, generation:u64) -> Option<Candidate> {
        assert!(self.started.elapsed()<Duration::from_secs(110),"internal case budget");
        assert!(self.pumps<MAX_PUMPS,"bounded renderpump exhausted"); self.pumps+=1;
        let tick=self.app.world().resource::<Counters>().ticks;
        // Retains the product's pre-first-tick suppression. No hidden preparation/extraction.
        if tick==0 {event("capture_rejected_before_tick",json!({"request":request,"pumps":self.pumps}));return None;}
        self.asset_and_clock_pump();
        let token=Token {request,generation,frame:self.pumps,tick,state};
        self.app.sub_app_mut(RenderApp).world_mut().resource_mut::<Request>().0=Some(token);
        let device=self.device();
        let scope=device.push_error_scope(wgpu::ErrorFilter::Validation);
        self.extract_only();
        if self.app.world().resource::<Fatal>().first.is_some() {drop(scope);return None;}
        self.app.sub_app_mut(RenderApp).run_default_schedule();
        let error=pollster::block_on(scope.pop());
        assert!(error.is_none(),"draw/copy scope error: {error:?}");
        let copies=std::mem::take(&mut self.app.sub_app_mut(RenderApp).world_mut().resource_mut::<Copies>().0);
        assert!(copies.len()<=1);
        let copy=copies.into_iter().next()?;
        let (tx,rx)=mpsc::channel();
        let map_token=copy.token.clone();
        copy.buffer.slice(..).map_async(wgpu::MapMode::Read,move |r|{event("map_callback",json!({"token":map_token.json(),"result":format!("{r:?}")}));let _=tx.send(r);});
        let poll=device.poll(wgpu::PollType::Wait {submission_index:Some(copy.submission.clone()),timeout:Some(Duration::from_secs(10))});
        event("bounded_device_poll",json!({"result":format!("{poll:?}")}));
        poll.expect("device poll failed");
        rx.recv_timeout(Duration::from_secs(10)).expect("map callback deadline").expect("map failed");
        let mapped=copy.buffer.slice(..).get_mapped_range().expect("mapped range failed");
        assert_eq!(mapped.len(),(STRIDE*H) as usize);
        let mut bytes=Vec::with_capacity((W*H*4) as usize);
        for row in mapped.chunks_exact(STRIDE as usize) {bytes.extend_from_slice(&row[..(W*4) as usize]);}
        drop(mapped);copy.buffer.unmap();
        assert_eq!(bytes.len(),(W*H*4) as usize);
        let done=copy.queue_done.load(Ordering::SeqCst);
        event("mapped_candidate_NOT_receipt",json!({"token":copy.token.json(),"queue_callback":done,"sha256":hash(&bytes),"bytes":bytes.len(),"publisher_cut":"UNPROVEN"}));
        Some(Candidate {token:copy.token,target:copy.target,submission:format!("{:?}",copy.submission),bytes,queue_done:done})
    }
    fn capture(&mut self,request:u64,state:u64,generation:u64,name:&str) -> Candidate {
        let before=self.snapshot();
        loop {
            if let Some(candidate)=self.pump(request,state,generation) {
                if !self.out.join("first-candidate.png").exists() {save(&self.out.join("first-candidate.png"),&candidate.bytes);}
                if pixel_oracle(&candidate.bytes,state) {
                    assert_eq!(before,self.snapshot(),"idle simulation/lifecycle oracle changed");
                    assert!(candidate.queue_done,"queue callback missing; not sufficient for success");
                    save(&self.out.join(name),&candidate.bytes);
                    event("image_and_idle_oracles_pass",json!({"token":candidate.token.json(),"snapshot":self.snapshot(),"file":name,"product_receipt":false}));
                    return candidate;
                }
                event("image_not_yet_expected",json!({"token":candidate.token.json()}));
            }
            assert!(self.app.world().resource::<Fatal>().first.is_none(),"fatal during capture");
        }
    }
}

// Fixed before execution: interior samples with sRGB byte tolerance <=2.
// Full geometry checked away from rasterization edges, not just 'non-black'.
fn expected(x:u32,y:u32,state:u64)->[u8;3] {
    let wx=x as f32+0.5-W as f32/2.0;
    let wy=H as f32/2.0-(y as f32+0.5);
    for (cx,cy,c) in [(-26.0,18.0,[0,0,255]),(26.0,18.0,[255,255,0]),(-26.0,-18.0,[0,255,255]),(26.0,-18.0,[255,0,255])] {
        if (wx-cx).abs()<4.0 && (wy-cy).abs()<4.0{return c;}
    }
    let cx=if state==1 {-12.0}else{12.0};
    if (wx-cx).abs()<10.0 && wy.abs()<8.0 {if state==1 {[255,0,0]}else{[0,255,0]}}else{[0,0,0]}
}
fn pixel_oracle(bytes:&[u8],state:u64)->bool {
    if bytes.len()!=(W*H*4) as usize{return false;}
    let mut checked=0;
    for y in 0..H {for x in 0..W {
        // Skip a fixed one-pixel edge band by comparing the expected 3x3 neighbourhood.
        if x==0 || y==0 || x+1==W || y+1==H {continue;}
        let e=expected(x,y,state);
        if (y-1..=y+1).any(|yy|(x-1..=x+1).any(|xx|expected(xx,yy,state)!=e)){continue;}
        let p=&bytes[((y*W+x)*4) as usize..][..4];
        if p[3]!=255 || (0..3).any(|i|p[i].abs_diff(e[i])>2){return false;}
        checked+=1;
    }}
    checked>2000
}
fn save(path:&Path,bytes:&[u8]) {
    // PNG is diagnostic RGBA8, unlike the product's RGB8 contract. Alpha is verified opaque.
    image::save_buffer_with_format(path,bytes,W,H,image::ColorType::Rgba8,image::ImageFormat::Png).unwrap();
    let decoded=image::open(path).unwrap().to_rgba8();
    assert_eq!(decoded.dimensions(),(W,H));assert_eq!(decoded.as_raw(),bytes);
    event("png_roundtrip",json!({"path":path,"sha256":hash(&std::fs::read(path).unwrap()),"pixel_sha256":hash(bytes)}));
}
fn invalid_shader(device:&wgpu::Device) {
    let _=device.create_shader_module(wgpu::ShaderModuleDescriptor {label:Some("INTENTIONALLY_INVALID_PROOF_SHADER"),source:wgpu::ShaderSource::Wgsl(Cow::Borrowed("@vertex fn invalid( -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); }"))});
}
// Small probe-local candidate arbiter. NOT the proposed product Publisher-Cut.
fn admit(candidate:&Candidate,active:&Token,min_frame:u64,consumed:bool,fatal:bool)->Result<(),&'static str> {
    if fatal{return Err("fatal");}
    if consumed{return Err("duplicate");}
    let t=&candidate.token;
    if t.request!=active.request{return Err("request");}
    if t.generation!=active.generation{return Err("generation");}
    if t.frame<min_frame{return Err("stale_frame");}
    if t.tick!=active.tick || t.state!=active.state{return Err("tick_state");}
    if !pixel_oracle(&candidate.bytes,active.state){return Err("pixels");}
    Ok(())
}
include!("study_base.rs");
include!("publication.rs");
include!("cases.rs");
