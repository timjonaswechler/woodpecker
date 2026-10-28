mod support;
use std::fs;
use woodpecker::{
    report::{
        self, Failure, Report,
        provider::{self, Outcome, Reference},
    },
    session::Session,
};

#[test]
fn github_provider_contract() {
    support::rendered();
    let path = support::artifacts("rust-github-provider");
    let gh = path.join("gh");
    let _environment = support::Environment::new(support::gh_environment(&gh));
    let mut config = support::config(&path);
    config.report.provider = provider::Config::Github;
    let root = config.artifact_dir.clone();
    let mut session = Session::start(config).unwrap();
    session.shutdown().unwrap();
    support::ended(&mut session);
    for mode in [
        "create",
        "duplicate-open",
        "duplicate-closed",
        "late-invalid",
        "search-failed",
        "search-invalid",
        "publish-failed",
        "publish-invalid",
        "both-failed",
    ] {
        let report = Report::create(
            Failure::panic(Some(format!("provider {mode}")), None, None),
            &session,
        );
        fs::write(
            gh.join("mode.json"),
            serde_json::json!({"mode":mode,"signature":report.signature().as_str()}).to_string(),
        )
        .unwrap();
        let _ = fs::remove_file(gh.join("calls.jsonl"));
        if mode == "both-failed" {
            if root.join("reports").exists() {
                fs::rename(root.join("reports"), root.join("previous-reports")).unwrap();
            }
            fs::write(root.join("reports"), b"owned blocking file").unwrap();
        }
        let outcome = report::submit(&report, &session);
        match mode {
            "create" => assert_eq!(
                outcome.unwrap(),
                Outcome::Created {
                    reference: Reference::Issue {
                        identifier: "12".into(),
                        url: "https://example.invalid/issues/12".into()
                    }
                }
            ),
            "duplicate-open" | "duplicate-closed" => assert_eq!(
                outcome.unwrap(),
                Outcome::Existing {
                    reference: Reference::Issue {
                        identifier: "42".into(),
                        url: "https://example.invalid/issues/42".into()
                    }
                }
            ),
            "both-failed" => {
                let error = outcome.unwrap_err();
                assert!(matches!(error, report::Error::FallbackFailed { .. }));
                assert!(error.to_string().contains("local fallback failed"));
                let data = support::json(&error);
                assert_eq!(data["error"]["provider"]["error"]["operation"], "search");
                assert_eq!(
                    fs::read(root.join("reports")).unwrap(),
                    b"owned blocking file"
                );
                assert!(data["error"]["local"].is_object());
            }
            _ => {
                let value = outcome.unwrap();
                let data = support::json(&value);
                assert_eq!(data["kind"], "fallback");
                assert_eq!(
                    data["provider_error"]["error"]["operation"],
                    if mode.starts_with("publish") {
                        "publish"
                    } else {
                        "search"
                    }
                );
                assert_eq!(
                    data["provider_error"]["error"]["kind"],
                    if mode.ends_with("failed") {
                        "command_failed"
                    } else {
                        "invalid_response"
                    }
                );
                let Outcome::Fallback { reference, .. } = value else {
                    unreachable!()
                };
                assert_eq!(
                    fs::read_to_string(root.join(reference.path)).unwrap(),
                    report.to_markdown()
                );
            }
        }
        let calls: Vec<serde_json::Value> = fs::read_to_string(gh.join("calls.jsonl"))
            .unwrap()
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect();
        assert!(
            calls
                .iter()
                .all(|v| v["cwd"] == support::root().join("test_app").to_str().unwrap())
        );
        assert!(calls.iter().all(|v| v["prompt"] == "1"));
        assert_eq!(
            calls[0]["arguments"],
            serde_json::json!([
                "api",
                "repos/{owner}/{repo}/issues?state=all&per_page=100",
                "--method",
                "GET",
                "--paginate"
            ])
        );
        if mode == "create" || mode.starts_with("publish") {
            assert_eq!(calls.len(), 2);
            assert_eq!(
                calls[1]["arguments"],
                serde_json::json!([
                    "api",
                    "repos/{owner}/{repo}/issues",
                    "--method",
                    "POST",
                    "--input",
                    "-"
                ])
            );
            assert_eq!(
                calls[1]["body"],
                serde_json::json!({"title":report.title(),"body":report.to_markdown()})
            );
        } else {
            assert_eq!(calls.len(), 1);
        }
        if matches!(mode, "create" | "duplicate-open" | "duplicate-closed") {
            assert!(
                !root.join("reports").exists(),
                "Successful remote output must not write a local report"
            );
        }
    }
    drop(session); // Join SDK threads before restoring this test process's environment.
}
