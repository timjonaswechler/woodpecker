// New study code included in our copied fixture, not in the frozen original.
static TRACE_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
static TRACE_ORIGIN: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
static TRACE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[derive(Resource, Default)]
struct RenderObservation { startup: u64, extractions: u64 }
fn observe_render_startup(mut r: ResMut<RenderObservation>) {
    r.startup += 1;
    event("render_startup_observed", json!({"count":r.startup}));
}
fn observe_extraction(mut r: ResMut<RenderObservation>) {
    r.extractions += 1;
    event("extract_schedule_observed", json!({"count":r.extractions}));
}

struct BackendLayer;
struct LogMessage(String);
impl tracing::field::Visit for LogMessage {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name()=="message" {self.0=format!("{value:?}");}
    }
}
impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for BackendLayer {
    fn on_event(&self, e:&tracing::Event<'_>, _:tracing_subscriber::layer::Context<'_,S>) {
        if e.metadata().target()=="bevy_render::error_handler" {
            let mut message=LogMessage(String::new()); e.record(&mut message);
            // Pinned Bevy source logs BEFORE storing in its callback latch. Name accordingly.
            event("backend_error_callback_log",json!({"message":message.0,"source_order":"log_then_latch"}));
        }
    }
}
fn backend_layer(_: &mut App) -> Option<bevy::log::BoxedLayer> {Some(Box::new(BackendLayer))}

