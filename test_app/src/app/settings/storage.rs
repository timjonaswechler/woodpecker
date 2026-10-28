//! Selected once at startup; controlled sessions never fall back to shared writes.
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
        )?;
        if let Some(seed) = crate::app::launch::value::<PathBuf>("--settings-seed") {
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
    ) -> Result<Self, String> {
        if let Some(session) = session {
            if !session.is_absolute() || !session.join("config").is_dir() {
                return Err("WOODPECKER_SESSION_DIR must name an existing absolute session workspace with a config directory".into());
            }
            return Ok(Self {
                path: session.join("config").join("settings.json"),
                seed: Some(project.join("settings.json")),
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
            path: project.join("settings.json"),
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
                let settings = serde_json::from_slice::<Settings>(&bytes)
                    .map_err(|e| format!("invalid starting settings: {e}"))?;
                std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&self.path)
                    .and_then(|mut file| file.write_all(&bytes))
                    .map_err(|e| e.to_string())?;
                return Ok(settings);
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
