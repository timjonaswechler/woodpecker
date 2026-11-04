// Fixed third-study cases; every process drives the real public product Warp once.
use bevy::render::{Extract, sync_world::MainEntity};

static TRACKED_CLONES:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(0);
#[derive(Component,bevy::render::extract_component::ExtractComponent)]
#[extract_app(RenderApp)]
struct Tracked(u64);
impl Clone for Tracked {
    fn clone(&self)->Self {TRACKED_CLONES.fetch_add(1,Ordering::SeqCst);Self(self.0)}
}
#[derive(Resource,Default)]
struct ChainCounters { original:u64, explicit_hook:u64, renders:u64, image_modified:u64 }
fn observe_render_invocation(mut c:ResMut<ChainCounters>) {c.renders+=1;}
fn observe_image_notifications(mut events:Extract<MessageReader<AssetEvent<Image>>>,scene:Extract<Res<Scene>>,mut c:ResMut<ChainCounters>) {
    for e in events.read() {
        if matches!(e,AssetEvent::Modified{id} if *id==scene.image.id()) {c.image_modified+=1;}
    }
}
impl Probe {
    fn install_chain_wrapper(&mut self,guarded:bool) {
        let hook=self.public_pre_extract();
        let mut original=self.app.sub_app_mut(RenderApp).take_extract().expect("real product extract gate");
        self.app.sub_app_mut(RenderApp).set_extract(move |main,render| {
            if main.resource::<Counters>().ticks==0 {original(main,render);return;}
            if guarded {
                render.resource_mut::<ChainCounters>().explicit_hook+=1;
                event("chain_explicit_hook_enter",json!({}));hook(main,render);
                if main.resource::<Fatal>().first.is_some() {event("chain_fatal_shortcircuit",json!({}));return;}
            }
            render.resource_mut::<ChainCounters>().original+=1;
            event("chain_original_extract_enter",json!({}));original(main,render);
        });
        event("chain_wrapper_configuration",json!({"guarded":guarded,"original_product_gate_preserved":true}));
    }
    fn chain(&mut self)->Value {
        let moving=self.app.world().resource::<Scene>().moving;
        let world=self.app.sub_app_mut(RenderApp).world_mut();
        let mut query=world.query::<(Entity,&MainEntity,&Tracked)>();
        let entities:Vec<_>=query.iter(world).map(|(e,m,t)|(format!("{e:?}"),format!("{:?}",m.id()),t.0)).collect();
        assert_eq!(entities.len(),1,"one synchronized actual mesh entity");
        assert_eq!(entities[0].1,format!("{moving:?}"));
        let c=world.resource::<ChainCounters>();let r=world.resource::<RenderObservation>();
        json!({"original":c.original,"explicit_hook":c.explicit_hook,"renders":c.renders,"image_modified":c.image_modified,
            "startup":r.startup,"extracts":r.extractions,"tracked_clones":TRACKED_CLONES.load(Ordering::SeqCst),"entities":entities})
    }
    fn measured_healthy_window(&mut self,guarded:bool) {
        let before=self.snapshot();let start=self.chain();
        for n in 1..=3 {
            let (id,moving)={let s=self.app.world().resource::<Scene>();(s.image.id(),s.moving)};
            self.app.world_mut().entity_mut(moving).insert(Tracked(n));
            // Controlled public notification for an existing real GPU Image, not a forged PNG.
            self.app.world_mut().write_message(AssetEvent::<Image>::Modified{id});
            let c=self.pump(710+n,1,1).expect("known ready camera output");
            assert!(pixel_oracle(&c.bytes,1));let counts=self.chain();
            assert_eq!(counts["entities"][0][2],n);
            assert_eq!(counts["startup"],1);
        }
        let end=self.chain();
        for key in ["original","renders","extracts","tracked_clones","image_modified"] {
            assert_eq!(end[key].as_u64().unwrap()-start[key].as_u64().unwrap(),3,"exactly one operation per measured pump: {key}");
        }
        assert_eq!(end["explicit_hook"].as_u64().unwrap()-start["explicit_hook"].as_u64().unwrap(),if guarded {3}else{0});
        self.assert_paused(&before);
        event("healthy_window_verified",json!({"guarded":guarded,"before":start,"after":end,"simulation":self.snapshot(),"implicit_original_pre_hook_count":"one per original callback by pinned Bevy source; not privately instrumented"}));
    }
    fn lease(&self,request:u64,generation:u64,min_frame:u64)->Lease {
        Lease {request,generation,min_frame,tick:1,state:1,target:self.app.world().resource::<Scene>().image.id(),deadline:Instant::now()+Duration::from_secs(30)}
    }
    fn target_texture(&self)->bevy::render::render_resource::Texture {
        let image=&self.app.world().resource::<Scene>().image;
        self.app.sub_app(RenderApp).world().resource::<RenderAssets<GpuImage>>().get(image).unwrap().texture.clone()
    }
}
fn checked_scope_error(scope:wgpu::ErrorScopeGuard,label:&str)->String {
    let error=pollster::block_on(scope.pop()).expect("intentionally invalid real GPU API operation must fail");
    assert!(matches!(error,wgpu::Error::Validation{..}));
    let description=error.to_string();
    event("actual_output_validation",json!({"operation":label,"error":description,"source":"real wgpu Validation, not CPU boolean injection"}));description
}
fn pipeline_validation(p:&Probe) {
    let device=p.device();
    let module_scope=device.push_error_scope(wgpu::ErrorFilter::Validation);
    let module=device.create_shader_module(wgpu::ShaderModuleDescriptor {label:Some("VALID_WGSL_OUTPUT_INTERFACE_TEST"),source:wgpu::ShaderSource::Wgsl(Cow::Borrowed(
        "@vertex fn vs(@builtin(vertex_index) i:u32)->@builtin(position) vec4<f32>{ var p=array<vec2<f32>,3>(vec2<f32>(0.0,0.5),vec2<f32>(-0.5,-0.5),vec2<f32>(0.5,-0.5));return vec4<f32>(p[i],0.0,1.0); } @fragment fn fs()->@location(0) vec4<u32>{return vec4<u32>(1u,0u,0u,1u);}"
    ))});
    assert!(pollster::block_on(module_scope.pop()).is_none(),"shader module must be VALID");
    event("valid_shader_module_accepted",json!({"fragment_output":"vec4<u32>","pipeline_target":"Rgba8UnormSrgb"}));
    let scope=device.push_error_scope(wgpu::ErrorFilter::Validation);
    let targets=[Some(wgpu::ColorTargetState {format:wgpu::TextureFormat::Rgba8UnormSrgb,blend:None,write_mask:wgpu::ColorWrites::ALL})];
    let _=device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label:Some("INTENTIONALLY_INCOMPATIBLE_PIPELINE_TARGET"),layout:None,
        vertex:wgpu::VertexState {module:&module,entry_point:Some("vs"),compilation_options:Default::default(),buffers:&[]},
        fragment:Some(wgpu::FragmentState {module:&module,entry_point:Some("fs"),compilation_options:Default::default(),targets:&targets}),
        primitive:Default::default(),depth_stencil:None,multisample:Default::default(),multiview_mask:None,cache:None,
    });
    checked_scope_error(scope,"create_render_pipeline / valid module incompatible color target");
}
fn draw_submit_validation(p:&Probe) {
    let device=p.device();let scope=device.push_error_scope(wgpu::ErrorFilter::Validation);
    let texture=device.create_texture(&wgpu::TextureDescriptor {label:Some("own-invalid-draw-target"),size:wgpu::Extent3d{width:4,height:4,depth_or_array_layers:1},mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,format:wgpu::TextureFormat::Rgba8Unorm,usage:wgpu::TextureUsages::RENDER_ATTACHMENT,view_formats:&[]});
    let view=texture.create_view(&Default::default());let mut encoder=device.create_command_encoder(&Default::default());
    {
        let attachments=[Some(wgpu::RenderPassColorAttachment {view:&view,depth_slice:None,resolve_target:None,ops:wgpu::Operations{load:wgpu::LoadOp::Clear(wgpu::Color::BLACK),store:wgpu::StoreOp::Store}})];
        let mut pass=encoder.begin_render_pass(&wgpu::RenderPassDescriptor {label:Some("INTENTIONALLY_DRAW_WITHOUT_PIPELINE"),color_attachments:&attachments,depth_stencil_attachment:None,timestamp_writes:None,occlusion_query_set:None,multiview_mask:None});
        pass.draw(0..3,0..1);
    }
    p.app.world().resource::<RenderQueue>().submit([encoder.finish()]);
    p.poll_bounded();checked_scope_error(scope,"draw without pipeline / encoder finish / submit");
}
fn copy_to_buffer(p:&Probe,buffer:&wgpu::Buffer)->wgpu::SubmissionIndex {
    let texture=p.target_texture();let mut encoder=p.device().create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(texture.as_image_copy(),wgpu::TexelCopyBufferInfo{buffer,layout:wgpu::TexelCopyBufferLayout{offset:0,bytes_per_row:Some(STRIDE),rows_per_image:Some(H)}},wgpu::Extent3d{width:W,height:H,depth_or_array_layers:1});
    p.app.world().resource::<RenderQueue>().submit([encoder.finish()])
}
fn copy_validation(p:&Probe) {
    let device=p.device();let scope=device.push_error_scope(wgpu::ErrorFilter::Validation);
    let buffer=device.create_buffer(&wgpu::BufferDescriptor{label:Some("INTENTIONALLY_TOO_SMALL_COPY_DESTINATION"),size:16,usage:wgpu::BufferUsages::MAP_READ|wgpu::BufferUsages::COPY_DST,mapped_at_creation:false});
    let _=copy_to_buffer(p,&buffer);p.poll_bounded();checked_scope_error(scope,"copy actual rendered target into undersized buffer");
}
fn map_destroyed_buffer(p:&Probe) {
    let device=p.device();let scope=device.push_error_scope(wgpu::ErrorFilter::Validation);
    let buffer=device.create_buffer(&wgpu::BufferDescriptor{label:Some("own-readback-to-destroy"),size:u64::from(STRIDE*H),usage:wgpu::BufferUsages::MAP_READ|wgpu::BufferUsages::COPY_DST,mapped_at_creation:false});
    let submission=copy_to_buffer(p,&buffer);
    device.poll(wgpu::PollType::Wait{submission_index:Some(submission),timeout:Some(Duration::from_secs(10))}).unwrap();
    event("map_fault_barrier",json!({"actual_scene_copy_completed":true,"operation":"destroy own staging buffer before map"}));
    buffer.destroy();let(tx,rx)=mpsc::channel();
    buffer.slice(..).map_async(wgpu::MapMode::Read,move|r|{event("actual_failed_map_callback",json!({"result":format!("{r:?}")}));let _=tx.send(r);});
    p.poll_bounded();let mapped=rx.recv_timeout(Duration::from_secs(5)).expect("bounded failed map callback");
    assert!(mapped.is_err(),"destroyed real buffer must not map successfully");
    let error=pollster::block_on(scope.pop());
    event("actual_map_failure_verified",json!({"result":format!("{mapped:?}"),"scoped_error":error.map(|e|e.to_string()),"cpu_fault_injection":false}));
}

