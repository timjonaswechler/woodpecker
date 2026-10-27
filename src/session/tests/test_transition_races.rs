use crate::session::*;
use crate::{
    command::{inspect, recording as rec_command, replay as replay_command},
    test_support as support,
};
use std::{
    io::{self, BufRead, BufReader, Read, Write},
    time::Duration,
};

struct Gate {
    entered: mpsc::Sender<&'static str>,
    release: Mutex<mpsc::Receiver<bool>>,
}
impl Gate {
    fn pause(&self, stage: &'static str) -> io::Result<()> {
        self.entered.send(stage).map_err(io::Error::other)?;
        match self
            .release
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(10))
        {
            Ok(true) => Ok(()),
            other => Err(io::Error::other(format!(
                "owned {stage} fault/deadline: {other:?}"
            ))),
        }
    }
}
struct GatedFile {
    file: cap_std::fs::File,
    gate: Arc<Gate>,
    header: bool,
}
impl Write for GatedFile {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.header {
            self.header = false;
            self.gate.pause("header")?;
        }
        self.file.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}
impl recording::file::Sink for GatedFile {
    fn sync_all(&self) -> io::Result<()> {
        self.gate.pause("footer-sync")?;
        self.file.sync_all()
    }
}
struct GatedReader {
    reader: BufReader<cap_std::fs::File>,
    gate: Option<Arc<Gate>>,
}
impl Read for GatedReader {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let data = self.fill_buf()?;
        let n = bytes.len().min(data.len());
        bytes[..n].copy_from_slice(&data[..n]);
        self.consume(n);
        Ok(n)
    }
}
impl BufRead for GatedReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if let Some(gate) = self.gate.take() {
            gate.pause("loader-read")?;
        }
        self.reader.fill_buf()
    }
    fn consume(&mut self, n: usize) {
        self.reader.consume(n)
    }
}
fn query() -> inspect::Command {
    inspect::Command::Entities {
        entity: None,
        with: vec![],
        without: vec![],
        projection: inspect::entity::Projection::Summary {},
    }
}

