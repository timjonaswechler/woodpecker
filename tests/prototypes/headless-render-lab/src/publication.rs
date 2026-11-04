// Probe-local publisher, NOT Woodpecker's screenshot/server implementation.
use std::io::Write;

#[derive(Clone)]
struct Lease {
    request:u64, generation:u64, min_frame:u64, tick:u64, state:u64,
    target:AssetId<Image>, deadline:Instant,
}
#[derive(Debug, PartialEq, Eq)]
enum PublicationState { Active, Staged, Published, Terminal(&'static str) }
struct Publisher {
    root:PathBuf, lease:Lease, status:PublicationState,
    staging:PathBuf, destination:PathBuf, receipt:Option<Value>, successes:u64,
}
impl Publisher {
    fn new(case_root:&Path,lease:Lease)->Self {
        let root=case_root.join("publisher");std::fs::create_dir_all(&root).unwrap();
        let staging=root.join(format!(".staging-{}-{}",lease.request,lease.generation));
        let destination=root.join(format!("result-{}-{}",lease.request,lease.generation));
        event("publisher_accepted",json!({"request":lease.request,"generation":lease.generation,"minimum_frame":lease.min_frame,"target":format!("{:?}",lease.target)}));
        Self {root,lease,status:PublicationState::Active,staging,destination,receipt:None,successes:0}
    }
    fn abort(&mut self,reason:&'static str) {
        assert_ne!(self.status,PublicationState::Published,"published history is not retroactively aborted");
        self.status=PublicationState::Terminal(reason);
        event("publisher_terminal",json!({"reason":reason,"final_path":self.destination}));
    }
    fn gate(&mut self,fatal:Option<&str>)->Result<(),&'static str> {
        if self.status==PublicationState::Published {return Err("already_published");}
        if let PublicationState::Terminal(reason)=self.status {return Err(reason);}
        if let Some(error)=fatal {
            self.abort("observed_fatal");event("publisher_observed_fatal",json!({"fatal":error}));return Err("observed_fatal");
        }
        if Instant::now()>=self.lease.deadline {self.abort("timeout");return Err("timeout");}
        Ok(())
    }
    fn stage(&mut self,c:&Candidate,fatal:Option<&str>,partial_write_fault:bool)->Result<(),&'static str> {
        self.gate(fatal)?;
        if self.status!=PublicationState::Active {return Err("already_staged");}
        let mismatch=if c.token.request!=self.lease.request {Some("request")}
            else if c.token.generation!=self.lease.generation {Some("generation")}
            else if c.token.frame<self.lease.min_frame {Some("frame")}
            else if c.token.tick!=self.lease.tick || c.token.state!=self.lease.state {Some("tick_state")}
            else if c.target!=self.lease.target {Some("target")}
            else if c.bytes.len()!=(W*H*4) as usize || !c.queue_done || !pixel_oracle(&c.bytes,c.token.state) {Some("readback_or_pixels")}
            else {None};
        if let Some(reason)=mismatch {
            event("publisher_stale_rejected",json!({"reason":reason,"candidate":c.token.json()}));return Err(reason);
        }
        if std::fs::create_dir(&self.staging).is_err() {self.abort("staging_io");return Err("staging_io");}
        let png=self.staging.join("frame.png");
        if partial_write_fault {
            // Real partial bytes in a nonfinal path; the failure itself is explicit CPU injection.
            let mut f=std::fs::File::create_new(&png).unwrap();f.write_all(b"\x89PNG\r\n\x1a\n").unwrap();f.sync_all().unwrap();
            self.abort("injected_partial_write");
            event("partial_write_fault_injection",json!({"staging_bytes":8,"final_output":false}));
            return Err("injected_partial_write");
        }
        if let Err(error)=image::save_buffer_with_format(&png,&c.bytes,W,H,image::ColorType::Rgba8,image::ImageFormat::Png) {
            self.abort("png_io");event("publisher_io_error",json!({"error":error.to_string()}));return Err("png_io");
        }
        std::fs::File::open(&png).unwrap().sync_all().unwrap();
        let decoded=image::open(&png).unwrap().to_rgba8();assert_eq!(decoded.as_raw(),&c.bytes);
        let receipt=json!({"request":c.token.request,"generation":c.token.generation,"frame":c.token.frame,
            "tick":c.token.tick,"state":c.token.state,"target":format!("{:?}",c.target),"submission":c.submission,
            "width":W,"height":H,"format":"Rgba8UnormSrgb -> RGBA8 PNG","stride":STRIDE,
            "map_result":"Ok","pixels_sha256":hash(&c.bytes),"png_sha256":hash(&std::fs::read(&png).unwrap()),
            "contract":"finite frame + no observed fatal at final owner check; not future device health"});
        let mut f=std::fs::File::create_new(self.staging.join("receipt.json")).unwrap();
        f.write_all(&serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();f.sync_all().unwrap();
        self.receipt=Some(receipt);self.status=PublicationState::Staged;
        event("publisher_stage_barrier",json!({"candidate":c.token.json(),"staging":self.staging,"destination_exists":self.destination.exists()}));
        Ok(())
    }
    fn commit(&mut self,fatal:Option<&str>)->Result<Value,&'static str> {
        self.gate(fatal)?;
        if self.status!=PublicationState::Staged {return Err("not_staged");}
        event("publisher_final_check",json!({"observed_fatal":false,"status":"Staged","request":self.lease.request,"generation":self.lease.generation}));
        // Same-parent atomic directory rename: final PNG and receipt become visible together.
        // No assertion of crash durability, cross-filesystem atomicity or a product sandbox.
        if let Err(error)=std::fs::rename(&self.staging,&self.destination) {
            self.abort("rename_io");event("publisher_io_error",json!({"error":error.to_string(),"operation":"rename"}));return Err("rename_io");
        }
        self.status=PublicationState::Published;self.successes+=1;assert_eq!(self.successes,1);
        let receipt=self.receipt.clone().unwrap();
        event("publisher_success",json!({"receipt":receipt,"destination":self.destination,"success_count":self.successes}));
        Ok(receipt)
    }
    fn assert_no_success(&self) {
        assert_eq!(self.successes,0);
        assert!(!self.destination.is_dir(),"no final committed bundle may exist");
        event("publisher_no_success_proven",json!({"status":format!("{:?}",self.status),"staging_exists":self.staging.exists(),"final_bundle":false}));
    }
}
