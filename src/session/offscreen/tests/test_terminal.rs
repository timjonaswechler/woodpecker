use super::*;

#[test]
fn offscreen_terminal_cut_rejects_deadline_stop_fatal_and_render_unwind() {
    use bevy::shader::ShaderCacheError;
    assert!(
        pipeline_failure(&CachedPipelineState::Err(
            ShaderCacheError::ShaderNotLoaded(Handle::<Shader>::default().id())
        ))
        .is_none()
    );
    assert!(
        pipeline_failure(&CachedPipelineState::Err(
            ShaderCacheError::ShaderImportNotYetAvailable
        ))
        .is_none()
    );
    assert_eq!(
        pipeline_failure(&CachedPipelineState::Err(
            ShaderCacheError::ProcessShaderError("permanent shader failure".into())
        ))
        .unwrap()
        .code,
        "render_pipeline_failed"
    );
    let mut unsupported = App::new();
    unsupported.insert_resource(OffscreenTarget(Handle::default()));
    install(&mut unsupported);
    assert_eq!(
        unsupported
            .world()
            .resource::<Status>()
            .error()
            .unwrap()
            .code,
        "render_profile_unsupported"
    );
    assert!(!unsupported.world().contains_resource::<Reader>());
    let status = Status::default();
    assert!(status.guard(Instant::now()).check().is_ok());
    assert_eq!(
        status
            .guard(Instant::now() - Duration::from_secs(31))
            .check()
            .unwrap_err()
            .code,
        "screenshot_failed"
    );
    status.stop();
    assert_eq!(
        status.guard(Instant::now()).check().unwrap_err().code,
        "screenshot_cancelled"
    );
    status.fail(Diagnostic::new("render_device_lost", "first"));
    status.fail(Diagnostic::new("render_internal", "second"));
    let error = status.guard(Instant::now()).check().unwrap_err();
    assert_eq!(
        (error.code.as_str(), error.message.as_str()),
        ("render_device_lost", "first")
    );
    let panic_status = Status::default();
    protect(Some(panic_status.clone()), || {
        panic!("GPU operation failed before notification")
    });
    assert_eq!(panic_status.error().unwrap().code, "render_panicked");
    let error = crate::session::Error::from(panic_status.error().unwrap());
    assert!(matches!(error, crate::session::Error::Render { .. }));
    let native = std::panic::catch_unwind(|| protect(None, || panic!("native behavior unchanged")));
    assert!(native.is_err());
}