#[test]
fn transition_races_obey_real_file_barriers_and_never_release_cancelled_plans() {
    // Six external-wire sessions, no app/renderer or global env. Real worker IO.
    for mode in ["success", "header-failed", "footer-failed"] {
        std::thread::scope(|scope| {
            let path = support::artifacts(&format!("recording-transition-{mode}"));
            let config = support::config(&path);
            let context = support::context(&config);
            let clock = config.tick.clock.clone();
            let root =
                cap_std::fs::Dir::open_ambient_dir(&path, cap_std::ambient_authority()).unwrap();
            let io_root = root.try_clone().unwrap();
            let (entered, arrival) = mpsc::channel();
            let (release, released) = mpsc::channel();
            let gate = Arc::new(Gate {
                entered,
                release: Mutex::new(released),
            });
            let (file_send, file_input) = mpsc::channel();
            let (file_reply, file_replies) = mpsc::channel();
            // Actual production worker and actual Recording with a wrapped OS File.
            let file_worker = scope.spawn(move || {
                recording::worker::run(io_root, file_input, file_reply, |root, path| {
                    recording::file::Recording::start_with(root, path, &clock, |file| GatedFile {
                        file,
                        gate: gate.clone(),
                        header: true,
                    })
                })
            });
            let recorder = recording::Recorder::from_io(file_send, file_replies);
            let mut command = std::process::Command::new("python3");
            command
                .arg(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("tests/faults/internal_peer.py"),
                )
                .arg(&path);
            let process = process::Process::spawn(command).unwrap();
            let pid = process.child.id();
            let wire = process.writer.as_ref().unwrap().clone();
            let shared = Arc::new(Shared::default());
            let cancel = Arc::new(AtomicBool::new(false));
            let (operations, input) = mpsc::channel();
            let (ready, initialized) = mpsc::channel();
            let (check, checks) = mpsc::channel();
            let (ack, checked) = mpsc::channel();
            let state = shared.clone();
            let abort = cancel.clone();
            let project = path.clone();
            let worker = std::thread::spawn(move || {
                coordinator::run(
                    config,
                    input,
                    state,
                    abort,
                    ready,
                    Some(coordinator::TestControls {
                        launched: Some((process, root, context, project)),
                        recorder: Some(recorder),
                        loop_checkpoint: Some((checks, ack)),
                        ..Default::default()
                    }),
                )
            });
            let mut owner = support::WorkerGuard {
                cancel: cancel.clone(),
                handle: Some(worker),
            };
            let wire = wire; // Before the pre-Ready worker guard on unwind as well.
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
            let wire = wire; // Drop this diagnostic sender before Session, including unwind.
            let mut start = session
                .send(rec_command::Start {
                    path: "recording.jsonl".into(),
                })
                .unwrap();
            assert_eq!(
                arrival.recv_timeout(Duration::from_secs(10)).unwrap(),
                "header"
            );
            let mut inside = session.send(query()).unwrap();
            let mut duplicate = session
                .send(rec_command::Start {
                    path: "../invalid.jsonl".into(),
                })
                .unwrap();
            assert_eq!(
                support::receive(&mut session, &mut duplicate)
                    .unwrap_err()
                    .code(),
                "recording_transition_in_progress"
            );
            let mut premature = session.send(rec_command::Stop {}).unwrap();
            assert_eq!(
                support::receive(&mut session, &mut premature)
                    .unwrap_err()
                    .code(),
                "recording_transition_in_progress"
            );
            assert!(session.try_receive(&mut start).unwrap().is_none());
            assert!(session.try_receive(&mut inside).unwrap().is_none());
            // Coordinator full-loop ack followed by a real pipe fence: no timing-based absence assertion.
            check.send(()).unwrap();
            checked.recv_timeout(Duration::from_secs(10)).unwrap();
            wire.send(
                serde_json::json!({"command":"test.fence","label":"header-fence"}).to_string(),
            )
            .unwrap();
            support::wait(|| path.join("header-fence").exists());
            assert!(
                support::json_lines(path.join("wire.jsonl"))
                    .iter()
                    .all(|v| v["command"] == "test.fence")
            );
            release.send(mode != "header-failed").unwrap();
            if mode == "header-failed" {
                assert!(matches!(
                    support::receive(&mut session, &mut start),
                    Err(Error::Io { .. })
                ));
                assert!(
                    support::receive(&mut session, &mut inside)
                        .unwrap()
                        .items
                        .is_empty()
                );
                assert!(!path.join("recording.jsonl").exists());
            } else {
                assert_eq!(
                    support::receive(&mut session, &mut start).unwrap().path,
                    "recording.jsonl"
                );
                assert!(
                    support::receive(&mut session, &mut inside)
                        .unwrap()
                        .items
                        .is_empty()
                );
                let mut stop = session.send(rec_command::Stop {}).unwrap();
                assert_eq!(
                    arrival.recv_timeout(Duration::from_secs(10)).unwrap(),
                    "footer-sync"
                );
                let mut outside = session.send(query()).unwrap();
                let mut second_stop = session.send(rec_command::Stop {}).unwrap();
                assert_eq!(
                    support::receive(&mut session, &mut second_stop)
                        .unwrap_err()
                        .code(),
                    "recording_transition_in_progress"
                );
                assert!(session.try_receive(&mut stop).unwrap().is_none());
                assert!(session.try_receive(&mut outside).unwrap().is_none());
                check.send(()).unwrap();
                checked.recv_timeout(Duration::from_secs(10)).unwrap();
                wire.send(
                    serde_json::json!({"command":"test.fence","label":"footer-fence"}).to_string(),
                )
                .unwrap();
                support::wait(|| path.join("footer-fence").exists());
                assert_eq!(
                    support::json_lines(path.join("wire.jsonl"))
                        .iter()
                        .filter(|v| v["command"] == "inspect.query")
                        .count(),
                    1
                );
                release.send(mode != "footer-failed").unwrap();
                if mode == "footer-failed" {
                    assert!(matches!(
                        support::receive(&mut session, &mut stop),
                        Err(Error::Io { .. })
                    ));
                } else {
                    assert_eq!(
                        support::receive(&mut session, &mut stop)
                            .unwrap()
                            .recorded_commands,
                        1
                    );
                }
                assert!(
                    support::receive(&mut session, &mut outside)
                        .unwrap()
                        .items
                        .is_empty()
                );
                let rows = support::json_lines(path.join("recording.jsonl"));
                assert_eq!(rows.len(), 3);
                assert_eq!(rows[1]["command"], "inspect.query");
                if mode == "success" {
                    assert_eq!(
                        rows[2],
                        serde_json::json!({"type":"recording_ended","outcome":"stopped","recorded_commands":1})
                    );
                }
                // Footer bytes alone are not claimed as a successful durable Stop in footer-failed.
            }
            assert!(session.try_receive_event().unwrap().is_none());
            drop(wire);
            session.shutdown().unwrap();
            drop(session);
            file_worker.join().unwrap();
            assert_eq!(unsafe { libc::kill(pid as i32, 0) }, -1);
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::ESRCH)
            );
        });
    }
    for mode in [
        "cancel-success",
        "cancel-racing-io-error",
        "error-before-stop",
    ] {
        let path = support::artifacts(&format!("replay-transition-{mode}"));
        let config = support::config(&path);
        let context = support::context(&config);
        let root = cap_std::fs::Dir::open_ambient_dir(&path, cap_std::ambient_authority()).unwrap();
        let header = serde_json::json!({"type":"recording_started","format_version":2,"clock":config.tick.clock});
        let command = serde_json::json!({"type":"command","command":"input.keyboard.press","arguments":{"key":"g"},"outcome":{"status":"completed","output":null}});
        let footer =
            serde_json::json!({"type":"recording_ended","outcome":"stopped","recorded_commands":1});
        std::fs::write(
            path.join("plan.jsonl"),
            format!("{header}\n{command}\n{footer}\n"),
        )
        .unwrap();
        std::fs::write(path.join("empty.jsonl"),format!("{header}\n{}\n",serde_json::json!({"type":"recording_ended","outcome":"stopped","recorded_commands":0}))).unwrap();
        let (entered, arrival) = mpsc::channel();
        let (release, released) = mpsc::channel();
        let gate = Arc::new(Gate {
            entered,
            release: Mutex::new(released),
        });
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let finished = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let loads = calls.clone();
        let completions = finished.clone();
        let mut replay = replay::Replay::new(
            Arc::new(root.try_clone().unwrap()),
            config.tick.clock.clone(),
        );
        replay.loader = Some(Arc::new(move |root, path, cancel, clock| {
            let first = loads.fetch_add(1, Ordering::SeqCst) == 0;
            let reader = GatedReader {
                reader: BufReader::new(recording::file::open_read(root, path)?),
                gate: if first { Some(gate.clone()) } else { None },
            };
            let result = replay::test_parse(reader, path, cancel, clock);
            completions.fetch_add(1, Ordering::SeqCst);
            result
        }));
        let mut command = std::process::Command::new("python3");
        command
            .arg(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/faults/internal_peer.py"),
            )
            .arg(&path);
        let process = process::Process::spawn(command).unwrap();
        let pid = process.child.id();
        let wire = process.writer.as_ref().unwrap().clone();
        let shared = Arc::new(Shared::default());
        let cancel = Arc::new(AtomicBool::new(false));
        let (operations, input) = mpsc::channel();
        let (ready, initialized) = mpsc::channel();
        let (check, checks) = mpsc::channel();
        let (ack, checked) = mpsc::channel();
        let state = shared.clone();
        let abort = cancel.clone();
        let project = path.clone();
        let worker = std::thread::spawn(move || {
            coordinator::run(
                config,
                input,
                state,
                abort,
                ready,
                Some(coordinator::TestControls {
                    launched: Some((process, root, context, project)),
                    replay: Some(replay),
                    loop_checkpoint: Some((checks, ack)),
                    ..Default::default()
                }),
            )
        });
        let mut owner = support::WorkerGuard {
            cancel: cancel.clone(),
            handle: Some(worker),
        };
        let wire = wire; // Release the diagnostic pipe clone before pre-Ready cleanup.
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
        let wire = wire; // Dropped before Session on both success and assertion failure.
        let mut plan = session
            .send(replay_command::Start {
                path: "plan.jsonl".into(),
            })
            .unwrap();
        assert_eq!(
            arrival.recv_timeout(Duration::from_secs(10)).unwrap(),
            "loader-read"
        );
        if mode == "error-before-stop" {
            release.send(false).unwrap();
            assert!(matches!(
                support::receive(&mut session, &mut plan),
                Err(Error::Io { .. })
            ));
            let mut stop = session.send(replay_command::Stop {}).unwrap();
            assert!(
                !support::receive(&mut session, &mut stop)
                    .unwrap()
                    .was_running
            );
        } else {
            let mut stop_a = session.send(replay_command::Stop {}).unwrap();
            let mut stop_b = session.send(replay_command::Stop {}).unwrap();
            let mut premature = session
                .send(replay_command::Start {
                    path: "empty.jsonl".into(),
                })
                .unwrap();
            assert_eq!(
                support::receive(&mut session, &mut premature)
                    .unwrap_err()
                    .code(),
                "replay_already_running"
            );
            let mut foreign = session
                .send(rec_command::Start {
                    path: "../invalid.jsonl".into(),
                })
                .unwrap();
            assert_eq!(
                support::receive(&mut session, &mut foreign)
                    .unwrap_err()
                    .code(),
                "replay_in_progress"
            );
            assert!(session.try_receive(&mut plan).unwrap().is_none());
            assert!(session.try_receive(&mut stop_a).unwrap().is_none());
            check.send(()).unwrap();
            checked.recv_timeout(Duration::from_secs(10)).unwrap();
            wire.send(
                serde_json::json!({"command":"test.fence","label":"loader-held"}).to_string(),
            )
            .unwrap();
            support::wait(|| path.join("loader-held").exists());
            assert!(
                support::json_lines(path.join("wire.jsonl"))
                    .iter()
                    .all(|v| v["command"] == "test.fence")
            );
            release.send(mode == "cancel-success").unwrap();
            assert_eq!(
                support::receive(&mut session, &mut plan).unwrap().outcome,
                replay_command::Outcome::Stopped {}
            );
            assert!(
                support::receive(&mut session, &mut stop_a)
                    .unwrap()
                    .was_running
            );
            assert!(
                support::receive(&mut session, &mut stop_b)
                    .unwrap()
                    .was_running
            );
        }
        assert_eq!(finished.load(Ordering::SeqCst), 1);
        let mut fresh = session
            .send(replay_command::Start {
                path: "empty.jsonl".into(),
            })
            .unwrap();
        assert_eq!(
            support::receive(&mut session, &mut fresh).unwrap().outcome,
            replay_command::Outcome::Completed {}
        );
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(finished.load(Ordering::SeqCst), 2);
        check.send(()).unwrap();
        checked.recv_timeout(Duration::from_secs(10)).unwrap();
        wire.send(serde_json::json!({"command":"test.fence","label":"loader-final"}).to_string())
            .unwrap();
        support::wait(|| path.join("loader-final").exists());
        assert!(
            support::json_lines(path.join("wire.jsonl"))
                .iter()
                .all(|v| v["command"] == "test.fence"),
            "Cancelled/error plan must never dispatch its prefix later"
        );
        assert!(session.try_receive_event().unwrap().is_none());
        drop(wire);
        session.shutdown().unwrap();
        drop(session);
        assert_eq!(unsafe { libc::kill(pid as i32, 0) }, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
    }
}
