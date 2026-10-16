//! Select persistence once per operation; controlled sessions never fall back to shared writes.
use super::Settings;
use std::{
    io::{ErrorKind, Write},
    path::PathBuf,
};

pub(super) struct Storage {
    path: PathBuf,
    seed: Option<PathBuf>,
    isolated: bool,
    required_seed: bool,
}

impl Storage {
    pub(super) fn current() -> Result<Self, String> {
        let mut storage = Self::select(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            std::env::var_os("WOODPECKER_SESSION_DIR").map(PathBuf::from),
            std::env::var_os("WOODPECKER_ARTIFACT_DIR").is_some(),
            std::env::args().any(|arg| arg == "--smoke-test"),
        )?;
        if let Some(seed) = crate::launch::value::<PathBuf>("--settings-seed") {
            if !storage.isolated {
                return Err("--settings-seed requires an isolated session workspace".into());
            }
            storage.seed = Some(seed);
            storage.required_seed = true;
        }
        Ok(storage)
    }

    fn select(
        project: PathBuf,
        session: Option<PathBuf>,
        controlled: bool,
        smoke: bool,
    ) -> Result<Self, String> {
        if let Some(session) = session {
            if !session.is_absolute() || !session.join("config").is_dir() {
                return Err("WOODPECKER_SESSION_DIR must name an existing absolute session workspace with a config directory".into());
            }
            return Ok(Self {
                path: session.join("config").join(if smoke {
                    "smoke-settings.json"
                } else {
                    "settings.json"
                }),
                seed: (!smoke).then(|| project.join("settings.json")),
                isolated: true,
                required_seed: false,
            });
        }
        if controlled {
            return Err(
                "controlled sessions require WOODPECKER_SESSION_DIR for isolated settings".into(),
            );
        }
        Ok(Self {
            path: project.join(if smoke {
                "artifacts/smoke-settings.json"
            } else {
                "settings.json"
            }),
            seed: None,
            isolated: false,
            required_seed: false,
        })
    }

    pub(super) fn load(&self) -> Result<Settings, String> {
        let result = self.read();
        if self.isolated {
            result
        } else {
            Ok(result.unwrap_or_default())
        }
    }

    fn read(&self) -> Result<Settings, String> {
        let bytes = match std::fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound && self.isolated => {
                let seed = self.seed.as_ref().map(std::fs::read).transpose();
                let bytes = match seed {
                    Ok(Some(bytes)) => bytes,
                    Ok(None) => serde_json::to_vec_pretty(&Settings::default())
                        .map_err(|e| e.to_string())?,
                    Err(error) if error.kind() == ErrorKind::NotFound && !self.required_seed => {
                        serde_json::to_vec_pretty(&Settings::default())
                            .map_err(|e| e.to_string())?
                    }
                    Err(error) => return Err(format!("cannot read starting settings: {error}")),
                };
                // Validate before publishing the seed; an invalid baseline must not
                // silently start a controlled run with different settings.
                serde_json::from_slice::<Settings>(&bytes)
                    .map_err(|e| format!("invalid starting settings: {e}"))?;
                std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&self.path)
                    .and_then(|mut file| file.write_all(&bytes))
                    .map_err(|e| e.to_string())?;
                bytes
            }
            Err(error) => return Err(error.to_string()),
        };
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())
    }

    pub(super) fn save(&self, settings: &Settings) -> Result<(), String> {
        let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
        let temporary = self.path.with_extension("json.tmp");
        std::fs::write(&temporary, json).map_err(|e| e.to_string())?;
        std::fs::rename(temporary, &self.path).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "signalstation-settings-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path.canonicalize().unwrap())
        }
        fn session(&self, id: &str) -> Storage {
            let path = self.0.join(id).join("workspace");
            std::fs::create_dir_all(path.join("config")).unwrap();
            Storage::select(self.0.clone(), Some(path), true, false).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn parallel_sessions_copy_the_seed_once_and_save_independently() {
        let fixture = Fixture::new();
        let seed = br#"{"player_name":"Baseline","speed":6.0,"notes":"initial"}"#;
        std::fs::write(fixture.0.join("settings.json"), seed).unwrap();
        let a = fixture.session("session-a");
        let b = fixture.session("session-b");
        assert_eq!(a.load().unwrap().player_name, "Baseline");
        assert_eq!(b.load().unwrap().speed, 6.0);
        assert_eq!(std::fs::read(&a.path).unwrap(), seed);
        assert_eq!(std::fs::read(&b.path).unwrap(), seed);
        std::thread::scope(|scope| {
            for (store, name) in [(&a, "Session A"), (&b, "Session B")] {
                scope.spawn(move || {
                    let mut settings = store.load().unwrap();
                    settings.player_name = name.into();
                    settings.notes = format!("{name}: Grüße 🙂");
                    for _ in 0..20 {
                        store.save(&settings).unwrap();
                    }
                });
            }
        });
        assert_eq!(a.load().unwrap().player_name, "Session A");
        assert_eq!(b.load().unwrap().player_name, "Session B");
        assert_eq!(a.load().unwrap().notes, "Session A: Grüße 🙂");
        assert_eq!(
            std::fs::read(fixture.0.join("settings.json")).unwrap(),
            seed
        );
        assert_eq!(
            fixture.session("session-c").load().unwrap().player_name,
            "Baseline"
        );
        std::fs::write(fixture.0.join("settings.json"), b"broken source").unwrap();
        assert_eq!(
            a.load().unwrap().player_name,
            "Session A",
            "the copied seed must not be reread"
        );
        assert!(!a.path.with_extension("json.tmp").exists());
        assert!(!b.path.with_extension("json.tmp").exists());
    }

    #[test]
    fn absent_seed_uses_defaults_but_invalid_seed_or_workspace_fails_closed() {
        let fixture = Fixture::new();
        assert_eq!(
            fixture.session("defaults").load().unwrap().speed,
            Settings::default().speed
        );
        assert!(Storage::select(fixture.0.clone(), None, true, false).is_err());
        assert!(Storage::select(fixture.0.clone(), Some("relative".into()), true, false).is_err());
        assert!(
            Storage::select(
                fixture.0.clone(),
                Some(fixture.0.join("missing")),
                true,
                false
            )
            .is_err()
        );
        std::fs::write(fixture.0.join("settings.json"), "invalid").unwrap();
        let invalid = fixture.session("invalid");
        assert!(invalid.load().is_err());
        assert!(!invalid.path.exists());
        let normal = Storage::select(fixture.0.clone(), None, false, false).unwrap();
        assert_eq!(normal.load().unwrap().speed, Settings::default().speed);
    }

    #[test]
    fn smoke_settings_also_stay_in_the_session() {
        let fixture = Fixture::new();
        let normal = fixture.session("smoke");
        let smoke = Storage::select(
            fixture.0.clone(),
            Some(normal.path.parent().unwrap().parent().unwrap().into()),
            true,
            true,
        )
        .unwrap();
        smoke.load().unwrap();
        assert_eq!(smoke.path.file_name().unwrap(), "smoke-settings.json");
        assert!(!fixture.0.join("artifacts/smoke-settings.json").exists());
    }
}