type PublicPreExtract = fn(&mut World, &mut World);
impl Probe {
    fn render_observation(&self) -> Value {
        let world=self.app.sub_app(RenderApp).world();
        let r=world.resource::<RenderObservation>();
        json!({"startup":r.startup,"extractions":r.extractions,"gpu_preprocessing_resource":world.contains_resource::<bevy::render::batching::gpu_preprocessing::GpuPreprocessingSupport>()})
    }
    fn public_pre_extract(&self) -> PublicPreExtract {
        let plugins=self.app.get_added_plugins::<bevy::render::extract_plugin::ExtractPlugin<RenderApp>>();
        assert_eq!(plugins.len(),1,"exact installed renderer extract plugin required");
        plugins[0].pre_extract
    }
    fn call_public_pre_extract(&mut self) {
        let hook=self.public_pre_extract();
        event("public_pre_extract_enter",json!({"snapshot":self.snapshot(),"render":self.render_observation()}));
        let subs=self.app.sub_apps_mut();
        let render=subs.sub_apps.get_mut(&RenderApp.intern()).unwrap();
        hook(subs.main.world_mut(),render.world_mut());
        event("public_pre_extract_return",json!({"fatal":self.app.world().resource::<Fatal>().first,"render":self.render_observation()}));
    }
    fn install_public_fatal_guard(&mut self) {
        let hook=self.public_pre_extract();
        let render=self.app.sub_app_mut(RenderApp);
        let mut original=render.take_extract().expect("normal extract callback");
        render.set_extract(move |main,render| {
            if main.resource::<Counters>().ticks==0 {return;}
            hook(main,render);
            if main.resource::<Fatal>().first.is_some() {
                event("public_fatal_shortcircuit",json!({"extract_schedule_skipped":true}));
                return;
            }
            original(main,render);
        });
        event("installed_public_fatal_guard",json!({"private_symbol_access":false,"full_extraction_replaced":false,"pre_tick_gate_retained":true}));
    }
    fn poll_bounded(&self) {
        event("poll_enter",json!({}));
        let result=self.device().poll(wgpu::PollType::Wait {submission_index:None,timeout:Some(Duration::from_secs(10))});
        event("poll_return",json!({"result":format!("{result:?}")}));
        result.expect("bounded device poll");
    }
    fn inject_uncaptured(&self) {
        event("fault_api_enter",json!({"kind":"uncaptured_validation"}));
        invalid_shader(&self.device());
        event("fault_api_return",json!({"kind":"uncaptured_validation","handler_calls":self.app.world().resource::<Fatal>().calls}));
    }
    fn inject_destroy(&self) {
        event("fault_api_enter",json!({"kind":"own_device_destroy"}));
        self.device().destroy();
        event("fault_api_return",json!({"kind":"own_device_destroy","hardware_or_driver_reset":false,"handler_calls":self.app.world().resource::<Fatal>().calls}));
        let queue=self.app.world().resource::<RenderQueue>().clone();
        queue.on_submitted_work_done(||event("queue_callback_after_destroy",json!({"success_status":false})));
    }
    fn assert_paused(&self,before:&Value) {
        assert_eq!(*before,self.snapshot(),"simulation/input oracle changed while paused");
        assert_eq!(self.snapshot()["ticks"],1);
        assert_eq!(self.snapshot()["updates"],1);
        assert_eq!(self.snapshot()["real_ns"],16_666_667);
        event("paused_oracles_unchanged",json!({"snapshot":self.snapshot(),"render":self.render_observation()}));
    }
    fn healthy_one_tick(&mut self) -> Candidate {
        self.tick();self.seed_idle_oracles();
        let a=self.capture(401,1,1,"healthy-A.png");
        assert_eq!(self.render_observation()["startup"],1);
        assert_eq!(self.render_observation()["gpu_preprocessing_resource"],true);
        a
    }
    fn observe_cut(&mut self,hook_only:bool) {
        event("observation_cut_enter",json!({"hook_only":hook_only}));
        self.poll_bounded();
        let drained=self.app.world().resource::<bevy::time::TimeReceiver>().0.try_iter().count();
        event("clock_channel_drained_without_tick",json!({"count":drained}));
        if hook_only {self.call_public_pre_extract();}else{self.extract_only();}
        event("observation_cut_return",json!({"fatal":self.app.world().resource::<Fatal>().first}));
    }
    fn publish_candidate(&mut self,a:&Candidate,hook_only:bool) -> bool {
        let before=self.snapshot();
        self.observe_cut(hook_only);
        self.assert_paused(&before);
        let fatal=self.app.world().resource::<Fatal>().first.clone();
        let accepted=fatal.is_none() && a.queue_done && pixel_oracle(&a.bytes,1) && a.token.tick==1;
        event("publication_decision",json!({"accepted":accepted,"token":a.token.json(),"fatal":fatal,"semantics":"verified_completed_frame_and_no_observed_fatal_at_cut; not future_device_health"}));
        if accepted {
            let receipt=json!({"request":a.token.request,"frame":a.token.frame,"tick":a.token.tick,"pixels_sha256":hash(&a.bytes),"scope":"probe-only verified finite frame; observed fatal gate"});
            std::fs::write(self.out.join("PROBE_RECEIPT.json"),serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
            event("probe_receipt_written",receipt);
        }
        accepted
    }
}

#[cfg(feature="product-control")]
fn product_one_tick(p:&mut Probe) {
    assert_eq!(p.snapshot()["updates"],0);
    let deadline=Instant::now()+Duration::from_secs(5);
    let labels=p.app.world().resource::<MainScheduleOrder>().labels.clone();
    assert_eq!(labels.len(),1,"actual product order must contain Control only");
    event("product_control_begin",json!({"labels":format!("{labels:?}"),"wire_command":"tick.warp.start ticks=1"}));
    while p.app.world().resource::<Counters>().updates==0 {
        assert!(Instant::now()<deadline,"real product Warp command was not executed");
        for &label in &labels {p.app.world_mut().run_schedule(label);}
        std::thread::yield_now();
    }
    assert_eq!(p.app.world().resource::<Counters>().updates,1);
    assert!(woodpecker::session::Plugin::warp_progress(p.app.world()).is_none(),"one-tick Warp must be complete");
    // Fixture pump's admission mirror only. Independent real evidence is the product
    // completed response (audited externally), Update count and fixed Bevy clock.
    p.app.world_mut().resource_mut::<Counters>().ticks=1;
    event("product_warp_completed_observed",json!({"snapshot":p.snapshot(),"fixture_tick_is_admission_mirror":true,"product_warp_active":false}));
}
#[cfg(feature="product-control")]
fn product_idle_control(p:&mut Probe) {
    let labels=p.app.world().resource::<MainScheduleOrder>().labels.clone();
    for label in labels {p.app.world_mut().run_schedule(label);}
    event("product_idle_control",json!({"snapshot":p.snapshot()}));
}
