use super::*;
use std::os::unix::fs::symlink;

#[test]
fn screenshot_path_replacement_never_reaches_the_outside_sentinel() {
    let path = crate::test_support::artifacts("screenshot-path-swap");
    let inside = path.join("root");
    let outside = path.join("outside");
    std::fs::create_dir_all(inside.join("shots")).unwrap();
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(outside.join("protected.png"), b"outside sentinel").unwrap();
    std::fs::write(inside.join("shots/protected.png"), b"inside original").unwrap();
    let root = Dir::open_ambient_dir(&inside, cap_std::ambient_authority()).unwrap();
    // The actual acceptance-time check precedes the deliberate directory exchange.
    assert!(prepare(&root, "shots/protected.png").unwrap());
    std::fs::rename(inside.join("shots"), inside.join("retired")).unwrap();
    symlink(&outside, inside.join("shots")).unwrap();
    let error = write(&root, "shots/protected.png", b"new encoded bytes").unwrap_err();
    assert_eq!(error.code, "invalid_screenshot_path");
    assert_eq!(
        std::fs::read(outside.join("protected.png")).unwrap(),
        b"outside sentinel"
    );
    assert_eq!(
        std::fs::read(inside.join("retired/protected.png")).unwrap(),
        b"inside original"
    );
    assert_eq!(std::fs::read_dir(&outside).unwrap().count(), 1);

    // Exchange just the final entry after a successful check.
    std::fs::create_dir(inside.join("final")).unwrap();
    std::fs::write(inside.join("final/protected.png"), b"final original").unwrap();
    assert!(prepare(&root, "final/protected.png").unwrap());
    std::fs::remove_file(inside.join("final/protected.png")).unwrap();
    symlink(
        outside.join("protected.png"),
        inside.join("final/protected.png"),
    )
    .unwrap();
    assert_eq!(
        write(&root, "final/protected.png", b"replacement")
            .unwrap_err()
            .code,
        "invalid_screenshot_path"
    );
    assert!(
        std::fs::symlink_metadata(inside.join("final/protected.png"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        std::fs::read(outside.join("protected.png")).unwrap(),
        b"outside sentinel"
    );

    // Do not mistake confinement for a blanket symlink ban. In-root links are valid;
    // atomic replacement replaces the link itself, not the referenced file.
    std::fs::write(inside.join("final/source.png"), b"legitimate source").unwrap();
    symlink("source.png", inside.join("final/legitimate.png")).unwrap();
    assert!(prepare(&root, "final/legitimate.png").unwrap());
    assert!(write(&root, "final/legitimate.png", b"fresh encoded payload").unwrap());
    assert_eq!(
        std::fs::read(inside.join("final/legitimate.png")).unwrap(),
        b"fresh encoded payload"
    );
    assert!(
        !std::fs::symlink_metadata(inside.join("final/legitimate.png"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        std::fs::read(inside.join("final/source.png")).unwrap(),
        b"legitimate source"
    );
    assert!(!write(&root, "final/fresh.png", b"fresh file").unwrap());
    assert_eq!(
        std::fs::read(inside.join("final/fresh.png")).unwrap(),
        b"fresh file"
    );
    assert_eq!(std::fs::read_dir(&outside).unwrap().count(), 1);
    assert!(
        std::fs::read_dir(inside.join("final")).unwrap().all(|e| !e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp"))
    );
}
