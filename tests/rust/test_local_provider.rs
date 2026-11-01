mod support;
use std::{
    fs,
    sync::{Arc, Barrier},
};
use woodpecker::{
    report::{
        self, Failure, Report,
        provider::{Outcome, Reference, local},
    },
    session::Session,
};

#[test]
fn local_provider_contract() {
    support::rendered();
    let path = support::artifacts("rust-local-provider");
    let config = support::config(&path);
    let root = config.artifact_dir.clone();
    for output in [
        "",
        "/absolute",
        "../escape",
        "reports/./nested",
        "reports/../escape",
    ] {
        let mut invalid = config.clone();
        invalid.report.output = output.into();
        assert_eq!(invalid.validate().unwrap_err().code(), "invalid_config");
    }
    let mut session = Session::start(config).unwrap();
    let report = Report::create(
        Failure::panic(Some("local provider acceptance".into()), None, None),
        &session,
    );
    session.shutdown().unwrap();
    support::ended(&mut session);
    let barrier = Arc::new(Barrier::new(6));
    let outcomes = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..6)
            .map(|_| {
                let barrier = barrier.clone();
                let session = &session;
                let report = &report;
                scope.spawn(move || {
                    barrier.wait();
                    report::submit(report, session).unwrap()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(
        outcomes
            .iter()
            .filter(|v| matches!(v, Outcome::Created { .. }))
            .count(),
        1
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|v| matches!(v, Outcome::Existing { .. }))
            .count(),
        5
    );
    let relative = support::file(outcomes[0].clone());
    assert_eq!(
        relative,
        std::path::PathBuf::from("reports").join(format!(
            "{}.md",
            report.signature().as_str().replace(':', "-")
        ))
    );
    assert!(!relative.is_absolute());
    assert!(
        relative
            .components()
            .all(|p| matches!(p, std::path::Component::Normal(_)))
    );
    assert_eq!(
        fs::read_to_string(root.join(&relative)).unwrap(),
        report.to_markdown()
    );
    let marker = format!(
        "<!-- bug_hunter-signature: {} -->",
        report.signature().as_str()
    );
    let annotated = format!("{marker}\nOwned annotation, preserved verbatim.\n");
    fs::write(root.join(&relative), &annotated).unwrap();
    assert!(matches!(
        report::submit(&report, &session).unwrap(),
        Outcome::Existing { .. }
    ));
    assert_eq!(fs::read_to_string(root.join(&relative)).unwrap(), annotated);
    for content in [
        "foreign content".to_string(),
        format!("```text\n{marker}\n```\n"),
        format!("{marker}\n{marker}\n"),
        format!("{marker} suffix\n"),
    ] {
        fs::write(root.join(&relative), &content).unwrap();
        assert!(matches!(
            report::submit(&report, &session),
            Err(report::Error::Local(local::Error::Conflict { .. }))
        ));
        assert_eq!(fs::read_to_string(root.join(&relative)).unwrap(), content);
    }
    let outside = path.join("outside");
    fs::create_dir(&outside).unwrap();
    fs::rename(root.join("reports"), root.join("archived-reports")).unwrap();
    std::os::unix::fs::symlink(&outside, root.join("reports")).unwrap();
    assert!(matches!(
        report::submit(&report, &session),
        Err(report::Error::Local(local::Error::InvalidPath { .. }))
    ));
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
    fs::remove_file(root.join("reports")).unwrap();
    fs::create_dir(root.join("reports")).unwrap();
    let sentinel = outside.join("sentinel");
    fs::write(&sentinel, b"owned sentinel").unwrap();
    std::os::unix::fs::symlink(&sentinel, root.join(&relative)).unwrap();
    assert!(matches!(
        report::submit(&report, &session),
        Err(report::Error::Local(local::Error::InvalidPath { .. }))
    ));
    assert_eq!(fs::read(&sentinel).unwrap(), b"owned sentinel");
    assert!(matches!(
        outcomes[0],
        Outcome::Created {
            reference: Reference::File(_)
        } | Outcome::Existing {
            reference: Reference::File(_)
        }
    ));
    fs::write(
        path.join("outcomes.json"),
        serde_json::to_vec_pretty(&outcomes).unwrap(),
    )
    .unwrap();
}
