//! Public API transport only. Python owns every explicit Start/Shutdown/Report/Submit.
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    io::{self, BufRead, Write},
    path::PathBuf,
};
use woodpecker::{
    report::{self, Failure, Report, provider},
    session::{self, Session},
};
#[derive(Deserialize)]
struct File {
    launch: session::launch::Config,
    tick: woodpecker::command::tick::Config,
    startup: session::startup::Config,
    report: report::Config,
}
fn main() {
    // No session/network action is possible without both technical guards.
    let repo = std::env::var("WOODPECKER_TEST_GITHUB_REPO").unwrap_or_default();
    if std::env::var("WOODPECKER_TEST_RENDERED").as_deref() != Ok("1")
        || std::env::var("WOODPECKER_TEST_GITHUB_CLEANUP").as_deref() != Ok("close")
        || repo.is_empty()
        || std::env::var("GH_REPO").as_deref() != Ok(repo.as_str())
    {
        eprintln!("Separate graphics/GitHub repository/close approval required");
        std::process::exit(2);
    }
    let mut session: Option<Session> = None;
    let mut report: Option<Report> = None;
    let mut closed = false;
    for line in io::stdin().lock().lines() {
        let request: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let action = request["action"].as_str().unwrap();
        let result: Result<Value, String> = (|| match action {
            "start" => {
                if session.is_some() {
                    return Err("already started".into());
                }
                let config: File =
                    serde_json::from_value(request["config"].clone()).map_err(|e| e.to_string())?;
                if !matches!(config.report.provider, provider::Config::Github) {
                    return Err("GitHub provider required".into());
                }
                let value = Session::start(session::Config {
                    launch: config.launch,
                    tick: config.tick,
                    startup: config.startup,
                    report: config.report,
                    artifact_dir: PathBuf::from(
                        request["artifact_dir"].as_str().ok_or("artifact_dir")?,
                    ),
                })
                .map_err(|e| e.to_string())?;
                let output = serde_json::to_value(value.capabilities()).unwrap();
                session = Some(value);
                Ok(output)
            }
            "shutdown" => {
                session
                    .as_mut()
                    .ok_or("not started")?
                    .shutdown()
                    .map_err(|e| e.to_string())?;
                closed = true;
                Ok(Value::Null)
            }
            "report" => {
                if !closed {
                    return Err("shut the zero-tick game down first".into());
                }
                let value = Report::create(
                    Failure::panic(
                        Some(request["message"].as_str().ok_or("message")?.into()),
                        None,
                        None,
                    ),
                    session.as_ref().ok_or("not started")?,
                );
                let output = json!({"title":value.title(),"body":value.to_markdown(),"signature":value.signature().as_str()});
                report = Some(value);
                Ok(output)
            }
            "submit" => report::submit(
                report.as_ref().ok_or("no report")?,
                session.as_ref().ok_or("no session")?,
            )
            .map(|v| serde_json::to_value(v).unwrap())
            .map_err(|e| e.to_string()),
            "quit" => {
                drop(session.take());
                Ok(Value::Null)
            }
            _ => Err("unsupported explicit operation".into()),
        })();
        println!(
            "{}",
            match result {
                Ok(value) => json!({"ok":true,"value":value}),
                Err(error) => json!({"ok":false,"error":error}),
            }
        );
        io::stdout().flush().unwrap();
        if action == "quit" {
            break;
        }
    }
}
