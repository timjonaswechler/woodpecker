//! Artifact/configuration/process environment and pure pending/event observation only.
#![allow(dead_code)]
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use woodpecker::{report, session};

pub fn rendered() {
    assert_eq!(
        std::env::var("WOODPECKER_TEST_RENDERED").as_deref(),
        Ok("1"),
        "One real manual-startup window, zero ticks. Fresh permission required: WOODPECKER_TEST_RENDERED=1"
    );
}
pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
pub fn artifacts(label: &str) -> PathBuf {
    let path = root().join("target/test-runs").join(format!(
        "{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    println!("Artefakte: {}", path.display());
    path
}
pub fn config(path: &Path) -> session::Config {
    #[derive(serde::Deserialize)]
    struct File {
        launch: session::launch::Config,
        tick: woodpecker::command::tick::Config,
        startup: session::startup::Config,
        report: report::Config,
    }
    let mut file: File =
        toml::from_str(&std::fs::read_to_string(root().join("tests/app.toml")).unwrap()).unwrap();
    file.launch.manifest_path = root().join(file.launch.manifest_path);
    session::Config {
        launch: file.launch,
        tick: file.tick,
        startup: file.startup,
        report: file.report,
        artifact_dir: path.join("session"),
    }
}
pub fn receive<T: serde::de::DeserializeOwned>(
    session: &mut session::Session,
    mut pending: session::Pending<T>,
) -> T {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(value) = session.try_receive(&mut pending).unwrap() {
            return value;
        }
        assert!(Instant::now() < until, "Pending deadline; no retry");
        std::thread::sleep(Duration::from_millis(5));
    }
}
/// Observe one terminal result without consuming the Pending handle itself.
pub fn take_once<T: serde::de::DeserializeOwned>(
    session: &mut session::Session,
    pending: &mut session::Pending<T>,
) -> T {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(value) = session.try_receive(pending).unwrap() {
            return value;
        }
        assert!(Instant::now() < until, "Pending deadline; no retry");
        std::thread::sleep(Duration::from_millis(5));
    }
}
pub fn window_positions() -> Vec<(i32, i32)> {
    let output = std::process::Command::new("python3").current_dir(root())
        .args(["-c", "import json; from tests._support import WINDOW_POSITIONS; print(json.dumps(WINDOW_POSITIONS))"])
        .output().unwrap();
    assert!(output.status.success(), "Pure window-position query failed");
    serde_json::from_slice(&output.stdout).unwrap()
}
pub fn progress(root: &Path, minimum: u64) -> u64 {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        if let Ok(bytes) = std::fs::read(root.join("simulation-ticks.json")) {
            let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            let ticks = value["ticks"].as_u64().unwrap();
            if ticks >= minimum {
                return ticks;
            }
        }
        assert!(
            Instant::now() < until,
            "Application counter deadline; no extra ticks"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}
pub struct OwnedProcess {
    pub pid: i32,
    pub group: i32,
}
pub fn owned_process(root: &Path) -> OwnedProcess {
    let until = Instant::now() + Duration::from_secs(15);
    let bytes = loop {
        if let Ok(bytes) = std::fs::read(root.join("process-probe.json")) {
            break bytes;
        }
        assert!(Instant::now() < until, "App PID probe missing");
        std::thread::sleep(Duration::from_millis(5));
    };
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let pid = i32::try_from(value["pid"].as_i64().unwrap()).unwrap();
    assert!(pid > 0);
    let mut current = pid;
    for _ in 0..16 {
        let output = std::process::Command::new("ps")
            .args(["-p", &current.to_string(), "-o", "ppid=", "-o", "command="])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "Owned process disappeared before observation"
        );
        let text = String::from_utf8(output.stdout).unwrap();
        let mut words = text.split_whitespace();
        let parent: i32 = words.next().unwrap().parse().unwrap();
        if current == pid {
            assert!(words.collect::<Vec<_>>().join(" ").contains("test_app"));
        }
        if parent == std::process::id() as i32 {
            // SAFETY: query only, PID was verified as a descendant of this test.
            let group = unsafe { libc::getpgid(pid) };
            assert!(group > 0);
            assert_ne!(group, unsafe { libc::getpgrp() });
            return OwnedProcess { pid, group };
        }
        current = parent;
    }
    panic!("PID is not owned by this test process");
}
pub fn wait_gone(process: &OwnedProcess, timeout: Duration) {
    let until = Instant::now() + timeout;
    loop {
        // Signal zero only queries existence. It does not kill or alter any process.
        let pid_gone = unsafe { libc::kill(process.pid, 0) } == -1
            && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH);
        let group_gone = unsafe { libc::kill(-process.group, 0) } == -1
            && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH);
        if pid_gone && group_gone {
            return;
        }
        assert!(
            Instant::now() < until,
            "Product did not reap the owned process/group"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}
