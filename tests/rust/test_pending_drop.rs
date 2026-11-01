mod support;
use std::time::{Duration, Instant};
use woodpecker::{
    command::{Command, recording, tick::warp},
    session::{Error, Session, history},
};

#[test]
fn pending_drop_contract() {
    assert_eq!(
        std::env::var("WOODPECKER_TEST_RENDERED").as_deref(),
        Ok("1"),
        "One manual game window, twelve ticks at 2/s finish naturally after dropping Pending; opt-in counter files, no inputs. Fresh permission required: WOODPECKER_TEST_RENDERED=1"
    );
    let path = support::artifacts("rust-pending-drop");
    let mut config = support::config(&path);
    config.launch.arguments.extend([
        "--woodpecker-diagnostics".into(),
        "--woodpecker-process-probe".into(),
    ]);
    let root = config.artifact_dir.clone();
    let mut session = Session::start(config).unwrap();
    let recording = session
        .send(recording::Start {
            path: "recordings/dropped.jsonl".into(),
        })
        .unwrap();
    assert_eq!(
        support::receive(&mut session, recording).path,
        "recordings/dropped.jsonl"
    );
    let request = warp::Start {
        ticks: 12,
        pace: Some(warp::Pace::TicksPerSecond { target: 2.0 }),
    };
    let mut pending = session.send(request.clone()).unwrap();
    let request_id = pending.request_id();
    assert!(support::progress(&root, 1) < 12);
    assert!(session.try_receive(&mut pending).unwrap().is_none());
    drop(pending); // No receive/poll, Stop or resubmission follows during the remaining Warp.
    let before = support::json(&session.history());
    assert_eq!(before.as_array().unwrap().len(), 2);
    assert_eq!(
        before[1]["command"],
        support::json(&Command::Start(request.clone()))
    );
    assert_eq!(before[1]["outcome"]["status"], "unanswered");
    assert!(
        matches!(session.shutdown(),Err(Error::ShutdownBlocked { ref code, .. }) if code=="shutdown_commands_pending")
    );
    assert_eq!(
        support::json(&session.history()),
        before,
        "Blocked shutdown adds no history"
    );
    assert_eq!(support::progress(&root, 12), 12);
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let snapshot = session.history();
        if matches!(snapshot[1].outcome, history::Outcome::Completed { .. }) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "Dropped request did not settle history"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    let snapshot = support::json(&session.history());
    assert_eq!(
        snapshot[1],
        serde_json::json!({"command":support::json(&Command::Start(request)),
        "outcome":{"status":"completed","output":{"requested_ticks":12,"executed_ticks":12,"outcome":"completed"}}})
    );
    let stop = session.send(recording::Stop {}).unwrap();
    assert_eq!(support::receive(&mut session, stop).recorded_commands, 1);
    let rows = support::json_lines(&root.join("recordings/dropped.jsonl"));
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[1]["command"], "tick.warp.start");
    assert_eq!(rows[1]["outcome"], snapshot[1]["outcome"]);
    assert_eq!(
        rows[2],
        serde_json::json!({"type":"recording_ended","outcome":"stopped","recorded_commands":1})
    );
    session.shutdown().unwrap();
    support::ended(&mut session);
    std::fs::write(
        path.join("dropped.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "dropped_request_id":request_id.as_u64(),"history":snapshot,"ticks":12
        }))
        .unwrap(),
    )
    .unwrap();
}
