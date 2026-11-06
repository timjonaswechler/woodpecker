use super::*;

#[test]
fn native_ready_remains_unchanged_and_render_failure_has_no_request_id() {
    assert_eq!(
        serde_json::to_value(Capabilities::default()).unwrap(),
        serde_json::json!({"screenshot":false})
    );
    let capabilities = Capabilities {
        screenshot: true,
        offscreen: true,
        ..Default::default()
    };
    let value = serde_json::to_value(&capabilities).unwrap();
    assert_eq!(
        value,
        serde_json::json!({"screenshot":true,"offscreen":true})
    );
    assert!(
        serde_json::from_value::<Capabilities>(value)
            .unwrap()
            .offscreen
    );
    let failure = Message::RenderFailed {
        error: Diagnostic::new("render_device_lost", "owned device"),
    };
    let decoded: Message = serde_json::from_str(&serde_json::to_string(&failure).unwrap()).unwrap();
    assert!(decoded.id().is_none());
    assert!(matches!(decoded, Message::RenderFailed { .. }));
}
