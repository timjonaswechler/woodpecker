mod support;
use woodpecker::{
    report::{Failure, Location, Report},
    session::Session,
};

#[test]
fn report_signature_contract() {
    support::rendered();
    let path = support::artifacts("rust-report-signature");
    let config = support::config(&path);
    let project = config
        .launch
        .manifest_path
        .parent()
        .unwrap()
        .canonicalize()
        .unwrap();
    let mut session = Session::start(config).unwrap();
    let panic = |message: Option<&str>| {
        Report::create(
            Failure::panic(message.map(str::to_string), None, None),
            &session,
        )
    };
    assert_eq!(
        panic(Some("stellar catalog invariant violated"))
            .signature()
            .as_str(),
        "v1:sha256:e539c36fcbeb8357ba855f705c08274f29e82d866288dffaa1406607eba9302c"
    );
    assert_eq!(
        panic(None).signature().as_str(),
        "v1:sha256:f7ad1a8211ca86c794de8c16b456c16006393c974b89e5aa11908ff1f35336a6"
    );
    let tracing = Report::create(
        Failure::tracing_error("stellar catalog invariant violated".into(), None, None),
        &session,
    );
    assert_eq!(
        tracing.signature().as_str(),
        "v1:sha256:86a70a6857c6f193019c24bf0d04b01bde9f127e28b5c45c8f021de0c9165e46"
    );
    let exit = Report::create(Failure::process_exit("exit status: 17".into()), &session);
    assert_eq!(
        exit.signature().as_str(),
        "v1:sha256:7ca80a848b0913a8eb5ab37f815efe34cd2c4bcdc46ad2fd1870c08902c3f240"
    );
    assert_eq!(
        exit.signature(),
        Report::create(Failure::process_exit("exit status: 0".into()), &session).signature()
    );
    let normalized = panic(Some("<project>/src/file <address> <timestamp>"));
    let noisy = format!(
        "\x1b[31m  {}/src/file 0xABCDEF12 2024-02-29T12:34:56Z\r\n\x1b[0m",
        project.display()
    );
    assert_eq!(panic(Some(&noisy)).signature(), normalized.signature());
    assert_ne!(
        panic(Some("HTTP 404")).signature(),
        panic(Some("HTTP 500")).signature()
    );
    assert_ne!(
        panic(Some("two spaces")).signature(),
        panic(Some("two  spaces")).signature()
    );
    let diagnostic = Report::create(
        Failure::panic(
            Some("stellar catalog invariant violated".into()),
            Some(Location {
                file: "changed.rs".into(),
                line: 99,
                column: Some(3),
            }),
            Some("different backtrace".into()),
        ),
        &session,
    );
    assert_eq!(
        diagnostic.signature(),
        panic(Some("stellar catalog invariant violated")).signature()
    );
    let message = "\r\n\x1b[31m  *Title* <tag> Grüße  \x1b[0m\r\nfull ``` diagnostic";
    let rendered = Report::create(
        Failure::panic(Some(message.into()), None, Some("trace\n````````\n".into())),
        &session,
    );
    assert_eq!(rendered.title(), "*Title* <tag> Grüße");
    assert_eq!(rendered.failure().message(), Some(message));
    let markdown = rendered.to_markdown();
    assert!(markdown.starts_with("# \\*Title\\* \\<tag\\> Grüße\n"));
    // Consume the complete document in order. Each block must close with its
    // exact delimiter; hashes and opening fences alone cannot prove payloads.
    fn block<'a>(rest: &mut &'a str, heading: &str, fence: &str, language: &str) -> &'a str {
        let prefix = format!("{heading}\n\n{fence}{language}\n");
        let body = rest
            .strip_prefix(&prefix)
            .expect("Missing or reordered section");
        let ending = format!("{fence}\n");
        let (payload, tail) = body.split_once(&ending).expect("Unclosed diagnostic fence");
        assert!(
            payload.ends_with('\n'),
            "Closing fence must begin on its own line"
        );
        *rest = if heading == "## Commands" {
            tail
        } else {
            tail.strip_prefix('\n').expect("Missing section separator")
        };
        payload
    }
    let header = format!(
        "# \\*Title\\* \\<tag\\> Grüße\n\n<!-- bug_hunter-signature: {} -->\n\n## Failure\n\nMissing diagnostic values are unavailable; JSON `null` denotes unavailable data.\n\n",
        rendered.signature().as_str()
    );
    let mut rest = markdown
        .strip_prefix(&header)
        .expect("Incorrect report header");
    assert_eq!(
        block(&mut rest, "### Message", "````", "text"),
        "\n\x1b[31m  *Title* <tag> Grüße  \x1b[0m\nfull ``` diagnostic\n"
    );
    let origin: serde_json::Value =
        serde_json::from_str(block(&mut rest, "### Origin", "```", "json")).unwrap();
    assert_eq!(
        origin,
        serde_json::json!({"kind":"panic", "location":null, "backtrace_status":"captured"})
    );
    assert_eq!(
        block(&mut rest, "### Backtrace", "`````````", "text"),
        "trace\n````````\n"
    );
    let application: serde_json::Value =
        serde_json::from_str(block(&mut rest, "## Application", "```", "json")).unwrap();
    assert_eq!(application, support::json(&rendered.context().application));
    let environment: serde_json::Value =
        serde_json::from_str(block(&mut rest, "## Environment", "```", "json")).unwrap();
    let context = rendered.context();
    assert_eq!(
        environment,
        serde_json::json!({
            "woodpecker_version": env!("CARGO_PKG_VERSION"), "protocol_version": 3,
            "capabilities": context.capabilities, "tick": context.tick,
            "platform": {"os": std::env::consts::OS, "arch": std::env::consts::ARCH},
            "toolchain": context.toolchain,
        })
    );
    let commands: serde_json::Value =
        serde_json::from_str(block(&mut rest, "## Commands", "```", "json")).unwrap();
    assert_eq!(commands, serde_json::json!([]));
    assert_eq!(
        rest, "",
        "Unexpected trailing or duplicated Markdown content"
    );
    assert!(!markdown.contains('\r'));
    assert!(markdown.ends_with('\n') && !markdown.ends_with("\n\n"));
    assert_eq!(
        markdown
            .matches(&format!(
                "<!-- bug_hunter-signature: {} -->",
                rendered.signature().as_str()
            ))
            .count(),
        1
    );
    assert_eq!(
        panic(Some(&"🦉".repeat(121))).title(),
        format!("{}...", "🦉".repeat(117))
    );
    assert_eq!(panic(None).title(), "panic");
    std::fs::write(path.join("report.md"), markdown).unwrap();
    session.shutdown().unwrap();
    support::ended(&mut session);
}
