//! Direct Rust Pending ownership, Drop and event delivery contracts.
use crate::{
    command::{self, recording, replay, tick::warp},
    report,
    session::{self, Session},
};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

#[test]
fn engine_startup_refuses_control_until_ready_and_reports_failure_or_stall() {
    let mut launch = config("engine_ready");
    launch.startup = session::startup::Config::Engine {
        timeout_seconds: 10,
        max_ticks: 20,
    };
    let root = launch.artifact_dir.clone();
    let mut session = session::Session::start(launch).unwrap();
    let rejected = session.send(command::tick::warp::Start {
        ticks: 1,
        pace: None,
    });
    assert!(matches!(rejected, Err(ref error) if error.code() == "startup_in_progress"));
    assert!(session.history().is_empty());
    std::fs::write(root.join("allow-engine-ready"), "").unwrap();
    wait(|| {
        matches!(
            session.try_receive_event(),
            Ok(Some(session::Event::EngineReady { ticks: 4 }))
        )
    });
    let pending = session
        .send(command::tick::warp::Start {
            ticks: 1,
            pace: None,
        })
        .unwrap();
    assert_eq!(session.receive(pending).unwrap().executed_ticks, 1);
    drop(session);

    for mode in ["engine_failed", "engine_stalled"] {
        let mut launch = config(mode);
        launch.startup = session::startup::Config::Engine {
            timeout_seconds: 1,
            max_ticks: 20,
        };
        let mut session = session::Session::start(launch).unwrap();
        let mut failure = None;
        wait(|| {
            if let Ok(Some(session::Event::StartupFailed { ticks, error })) =
                session.try_receive_event()
            {
                failure = Some((ticks, error));
            }
            failure.is_some()
        });
        let (ticks, error) = failure.unwrap();
        if mode == "engine_failed" {
            assert_eq!(ticks, Some(3));
            assert_eq!(error.code, "startup_asset_failed");
        } else {
            assert_eq!(
                ticks, None,
                "an unresponsive process has no confirmed tick count"
            );
            assert_eq!(error.code, "startup_timeout");
        }
    }
}
fn config(mode: &str) -> session::Config {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    session::Config {
        launch: session::launch::Config {
            manifest_path: root.join("tests/connection/protocol_faults/peer/Cargo.toml"),
            package: "process_fixture".into(),
            target: session::launch::Target::Binary {
                name: "process_fixture".into(),
            },
            features: vec![],
            arguments: vec![mode.into()],
        },
        artifact_dir: root.join(format!(
            "target/session-api-tests/{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )),
        tick: Default::default(),
        startup: Default::default(),
        report: report::Config {
            tracing_errors: false,
            output: "reports".into(),
            provider: report::provider::Config::Local,
        },
    }
}
fn wait(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !condition() {
        assert!(Instant::now() < deadline, "condition timed out");
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn session_rejects_an_invalid_clock_before_launching() {
    let mut configuration = config("normal");
    configuration.tick.clock = crate::command::tick::Clock::Fixed {
        seconds_per_tick: 0.0,
    };
    assert_eq!(
        configuration.validate().unwrap_err().code(),
        "invalid_config"
    );
    assert!(!configuration.artifact_dir.exists());
}

#[test]
fn fixed_clock_requires_acknowledgement_from_the_application_bridge() {
    let mut configuration = config("normal");
    configuration.tick.clock = crate::command::tick::Clock::Fixed {
        seconds_per_tick: 0.016666667,
    };
    let error = Session::start(configuration)
        .err()
        .expect("legacy bridge cannot accept a fixed clock");
    assert_eq!(error.code(), "invalid_config");
    assert!(
        error
            .message()
            .contains("does not support the requested fixed tick clock")
    );
}

#[test]
fn pending_correlation_history_and_drop() {
    let mut first = Session::start(config("normal")).unwrap();
    let mut second = Session::start(config("normal")).unwrap();
    let mut pending = first
        .send(warp::Start {
            ticks: 100,
            pace: None,
        })
        .unwrap();
    assert_eq!(pending.request_id().as_u64(), 1);
    assert_eq!(
        second.try_receive(&mut pending).unwrap_err().code(),
        "invalid_pending"
    );
    assert_eq!(
        first.shutdown().unwrap_err().code(),
        "shutdown_commands_pending"
    );
    let stop = first.send(warp::Stop {}).unwrap();
    assert_eq!(stop.request_id().as_u64(), 2);
    assert!(first.receive(stop).unwrap().was_running);
    let completion = first.receive(pending).unwrap();
    assert_eq!(completion.outcome, warp::Outcome::Stopped);

    let abandoned = first
        .send(warp::Start {
            ticks: 100,
            pace: None,
        })
        .unwrap();
    drop(abandoned);
    assert_eq!(
        first.shutdown().unwrap_err().code(),
        "shutdown_commands_pending"
    );
    let stop = first.send(warp::Stop {}).unwrap();
    first.receive(stop).unwrap();
    wait(|| {
        first
            .history()
            .iter()
            .all(|e| !matches!(e.outcome, session::history::Outcome::Unanswered))
    });
    for _ in 0..60 {
        let pending = first
            .send(warp::Start {
                ticks: 1,
                pace: None,
            })
            .unwrap();
        first.receive(pending).unwrap();
    }
    assert_eq!(first.history().len(), 50);
    let mut once = first.send(warp::Stop {}).unwrap();
    wait(|| first.try_receive(&mut once).unwrap().is_some());
    assert_eq!(
        first.try_receive(&mut once).unwrap_err().code(),
        "invalid_pending"
    );
    first.shutdown().unwrap();
    second.shutdown().unwrap();
    assert!(first.try_receive_event().is_err());
}

#[test]
fn pipe_drain_and_progress_do_not_need_polling() {
    let mut session = Session::start(config("full_pipes")).unwrap();
    let pending = session
        .send(warp::Start {
            ticks: 1,
            pace: None,
        })
        .unwrap();
    wait(|| {
        matches!(
            session.history()[0].outcome,
            session::history::Outcome::Completed { .. }
        )
    });
    assert_eq!(session.receive(pending).unwrap().executed_ticks, 1);
    session.shutdown().unwrap();
}

#[test]
fn overflow_reserves_exactly_one_end_event() {
    let settings = config("overflow");
    let root = settings.artifact_dir.clone();
    let mut session = Session::start(settings).unwrap();
    let pid: i32 = std::fs::read_to_string(root.join("pid"))
        .unwrap()
        .parse()
        .unwrap();
    std::fs::write(root.join("allow-overflow"), b"start burst").unwrap();
    // Wait for overflow cleanup without consuming the bounded event queue.
    wait(|| unsafe { libc::kill(pid, 0) } != 0);
    // Process exit can precede draining its buffered protocol messages. Freeze
    // the producer before consuming; reading early would reopen queue capacity.
    wait(|| session.worker.as_ref().unwrap().is_finished());
    let mut count = 0;
    loop {
        match session.receive_event().unwrap() {
            session::Event::ProtocolError { .. } => count += 1,
            session::Event::Ended {
                reason:
                    session::EndReason::EventQueueOverflow {
                        capacity,
                        dropped_events,
                    },
            } => {
                assert_eq!(count, 256);
                assert_eq!(capacity, 256);
                assert!(dropped_events > 0);
                break;
            }
            event => panic!("unexpected event: {event:?}"),
        }
    }
    assert!(session.receive_event().is_err());
}

#[test]
fn dropped_recording_control_still_finishes_without_a_consumer() {
    let settings = config("normal");
    let root = settings.artifact_dir.clone();
    let mut session = Session::start(settings).unwrap();
    drop(
        session
            .send(recording::Start {
                path: "empty.jsonl".into(),
            })
            .unwrap(),
    );
    wait(|| {
        session.history().last().is_some_and(|entry| {
            matches!(entry.outcome, session::history::Outcome::Completed { .. })
        })
    });
    let stop = session.send(recording::Stop {}).unwrap();
    assert_eq!(session.receive(stop).unwrap().recorded_commands, 0);
    let rows: Vec<serde_json::Value> = std::fs::read_to_string(root.join("empty.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1]["outcome"], "stopped");
    session.shutdown().unwrap();
}

#[test]
fn unsolicited_ready_does_not_change_rust_capabilities() {
    let mut session = Session::start(config("unsolicited")).unwrap();
    let stop = session.send(warp::Stop {}).unwrap();
    session.receive(stop).unwrap();
    assert!(
        matches!(session.receive_event().unwrap(), session::Event::ProtocolError { code, .. } if code == "fixture")
    );
    assert!(
        matches!(session.receive_event().unwrap(), session::Event::ProtocolError { code, .. } if code == "unexpected_ready")
    );
    assert!(!session.capabilities().screenshot);
    session.shutdown().unwrap();
}

#[test]
fn dropped_replay_handle_keeps_execution_and_history_progressing() {
    let settings = config("normal");
    let root = settings.artifact_dir.clone();
    let mut session = Session::start(settings).unwrap();
    std::fs::write(root.join("input.jsonl"), concat!(
        "{\"type\":\"recording_started\",\"format_version\":1}\n",
        "{\"type\":\"command\",\"command\":\"tick.warp.start\",\"arguments\":{\"ticks\":1},\"outcome\":{\"status\":\"unanswered\"}}\n",
        "{\"type\":\"recording_ended\",\"outcome\":\"session_ended\",\"recorded_commands\":1}\n"
    )).unwrap();
    drop(
        session
            .send(replay::Start {
                path: "input.jsonl".into(),
            })
            .unwrap(),
    );
    wait(|| {
        session.history().len() == 2
            && session
                .history()
                .iter()
                .all(|entry| matches!(entry.outcome, session::history::Outcome::Completed { .. }))
    });
    session.shutdown().unwrap();
}
