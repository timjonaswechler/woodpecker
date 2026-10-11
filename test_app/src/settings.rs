use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Resource, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Settings {
    pub sensitivity: f32,
    pub speed: f32,
    pub fov: f32,
    pub show_path: bool,
    pub shadows: bool,
    pub invert_y: bool,
    pub player_name: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sensitivity: 1.0,
            speed: 4.5,
            fov: 75.0,
            show_path: true,
            shadows: true,
            invert_y: false,
            player_name: "Spieler".into(),
        }
    }
}

impl Settings {
    pub fn path() -> PathBuf {
        if std::env::args().any(|arg| arg == "--smoke-test") {
            return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("artifacts/smoke-settings.json");
        }
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("settings.json")
    }

    pub fn load() -> Self {
        let mut value: Self = std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        value.sensitivity = value.sensitivity.clamp(0.2, 3.0);
        value.speed = value.speed.clamp(2.0, 8.0);
        value.fov = value.fov.clamp(55.0, 105.0);
        value.player_name = value
            .player_name
            .chars()
            .filter(|c| !c.is_control())
            .take(24)
            .collect();
        value
    }

    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        let temporary = Self::path().with_extension("json.tmp");
        std::fs::write(&temporary, json).map_err(|e| e.to_string())?;
        std::fs::rename(temporary, Self::path()).map_err(|e| e.to_string())
    }
}
