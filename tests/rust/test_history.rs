mod support;
use woodpecker::{
    command::{Command, inspect, tick::warp},
    report::{Failure, Report},
    session::{Session, history},
};

#[test]
fn history_contract() {
    assert_eq!(
        std::env::var("WOODPECKER_TEST_RENDERED").as_deref(),
        Ok("1"),
        "One manual window, 120-tick Warp at 2/s stopped early; 55 unique read-only hierarchy queries, no inputs. Fresh permission required: WOODPECKER_TEST_RENDERED=1"
    );
    let path = support::artifacts("rust-history");
    let mut session = Session::start(support::config(&path)).unwrap();
    let mut active = session
        .send(warp::Start {
            ticks: 120,
            pace: Some(warp::Pace::TicksPerSecond { target: 2.0 }),
        })
        .unwrap();
    let mut accepted = Vec::new();
    for index in 0..55 {
        // Vary a valid public argument, not a fictitious registered type path.
        let query = inspect::Command::Entities {
            entity: None,
            with: vec![],
            without: vec![],
            projection: inspect::entity::Projection::Hierarchy { depth: index as u8 },
        };
        let pending = session.send(query.clone()).unwrap();
        let _output: inspect::Output = support::receive(&mut session, pending);
        accepted.push(Command::Inspect(query));
        let snapshot = session.history();
        assert_eq!(snapshot.len(), 50.min(index + 2));
        assert_eq!(snapshot.last().unwrap().command, *accepted.last().unwrap());
        assert!(matches!(
            snapshot.last().unwrap().outcome,
            history::Outcome::Completed { .. }
        ));
    }
    let snapshot = session.history();
    assert_eq!(
        snapshot
            .iter()
            .map(|e| e.command.clone())
            .collect::<Vec<_>>(),
        accepted[5..]
    );
    assert!(
        snapshot
            .iter()
            .all(|e| matches!(e.outcome, history::Outcome::Completed { .. }))
    );
    assert!(
        session.try_receive(&mut active).unwrap().is_none(),
        "The evicted request must still be active"
    );
    let stop = session.send(warp::Stop {}).unwrap();
    assert!(support::receive(&mut session, stop).was_running);
    let result: warp::Completion = support::take_once(&mut session, &mut active);
    assert_eq!(result.requested_ticks, 120);
    assert_eq!(result.outcome, warp::Outcome::Stopped);
    assert!(result.executed_ticks < 120);
    let after = session.history();
    assert_eq!(after.len(), 50);
    assert_eq!(
        after[..49]
            .iter()
            .map(|e| e.command.clone())
            .collect::<Vec<_>>(),
        accepted[6..]
    );
    assert!(matches!(after[49].command, Command::Stop(_)));
    assert!(matches!(
        after[49].outcome,
        history::Outcome::Completed { .. }
    ));
    let report = Report::create(
        Failure::panic(Some("history window probe".into()), None, None),
        &session,
    );
    assert_eq!(
        support::json(&report.context().commands),
        support::json(&after)
    );
    assert!(
        support::json(&after)
            .as_array()
            .unwrap()
            .iter()
            .all(|e| e.as_object().unwrap().len() == 2)
    );
    session.shutdown().unwrap();
    support::ended(&mut session);
    std::fs::write(
        path.join("history.json"),
        serde_json::to_vec_pretty(&after).unwrap(),
    )
    .unwrap();
}
