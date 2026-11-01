mod support;
use std::time::{Duration, Instant};
use woodpecker::{
    command::{recording, tick::warp},
    session::{Error, Session},
};

#[test]
fn session_shutdown_contract() {
    assert_eq!(
        std::env::var("WOODPECKER_TEST_RENDERED").as_deref(),
        Ok("1"),
        "Two serial manual windows: clean zero-tick shutdown, then Drop during a 120-tick Warp at 2/s with recording. Expected product hard cleanup, no inputs. Fresh permission required: WOODPECKER_TEST_RENDERED=1"
    );
    let path = support::artifacts("rust-session-shutdown");
    let mut config = support::config(&path.join("clean"));
    config.launch.arguments.extend([
        "--woodpecker-diagnostics".into(),
        "--woodpecker-process-probe".into(),
    ]);
    let root = config.artifact_dir.clone();
    let mut clean = Session::start(config).unwrap();
    let process = support::owned_process(&root);
    let capabilities = support::json(clean.capabilities());
    let mut token = clean.send(warp::Stop {}).unwrap();
    let id = token.request_id();
    let command = token.command().clone();
    assert!(!support::take_once(&mut clean, &mut token).was_running);
    clean.shutdown().unwrap();
    assert!(matches!(clean.send(warp::Stop {}), Err(Error::Ended)));
    assert!(matches!(clean.shutdown(), Err(Error::Ended)));
    assert!(
        matches!(clean.try_receive_event(), Err(Error::Ended)),
        "Normal shutdown must not invent Ended event"
    );
    assert!(matches!(clean.receive_event(), Err(Error::Ended)));
    assert_eq!(support::json(clean.capabilities()), capabilities);
    assert_eq!(token.request_id(), id);
    assert_eq!(token.command(), &command);
    support::wait_gone(&process, Duration::from_secs(5));
    drop(clean);

    let mut config = support::config(&path.join("drop"));
    config.launch.arguments.extend([
        "--woodpecker-diagnostics".into(),
        "--woodpecker-process-probe".into(),
    ]);
    let root = config.artifact_dir.clone();
    let mut active = Session::start(config).unwrap();
    let process = support::owned_process(&root);
    let record = active
        .send(recording::Start {
            path: "recordings/unfinished.jsonl".into(),
        })
        .unwrap();
    support::receive(&mut active, record);
    let mut pending = active
        .send(warp::Start {
            ticks: 120,
            pace: Some(warp::Pace::TicksPerSecond { target: 2.0 }),
        })
        .unwrap();
    let id = pending.request_id();
    let command = pending.command().clone();
    let ticks = support::progress(&root, 1);
    assert!(ticks < 120);
    assert!(active.try_receive(&mut pending).unwrap().is_none());
    let start = Instant::now();
    drop(active); // The product must kill/reap; the harness sends no signal.
    // No five-second Drop guarantee exists. Use the independent final app
    // counter below to reject waiting for natural completion; elapsed time is
    // diagnostic, not a throughput/scheduler assertion.
    support::wait_gone(&process, Duration::from_secs(5));
    assert_eq!(pending.request_id(), id);
    assert_eq!(pending.command(), &command);
    assert!(support::progress(&root, 1) < 120);
    assert!(!root.join("reports").exists());
    // Deliberately no assumption that the unfinished recording has a durable footer.
    std::fs::write(path.join("drop.json"),serde_json::to_vec_pretty(&serde_json::json!({
        "pid":process.pid,"group":process.group,"ticks_before":ticks,"drop_seconds":start.elapsed().as_secs_f64(),
        "product_forced_cleanup":true,"harness_signals":0
    })).unwrap()).unwrap();
}
