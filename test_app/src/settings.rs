use bevy::prelude::*;
use serde::{Deserialize, Serialize};
mod storage;

#[derive(Resource, Serialize, Deserialize, Clone, Reflect)]
#[reflect(Resource)]
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
    pub fn load() -> Self {
        let mut value = storage::Storage::current()
            .and_then(|storage| storage.load())
            .expect("settings initialization failed");
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
        storage::Storage::current()?.save(self)
    }
}
