//! The file representation is converted before Settings can be used.
use super::Settings;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(default)]
pub(super) struct RawSettings {
    sensitivity: f32,
    speed: f32,
    fov: f32,
    show_path: bool,
    shadows: bool,
    invert_y: bool,
    player_name: String,
    notes: String,
}

impl Default for RawSettings {
    fn default() -> Self {
        Self {
            sensitivity: 1.0,
            speed: 4.5,
            fov: 75.0,
            show_path: true,
            shadows: true,
            invert_y: false,
            player_name: "Spieler".into(),
            notes: String::new(),
        }
    }
}

pub(super) enum Control {
    Sensitivity,
    Speed,
    Fov,
}
impl Control {
    pub fn normalize(self, value: f32) -> Result<f32, String> {
        let (name, min, max) = match self {
            Self::Sensitivity => ("sensitivity", 0.2, 3.0),
            Self::Speed => ("speed", 2.0, 8.0),
            Self::Fov => ("fov", 55.0, 105.0),
        };
        if !value.is_finite() {
            return Err(format!("{name} must be finite"));
        }
        Ok(value.clamp(min, max))
    }
}

pub(super) fn player_name(value: &str) -> String {
    let cleaned: String = value.chars().filter(|c| !c.is_control()).take(24).collect();
    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        "Spieler".into()
    } else {
        cleaned.into()
    }
}

impl TryFrom<RawSettings> for Settings {
    type Error = String;
    fn try_from(raw: RawSettings) -> Result<Self, Self::Error> {
        Ok(Self {
            sensitivity: Control::Sensitivity.normalize(raw.sensitivity)?,
            speed: Control::Speed.normalize(raw.speed)?,
            fov: Control::Fov.normalize(raw.fov)?,
            show_path: raw.show_path,
            shadows: raw.shadows,
            invert_y: raw.invert_y,
            player_name: player_name(&raw.player_name),
            notes: raw.notes,
            dirty: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn import_and_edits_share_normalization_and_preserve_notes() {
        let mut settings: Settings = serde_json::from_str(
            r#"{"player_name":"  Jäger\n  ","speed":999,"notes":"\n世界🙂\n"}"#,
        )
        .unwrap();
        assert_eq!(settings.player_name(), "Jäger");
        assert_eq!(settings.speed(), 8.0);
        settings.apply(super::super::Edit::Speed(-100.0)).unwrap();
        assert_eq!(settings.speed(), 2.0);
        assert!(settings.apply(super::super::Edit::Speed(f32::NAN)).is_err());
        assert_eq!(settings.speed(), 2.0);
        settings.set_name("\n  ");
        settings.apply(super::super::Edit::Defaults).unwrap();
        assert_eq!(settings.player_name(), "Spieler");
        assert_eq!(settings.notes(), "\n世界🙂\n");
        let round_trip: Settings =
            serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
        assert_eq!(round_trip.notes(), settings.notes());
    }
    #[test]
    fn old_files_keep_missing_field_defaults() {
        let settings: Settings =
            serde_json::from_str(r#"{"player_name":"Jäger","speed":6.0}"#).unwrap();
        assert_eq!(settings.speed(), 6.0);
        assert!(settings.notes().is_empty());
    }
}
