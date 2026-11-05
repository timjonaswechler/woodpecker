use super::*;

#[test]
fn terminal_check_after_staging_preserves_existing_file_and_cleans_temporary() {
    let path = crate::test_support::artifacts("screenshot-final-check");
    let root = Dir::open_ambient_dir(&path, cap_std::ambient_authority()).unwrap();
    for code in [
        "render_device_lost",
        "screenshot_cancelled",
        "screenshot_failed",
    ] {
        std::fs::write(path.join("old.png"), b"previous final bytes").unwrap();
        let error = write_checked(&root, "old.png", b"fully encoded staged bytes", || {
            // Called after the actual temporary-file write/sync, before rename.
            assert!(std::fs::read_dir(&path).unwrap().count() > 1);
            Err(Diagnostic::new(code, "terminal barrier"))
        })
        .unwrap_err();
        assert_eq!(error.code, code);
        assert_eq!(
            std::fs::read(path.join("old.png")).unwrap(),
            b"previous final bytes"
        );
        assert_eq!(std::fs::read_dir(&path).unwrap().count(), 1);
    }
}
