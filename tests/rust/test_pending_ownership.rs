mod support;
use woodpecker::{
    command::{Command, tick::warp},
    session::{Error, Session},
};

#[test]
fn pending_ownership_contract() {
    assert_eq!(
        std::env::var("WOODPECKER_TEST_RENDERED").as_deref(),
        Ok("1"),
        "Two real manual-startup windows; A two ticks, B zero; no inputs. Fresh permission required: WOODPECKER_TEST_RENDERED=1"
    );
    let path = support::artifacts("rust-pending-ownership");
    let positions = support::window_positions();
    let mut a_config = support::config(&path.join("a"));
    let mut b_config = support::config(&path.join("b"));
    a_config.launch.arguments.extend([
        "--window-position".into(),
        format!("{},{}", positions[0].0, positions[0].1),
    ]);
    b_config.launch.arguments.extend([
        "--window-position".into(),
        format!("{},{}", positions[1].0, positions[1].1),
    ]);
    let mut a = Session::start(a_config).unwrap();
    let mut b = Session::start(b_config).unwrap();
    let request = warp::Start {
        ticks: 2,
        pace: Some(warp::Pace::AsFastAsPossible),
    };
    let mut a_pending = a.send(request.clone()).unwrap();
    let mut b_pending = b.send(warp::Stop {}).unwrap();
    let (a_id, b_id) = (a_pending.request_id(), b_pending.request_id());
    // Same numeric ID does not grant ownership. The output types differ too.
    assert_eq!(a_id, b_id);
    assert_eq!(a_pending.command(), &Command::Start(request));
    assert!(matches!(
        b.try_receive(&mut a_pending),
        Err(Error::InvalidPending { .. })
    ));
    assert!(matches!(
        a.try_receive(&mut b_pending),
        Err(Error::InvalidPending { .. })
    ));
    let a_result: warp::Completion = support::take_once(&mut a, &mut a_pending);
    // The other public receive entry point consumes its concrete Pending.
    let b_result: warp::Stopped = b.receive(b_pending).unwrap();
    assert_eq!(a_result.requested_ticks, 2);
    assert_eq!(a_result.executed_ticks, 2);
    assert_eq!(a_result.outcome, warp::Outcome::Completed);
    assert!(!b_result.was_running);
    assert!(matches!(
        a.try_receive(&mut a_pending),
        Err(Error::InvalidPending { .. })
    ));
    assert!(matches!(
        a.receive(a_pending),
        Err(Error::InvalidPending { .. })
    ));
    std::fs::write(
        path.join("outcomes.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "a":support::json(&a_result), "b":support::json(&b_result),
            "a_id":a_id.as_u64(), "b_id":b_id.as_u64()
        }))
        .unwrap(),
    )
    .unwrap();
    a.shutdown().unwrap();
    support::ended(&mut a);
    b.shutdown().unwrap();
    support::ended(&mut b);
}
