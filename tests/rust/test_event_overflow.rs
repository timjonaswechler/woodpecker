mod support;
use std::time::Duration;
use woodpecker::{
    command::{input::keyboard, tick::warp},
    report::Origin,
    session::{EndReason, Error, Event, Session},
};

#[test]
fn event_overflow_contract() {
    assert_eq!(
        std::env::var("WOODPECKER_TEST_RENDERED").as_deref(),
        Ok("1"),
        "One manual window, F9/one requested tick emits 300 bounded tracing events. No event reads until product overflow kills the owned group. Fresh permission required: WOODPECKER_TEST_RENDERED=1"
    );
    let path = support::artifacts("rust-event-overflow");
    let _environment = support::Environment::new([(
        "RUST_LOG".into(),
        "off,signalstation::overflow=error".into(),
    )]);
    let mut config = support::config(&path);
    config.report.tracing_errors = true;
    config.launch.arguments.extend([
        "--woodpecker-diagnostics".into(),
        "--woodpecker-process-probe".into(),
        "--woodpecker-report-action".into(),
        "event-overflow".into(),
    ]);
    let root = config.artifact_dir.clone();
    let mut session = Session::start(config).unwrap();
    let process = support::owned_process(&root);
    let press = session
        .send(keyboard::Press {
            key: keyboard::Key::F9,
        })
        .unwrap();
    support::receive(&mut session, press);
    let mut tick = session
        .send(warp::Start {
            ticks: 1,
            pace: None,
        })
        .unwrap();
    // No Server and no receive_event/try_receive_event before this process/group check.
    support::wait_gone(&process, Duration::from_secs(15));
    assert!(matches!(session.send(warp::Stop {}), Err(Error::Ended)));
    let mut events = Vec::new();
    loop {
        match session.try_receive_event() {
            Ok(Some(event)) => events.push(event),
            Ok(None) => panic!("Handle must be terminal once product cleanup is complete"),
            Err(Error::Ended) => break,
            Err(error) => panic!("{error}"),
        }
    }
    assert_eq!(events.len(), 257);
    for (index, event) in events[..256].iter().enumerate() {
        let Event::Failure { failure } = event else {
            panic!("Expected ordered tracing Failure, got {event:?}");
        };
        assert!(
            matches!(failure.origin(),Origin::TracingError { target:Some(target),.. } if target=="signalstation::overflow")
        );
        assert_eq!(
            failure.message(),
            Some(format!("Signalstation overflow event {index}").as_str())
        );
    }
    let Event::Ended {
        reason:
            EndReason::EventQueueOverflow {
                capacity,
                dropped_events,
            },
    } = &events[256]
    else {
        panic!("Exactly one reserved overflow Ended must be last");
    };
    assert_eq!(*capacity, 256);
    assert!(*dropped_events > 0);
    assert!(matches!(session.try_receive_event(), Err(Error::Ended)));
    assert!(matches!(session.receive_event(), Err(Error::Ended)));
    match session.try_receive(&mut tick) {
        Ok(Some(value)) => {
            assert_eq!(value.requested_ticks, 1);
            assert_eq!(value.executed_ticks, 1);
            assert_eq!(value.outcome, warp::Outcome::Completed);
        }
        Err(Error::Ended) => {}
        other => panic!(
            "Tick is known completed or unknown after overflow, never a fabricated success: {other:?}"
        ),
    }
    assert!(
        !root.join("reports").exists(),
        "Direct handle does not publish reports implicitly"
    );
    std::fs::write(
        path.join("events.json"),
        serde_json::to_vec_pretty(&events).unwrap(),
    )
    .unwrap();
    drop(session); // All product-owned work joined before restoring the environment.
}