pub fn json_lines(path: &Path) -> Vec<serde_json::Value> {
    let text = std::fs::read_to_string(path).unwrap();
    assert!(text.ends_with('\n'));
    text.lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
pub fn ended(session: &mut session::Session) {
    let until = Instant::now() + Duration::from_secs(15);
    loop {
        match session.try_receive_event() {
            Ok(Some(session::Event::Ended { .. })) | Err(session::Error::Ended) => return,
            Ok(_) => {}
            Err(error) => panic!("Event observation failed: {error}"),
        }
        assert!(
            Instant::now() < until,
            "Owned process must end before Session drop"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}
pub fn json(value: &impl serde::Serialize) -> serde_json::Value {
    serde_json::to_value(value).unwrap()
}
pub fn file(outcome: report::provider::Outcome) -> PathBuf {
    match outcome {
        report::provider::Outcome::Created {
            reference: report::provider::Reference::File(r),
        }
        | report::provider::Outcome::Existing {
            reference: report::provider::Reference::File(r),
        } => r.path,
        _ => panic!("Expected local reference"),
    }
}

/// Own test-process environment only. Set before Session/foreign threads; restore after their join.
/// Each integration binary has one test; do not use this while any session/provider thread lives.
pub struct Environment(Vec<(String, Option<std::ffi::OsString>)>);
impl Environment {
    pub fn new(values: impl IntoIterator<Item = (String, String)>) -> Self {
        let mut old = Vec::new();
        for (key, value) in values {
            old.push((key.clone(), std::env::var_os(&key)));
            // SAFETY: callers set this before starting SDK threads in this single-test binary.
            unsafe {
                std::env::set_var(key, value);
            }
        }
        Self(old)
    }
}
impl Drop for Environment {
    fn drop(&mut self) {
        for (key, value) in self.0.drain(..).rev() {
            // SAFETY: scope ordering drops and joins all sessions/providers before this guard.
            unsafe {
                match value {
                    Some(v) => std::env::set_var(key, v),
                    None => std::env::remove_var(key),
                }
            }
        }
    }
}
pub fn gh_environment(path: &Path) -> Vec<(String, String)> {
    let output = std::process::Command::new("python3").current_dir(root())
        .args(["-c", "import json,sys; from tests.providers import gh_fixture; print(json.dumps(gh_fixture(sys.argv[1])))"])
        .arg(path).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let map: std::collections::BTreeMap<String, String> =
        serde_json::from_slice(&output.stdout).unwrap();
    map.into_iter().collect()
}
pub fn git(path: &Path, arguments: &[&str]) -> String {
    assert!(
        path.starts_with(root().join("target/test-runs")),
        "Only owned fixture repositories"
    );
    let hooks = path.join(".acceptance-empty-hooks");
    std::fs::create_dir_all(&hooks).unwrap();
    let output = std::process::Command::new("git")
        .current_dir(path)
        .args([
            "-c",
            "user.name=Woodpecker Tests",
            "-c",
            "user.email=tests@example.invalid",
            "-c",
            "commit.gpgsign=false",
        ])
        .arg("-c")
        .arg(format!("core.hooksPath={}", hooks.display()))
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(arguments)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}
pub fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        assert!(
            !entry.file_type().unwrap().is_symlink(),
            "Fixture must not follow foreign links"
        );
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &to.join(entry.file_name()));
        } else {
            std::fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
        }
    }
}