pub fn publication_main() {
    let mut args=std::env::args().skip(1);let case=args.next().expect("fixed case");let out=PathBuf::from(args.next().expect("new case root"));assert!(args.next().is_none());
    event("publication_study_begin",json!({"case":case,"pid":std::process::id(),"max_pumps":64,"limit_seconds":120}));
    let mut p=Probe::new_mode(out,true);let guarded=case!="healthy_original";
    p.install_chain_wrapper(guarded);product_one_tick(&mut p);p.seed_idle_oracles();
    let initial=p.capture(700,1,1,"diagnostic-A.png");let paused=p.snapshot();
    assert!(pixel_oracle(&initial.bytes,1));assert_eq!(p.render_observation()["startup"],1);
    if case=="healthy_original" || case=="healthy_wrapper" {p.measured_healthy_window(guarded);}
    else if ["pipeline_validation","draw_submit_validation","copy_validation","map_destroyed_buffer"].contains(&case.as_str()) {
        let mut owner=Publisher::new(&p.out,p.lease(800,1,p.pumps+1));
        match case.as_str(){"pipeline_validation"=>pipeline_validation(&p),"draw_submit_validation"=>draw_submit_validation(&p),"copy_validation"=>copy_validation(&p),_=>map_destroyed_buffer(&p)}
        owner.abort("actual_output_chain_error");owner.assert_no_success();
        assert!(owner.stage(&initial,None,false).is_err());
        let after=p.capture(900,1,1,"healthy-after-api-error.png");assert_eq!(initial.bytes,after.bytes);
        assert!(p.app.world().resource::<Fatal>().first.is_none());
        event("output_error_and_healthy_control_verified",json!({"case":case,"request_failed":800,"healthy_control_request":900}));
    } else if case=="stale_request_generation" {
        let old_request=p.capture(800,1,2,"held-old-request.png");
        let old_generation=p.capture(801,1,1,"held-old-generation.png");
        let mut owner=Publisher::new(&p.out,p.lease(801,2,p.pumps+1));
        let fresh=p.capture(801,1,2,"fresh-current.png");
        assert_eq!(owner.stage(&old_request,None,false),Err("request"));
        assert_eq!(owner.stage(&old_generation,None,false),Err("generation"));
        p.observe_cut(true);owner.stage(&fresh,p.app.world().resource::<Fatal>().first.as_deref(),false).unwrap();
        owner.commit(p.app.world().resource::<Fatal>().first.as_deref()).unwrap();
    } else if case=="stale_frame" {
        let old=p.capture(800,1,1,"held-old-frame.png");
        let mut owner=Publisher::new(&p.out,p.lease(800,1,old.token.frame+1));
        assert_eq!(owner.stage(&old,None,false),Err("frame"));
        let fresh=p.capture(800,1,1,"fresh-current.png");p.observe_cut(true);
        owner.stage(&fresh,None,false).unwrap();owner.commit(None).unwrap();
    } else {
        let mut lease=p.lease(800,1,p.pumps+1);
        if case=="expired_deadline" {lease.deadline=Instant::now();}
        let mut owner=Publisher::new(&p.out,lease);
        let candidate=p.capture(800,1,1,"request-candidate.png");
        p.observe_cut(true);
        match case.as_str() {
            "publisher_healthy"=>{
                owner.stage(&candidate,None,false).unwrap();let r=owner.commit(None).unwrap();
                assert_eq!(r["frame"],candidate.token.frame);assert_eq!(owner.commit(None),Err("already_published"));
                assert_eq!(owner.stage(&candidate,None,false),Err("already_published"));assert_eq!(owner.successes,1);
            }
            "cancel_after_map"=>{owner.abort("cancelled");assert_eq!(owner.stage(&candidate,None,false),Err("cancelled"));owner.assert_no_success();}
            "expired_deadline"=>{assert_eq!(owner.stage(&candidate,None,false),Err("timeout"));owner.assert_no_success();}
            "partial_write_injection"=>{assert_eq!(owner.stage(&candidate,None,true),Err("injected_partial_write"));assert_eq!(std::fs::metadata(owner.staging.join("frame.png")).unwrap().len(),8);owner.assert_no_success();}
            "observed_device_fatal_before_commit"=>{
                owner.stage(&candidate,None,false).unwrap();p.inject_destroy();p.observe_cut(true);
                let fatal=p.app.world().resource::<Fatal>().first.clone();assert!(fatal.as_ref().is_some_and(|e|e.starts_with("DeviceLost")));
                assert_eq!(owner.commit(fatal.as_deref()),Err("observed_fatal"));owner.assert_no_success();
            }
            "cancel_after_stage"=>{owner.stage(&candidate,None,false).unwrap();owner.abort("cancelled");assert_eq!(owner.commit(None),Err("cancelled"));owner.assert_no_success();}
            "rename_io_error"=>{
                owner.stage(&candidate,None,false).unwrap();std::fs::write(&owner.destination,b"OWN_SENTINEL_DO_NOT_REPLACE").unwrap();
                assert_eq!(owner.commit(None),Err("rename_io"));assert_eq!(std::fs::read(&owner.destination).unwrap(),b"OWN_SENTINEL_DO_NOT_REPLACE");owner.assert_no_success();
            }
            _=>panic!("unknown fixed publication case"),
        }
    }
    product_idle_control(&mut p);p.assert_paused(&paused);
    event("publication_case_complete",json!({"case":case,"snapshot":p.snapshot(),"render":p.render_observation(),"chain":p.chain(),"pumps":p.pumps,"product_capture_implemented":false,"general_headless_PASS":false}));
}
