use super::*;
use crate::{
    command::{inspect, replay, tick::warp},
    test_support as support,
};
use std::time::Duration;

#[test]
fn exhausted_ids_never_admit_work_or_lose_pending_and_reserved_shutdown() {
    // Two external-wire variants. No Cargo launch, app, renderer or global env.
    for mode in ["external", "internal-replay"] {
        let path = support::artifacts(&format!("request-ids-{mode}"));
        let config = support::config(&path);
        let context = support::context(&config);
        let root = cap_std::fs::Dir::open_ambient_dir(&path, cap_std::ambient_authority()).unwrap();
        if mode == "internal-replay" {
            let header = serde_json::json!({"type":"recording_started","format_version":2,"clock":config.tick.clock});
            let command = serde_json::json!({"type":"command","command":"input.keyboard.press","arguments":{"key":"g"},"outcome":{"status":"completed","output":null}});
            let footer = serde_json::json!({"type":"recording_ended","outcome":"stopped","recorded_commands":1});
            std::fs::write(
                path.join("plan.jsonl"),
                format!("{header}\n{command}\n{footer}\n"),
            )
            .unwrap();
        }
        let mut command = std::process::Command::new("python3");
        command
            .arg(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/faults/internal_peer.py"),
            )
            .arg(&path);
        let process = process::Process::spawn(command).unwrap();
        let pid = process.child.id();
        let shared = Arc::new(Shared::default());
        let cancel = Arc::new(AtomicBool::new(false));
        let (operations, input) = mpsc::channel();
        let (ready, initialized) = mpsc::channel();
        let state = shared.clone();
        let abort = cancel.clone();
        let project = path.clone();
        let worker = std::thread::spawn(move || {
            run(
                config,
                input,
                state,
                abort,
                ready,
                Some(TestControls {
                    launched: Some((process, root, context, project)),
                    next_id: u64::MAX - 1,
                    ..Default::default()
                }),
            )
        });
        let mut owner = support::WorkerGuard {
            cancel: cancel.clone(),
            handle: Some(worker),
        };
        let capabilities = initialized
            .recv_timeout(Duration::from_secs(10))
            .unwrap()
            .unwrap();
        let mut session = Session {
            shared,
            operations,
            cancel,
            worker: owner.handle.take(),
            capabilities,
        };
        if mode == "external" {
            let mut work = session
                .send(warp::Start {
                    ticks: 1,
                    pace: None,
                })
                .unwrap();
            assert_eq!(work.request_id().as_u64(), u64::MAX - 1);
            support::wait(|| path.join("wire.jsonl").exists());
            assert!(session.try_receive(&mut work).unwrap().is_none());
            let before = serde_json::to_value(session.history()).unwrap();
            for _ in 0..2 {
                assert!(matches!(
                    session.send(warp::Stop {}),
                    Err(Error::RequestIdExhausted)
                ));
                assert_eq!(serde_json::to_value(session.history()).unwrap(), before);
            }
            assert!(
                matches!(session.shutdown(),Err(Error::ShutdownBlocked {ref code,..}) if code=="shutdown_commands_pending")
            );
            assert_eq!(support::json_lines(path.join("wire.jsonl")).len(), 1);
            std::fs::write(
                path.join("release-warp"),
                "explicit external response gate; not game ticks",
            )
            .unwrap();
            assert_eq!(
                support::receive(&mut session, &mut work)
                    .unwrap()
                    .executed_ticks,
                1
            );
        } else {
            let mut plan = session
                .send(replay::Start {
                    path: "plan.jsonl".into(),
                })
                .unwrap();
            assert_eq!(plan.request_id().as_u64(), u64::MAX - 1);
            let output = support::receive(&mut session, &mut plan).unwrap();
            let value = serde_json::to_value(output).unwrap();
            assert_eq!(value["outcome"]["kind"], "blocked");
            assert_eq!(value["outcome"]["code"], "request_id_exhausted");
            assert_eq!(
                session.history().len(),
                1,
                "Unaccepted internal command has no history entry"
            );
            assert!(
                !path.join("wire.jsonl").exists(),
                "No plan prefix may reach the peer"
            );
            assert!(matches!(
                session.send(inspect::Command::Entities {
                    entity: None,
                    with: vec![],
                    without: vec![],
                    projection: inspect::entity::Projection::Summary {}
                }),
                Err(Error::RequestIdExhausted)
            ));
        }
        session.shutdown().unwrap();
        let wire = support::json_lines(path.join("wire.jsonl"));
        assert_eq!(wire.last().unwrap()["request_id"], u64::MAX);
        assert_eq!(wire.last().unwrap()["command"], "shutdown");
        assert_eq!(wire.len(), if mode == "external" { 2 } else { 1 });
        assert!(matches!(session.send(warp::Stop {}), Err(Error::Ended)));
        drop(session);
        assert_eq!(unsafe { libc::kill(pid as i32, 0) }, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
    }
}
