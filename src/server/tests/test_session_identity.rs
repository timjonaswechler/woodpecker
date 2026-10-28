use super::*;
use std::os::unix::fs::symlink;

#[test]
fn session_identity_collisions_preserve_roots_and_reject_ambiguous_prefixes() {
    let path = crate::test_support::artifacts("session-identity");
    let root = path.join("sessions");
    std::fs::create_dir(&root).unwrap();
    let outside = path.join("outside");
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(outside.join("sentinel"), b"outside original").unwrap();
    let ids: Vec<_> = (1..=4).map(|n| format!("deadbeef{n:024x}")).collect();
    std::fs::create_dir(root.join(&ids[0])).unwrap();
    std::fs::write(root.join(&ids[0]).join("sentinel"), b"existing root").unwrap();
    symlink(&outside, root.join(&ids[1])).unwrap();
    let retained = entry::Entry::new(ids[2].clone(), root.join(&ids[2]), 4096);
    let mut directory = Directory::default();
    directory.entries.insert(ids[2].clone(), retained.clone());
    let inner = Inner {
        directory: Mutex::new(directory),
        root: root.clone(),
        timeout: Duration::from_secs(2),
        activity_bytes: 4096,
    };
    let mut config = crate::test_support::config(&path);
    // A missing owned manifest exercises identity/launch failure without a game.
    config.launch.manifest_path = path.join("missing/Cargo.toml");
    let create = Create {
        launch: config.launch,
        tick: config.tick,
        startup: config.startup,
        report: config.report,
    };
    let mut candidates = ids.iter();
    let mut calls = 0;
    let created = inner
        .create_using(create, || {
            calls += 1;
            Ok(candidates
                .next()
                .expect("bounded candidates exhausted")
                .clone())
        })
        .unwrap();
    assert_eq!(calls, 4);
    assert_eq!(created.id, ids[3]);
    assert_eq!(created.artifact_dir, root.join(&ids[3]));
    assert_eq!(inner.list().len(), 2);
    assert!(Arc::ptr_eq(&inner.select(&ids[2]).unwrap(), &retained));
    assert_eq!(
        inner.select(&ids[3].to_uppercase()).unwrap().detail().id,
        ids[3]
    );
    let error = match inner.select("deadbeef") {
        Err(e) => e,
        Ok(_) => panic!("Ambiguous prefix selected a session"),
    };
    assert_eq!(error.code(), "ambiguous_session_id");
    assert_eq!(
        std::fs::read(root.join(&ids[0]).join("sentinel")).unwrap(),
        b"existing root"
    );
    assert_eq!(
        std::fs::read(outside.join("sentinel")).unwrap(),
        b"outside original"
    );
    assert_eq!(std::fs::read_link(root.join(&ids[1])).unwrap(), outside);
    let created = inner.select(&ids[3]).unwrap();
    crate::test_support::wait(|| created.done.load(Ordering::Acquire));
    assert_eq!(created.detail().state, Lifecycle::Failed);
    assert_eq!(
        std::fs::read(root.join(&ids[0]).join("sentinel")).unwrap(),
        b"existing root"
    );
    assert_eq!(
        std::fs::read(outside.join("sentinel")).unwrap(),
        b"outside original"
    );
}
