//! Owned artifacts/worker cleanup and pure bounded observations. No starts, sends or ticks.
#![allow(dead_code)] // Helpers are selected by feature-specific owner tests.
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
pub(crate) struct WorkerGuard {
    pub cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub handle: Option<std::thread::JoinHandle<()>>,
}
impl Drop for WorkerGuard {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        if let Some(worker) = self.handle.take() {
            let _ = worker.join();
        }
    }
}
pub(crate) fn artifacts(label: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/test-runs")
        .join(format!(
            "internal-{label}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
    std::fs::create_dir_all(&path).unwrap();
    println!("Artefakte: {}", path.display());
    path
}
pub(crate) fn config(path: &std::path::Path) -> crate::session::Config {
    #[derive(serde::Deserialize)]
    struct File {
        launch: crate::session::launch::Config,
        tick: crate::command::tick::Config,
        startup: crate::session::startup::Config,
        report: crate::report::Config,
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = std::process::Command::new("python3")
        .args([
            "-c",
            "import json,sys,tomllib; print(json.dumps(tomllib.load(open(sys.argv[1],'rb'))))",
        ])
        .arg(root.join("tests/app.toml"))
        .output()
        .unwrap();
    assert!(output.status.success());
    let data: File = serde_json::from_slice(&output.stdout).unwrap();
    crate::session::Config {
        launch: data.launch,
        tick: data.tick,
        startup: data.startup,
        report: data.report,
        artifact_dir: path.into(),
    }
}
pub(crate) fn context(config: &crate::session::Config) -> crate::report::Context {
    crate::report::Context {
        application: crate::report::Application {
            package: "owned-external-wire".into(),
            version: "0.0.0".into(),
            target: config.launch.target.clone(),
            features: vec![],
            arguments: vec![],
            source: None,
        },
        woodpecker_version: env!("CARGO_PKG_VERSION").into(),
        protocol_version: 3,
        capabilities: crate::session::Capabilities::default(),
        tick: config.tick.clone(),
        platform: crate::report::Platform {
            os: std::env::consts::OS.into(),
            arch: std::env::consts::ARCH.into(),
        },
        toolchain: crate::report::Toolchain::default(),
        commands: vec![],
    }
}
pub(crate) fn receive<T: serde::de::DeserializeOwned>(
    session: &mut crate::session::Session,
    pending: &mut crate::session::Pending<T>,
) -> Result<T, crate::session::Error> {
    let end = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(value) = session.try_receive(pending)? {
            return Ok(value);
        }
        assert!(Instant::now() < end, "Pending deadline");
        std::thread::sleep(Duration::from_millis(2));
    }
}
pub(crate) fn json_lines(path: impl AsRef<std::path::Path>) -> Vec<serde_json::Value> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|v| serde_json::from_str(v).unwrap())
        .collect()
}
pub(crate) fn wait(mut observed: impl FnMut() -> bool) {
    let end = Instant::now() + Duration::from_secs(10);
    while !observed() {
        assert!(
            Instant::now() < end,
            "Observation deadline; no action retried"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}
