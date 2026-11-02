mod support;
use std::fs;
use woodpecker::{
    command::input::keyboard,
    report::{Failure, Report},
    session::Session,
};

#[test]
fn report_snapshot_contract() {
    support::rendered();
    let path = support::artifacts("rust-report-snapshot");
    let project = path.join("project");
    fs::create_dir(&project).unwrap();
    support::copy_tree(&support::root().join("test_app/src"), &project.join("src"));
    support::copy_tree(
        &support::root().join("test_app/asset"),
        &project.join("asset"),
    );
    let mut manifest: toml::Value =
        toml::from_str(&fs::read_to_string(support::root().join("test_app/Cargo.toml")).unwrap())
            .unwrap();
    manifest["dependencies"]["woodpecker"]["path"] =
        toml::Value::String(support::root().to_str().unwrap().into());
    manifest["package"]["version"] = toml::Value::String("0.56.1".into());
    let tables = manifest.as_table_mut().unwrap();
    tables.insert("workspace".into(), toml::Value::Table(Default::default()));
    let workspace: toml::Value =
        toml::from_str(&fs::read_to_string(support::root().join("Cargo.toml")).unwrap()).unwrap();
    tables.insert("profile".into(), workspace["profile"].clone());
    fs::write(
        project.join("Cargo.toml"),
        toml::to_string(&manifest).unwrap(),
    )
    .unwrap();
    fs::copy(
        support::root().join("Cargo.lock"),
        project.join("Cargo.lock"),
    )
    .unwrap();
    // Resolve the standalone fixture's own lock before the clean Git baseline.
    // --no-deps alone does not guarantee that the later automation build is locked.
    let metadata = std::process::Command::new("cargo")
        .args([
            "metadata",
            "--format-version",
            "1",
            "--features",
            "automation",
            "--manifest-path",
        ])
        .arg(project.join("Cargo.toml"))
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(
        metadata.status.success(),
        "{}",
        String::from_utf8_lossy(&metadata.stderr)
    );
    fs::write(path.join("fixture-metadata.json"), metadata.stdout).unwrap();
    support::git(&project, &["init", "-q"]);
    support::git(&project, &["add", "."]);
    support::git(&project, &["commit", "-qm", "Acceptance fixture baseline"]);
    let original_head = support::git(&project, &["rev-parse", "HEAD"]);
    let _environment = support::Environment::new([(
        "CARGO_TARGET_DIR".into(),
        support::root().join("target").display().to_string(),
    )]);
    let mut config = support::config(&path);
    config.launch.manifest_path = project.join("Cargo.toml");
    let index = config
        .launch
        .arguments
        .iter()
        .position(|s| s == "--settings-seed")
        .unwrap();
    config.launch.arguments[index + 1] = support::root()
        .join("tests/settings.json")
        .display()
        .to_string();
    // Expected metadata comes from the chosen fixture/configuration, not a
    // second report snapshot (which could repeat the same wrong value).
    let expected_target = support::json(&config.launch.target);
    let expected_features = config.launch.features.clone();
    let expected_arguments = config.launch.arguments.clone();
    let expected_tick = support::json(&config.tick);
    let tool_version = |variable: &str, fallback: &str| {
        let output = std::process::Command::new(
            std::env::var_os(variable).unwrap_or_else(|| fallback.into()),
        )
        .arg("--version")
        .current_dir(&project)
        .output()
        .ok()?;
        if !output.status.success() {
            return None;
        }
        Some(String::from_utf8(output.stdout).ok()?.trim().to_owned())
    };
    let expected_toolchain = serde_json::json!({
        "cargo": tool_version("CARGO", "cargo"),
        "rustc": tool_version("RUSTC", "rustc"),
    });
    let mut session = Session::start(config).unwrap();
    let pending = session
        .send(keyboard::Press {
            key: keyboard::Key::Unknown("shift_left".into()),
        })
        .unwrap();
    support::receive(&mut session, pending);
    let report = Report::create(
        Failure::panic(Some("snapshot acceptance".into()), None, None),
        &session,
    );
    let original = support::json(&report);
    let markdown = report.to_markdown();
    let context = &original["context"];
    assert_eq!(context["application"]["package"], "test_app");
    assert_eq!(context["application"]["version"], "0.56.1");
    assert_eq!(context["application"]["source"]["commit"], original_head);
    assert_eq!(context["application"]["source"]["dirty"], false);
    assert_eq!(context["application"]["target"], expected_target);
    assert_eq!(
        context["application"]["features"],
        support::json(&expected_features)
    );
    assert_eq!(
        context["application"]["arguments"],
        support::json(&expected_arguments)
    );
    assert_eq!(context["woodpecker_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(context["protocol_version"], 3);
    assert_eq!(context["tick"], expected_tick);
    assert_eq!(
        context["capabilities"],
        serde_json::json!({"screenshot": true, "fixed_clock": true})
    );
    assert_eq!(
        context["toolchain"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["cargo", "rustc"]
    );
    for tool in ["cargo", "rustc"] {
        // These optional, bounded observations may legally be unavailable.
        // When present, compare the value with an independent tool invocation.
        if !context["toolchain"][tool].is_null() {
            assert_eq!(
                context["toolchain"][tool], expected_toolchain[tool],
                "{tool}"
            );
        }
    }
    assert_eq!(context["platform"]["os"], std::env::consts::OS);
    assert_eq!(context["platform"]["arch"], std::env::consts::ARCH);
    assert_eq!(context["commands"].as_array().unwrap().len(), 1);
    assert_eq!(
        context["commands"][0]["command"]["command"],
        "input.keyboard.press"
    );
    assert_eq!(
        context["commands"][0],
        serde_json::json!({
            "command": {"command":"input.keyboard.press", "arguments":{"key":"shift_left"}},
            "outcome": {"status":"completed", "output":null}
        })
    );
    let fields: std::collections::BTreeSet<_> = context
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        fields,
        [
            "application",
            "woodpecker_version",
            "protocol_version",
            "capabilities",
            "tick",
            "platform",
            "toolchain",
            "commands"
        ]
        .into_iter()
        .collect()
    );
    assert_eq!(context["application"].as_object().unwrap().len(), 6);
    assert_eq!(context["commands"][0].as_object().unwrap().len(), 2);
    let pending = session
        .send(keyboard::Release {
            key: keyboard::Key::Unknown("shift_left".into()),
        })
        .unwrap();
    support::receive(&mut session, pending);
    fs::write(project.join("later.txt"), "owned later state").unwrap();
    support::git(&project, &["checkout", "-qb", "later-state"]);
    support::git(&project, &["add", "."]);
    support::git(
        &project,
        &["commit", "-qm", "Acceptance fixture later state"],
    );
    assert_ne!(
        support::git(&project, &["rev-parse", "HEAD"]),
        original_head
    );
    let later = Report::create(report.failure().clone(), &session);
    assert_eq!(later.context().commands.len(), 2);
    assert_eq!(support::json(&report), original);
    assert_eq!(report.to_markdown(), markdown);
    assert_eq!(
        later.context().application.source.as_ref().unwrap().commit,
        original_head
    );
    session.shutdown().unwrap();
    support::ended(&mut session);
    fs::write(
        path.join("snapshot.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    drop(session);
}
