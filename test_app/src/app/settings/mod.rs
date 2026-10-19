//! Valid preferences and personal text; all edits and persistence enter here.
mod storage;
mod values;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Serialize, Deserialize, Clone, Reflect)]
#[reflect(Resource)]
#[serde(try_from = "values::RawSettings")]
pub struct Settings {
    sensitivity: f32,
    speed: f32,
    fov: f32,
    show_path: bool,
    shadows: bool,
    invert_y: bool,
    player_name: String,
    notes: String,
    #[serde(skip)]
    #[reflect(ignore)]
    dirty: bool,
}

pub enum Edit {
    Sensitivity(f32),
    Speed(f32),
    Fov(f32),
    Path,
    Shadows,
    Invert,
    Defaults,
}

impl Default for Settings {
    fn default() -> Self {
        values::RawSettings::default()
            .try_into()
            .expect("valid defaults")
    }
}

impl Settings {
    pub fn sensitivity(&self) -> f32 {
        self.sensitivity
    }
    pub fn speed(&self) -> f32 {
        self.speed
    }
    pub fn fov(&self) -> f32 {
        self.fov
    }
    pub fn show_path(&self) -> bool {
        self.show_path
    }
    pub fn shadows(&self) -> bool {
        self.shadows
    }
    pub fn invert_y(&self) -> bool {
        self.invert_y
    }
    pub fn player_name(&self) -> &str {
        &self.player_name
    }
    pub fn notes(&self) -> &str {
        &self.notes
    }

    /// Also used by development checks to read the persisted snapshot.
    pub fn load() -> Self {
        storage::Storage::current()
            .and_then(|s| s.load())
            .expect("settings initialization failed")
    }

    pub fn apply(&mut self, edit: Edit) -> Result<(), String> {
        match edit {
            Edit::Sensitivity(delta) => {
                self.sensitivity =
                    values::Control::Sensitivity.normalize(self.sensitivity + delta)?
            }
            Edit::Speed(delta) => {
                self.speed = values::Control::Speed.normalize(self.speed + delta)?
            }
            Edit::Fov(delta) => self.fov = values::Control::Fov.normalize(self.fov + delta)?,
            Edit::Path => self.show_path = !self.show_path,
            Edit::Shadows => self.shadows = !self.shadows,
            Edit::Invert => self.invert_y = !self.invert_y,
            Edit::Defaults => {
                let defaults = Self::default();
                self.sensitivity = defaults.sensitivity;
                self.speed = defaults.speed;
                self.fov = defaults.fov;
                self.show_path = defaults.show_path;
                self.shadows = defaults.shadows;
                self.invert_y = defaults.invert_y;
            }
        }
        self.dirty = true;
        Ok(())
    }

    pub fn set_name(&mut self, name: &str) {
        let name = values::player_name(name);
        if self.player_name != name {
            self.player_name = name;
            self.dirty = true;
        }
    }

    pub fn set_notes(&mut self, notes: &str) {
        if self.notes != notes {
            self.notes = notes.to_owned();
            self.dirty = true;
        }
    }
}

#[derive(Resource)]
pub(crate) struct Persistence(storage::Storage);

pub(crate) fn install(app: &mut App, smoke: bool) {
    let storage = storage::Storage::current().expect("settings storage initialization failed");
    let settings = if smoke {
        Settings::default()
    } else {
        storage.load().expect("settings initialization failed")
    };
    app.insert_resource(settings)
        .insert_resource(Persistence(storage))
        .add_systems(
            Update,
            handle_actions.in_set(crate::app::schedule::Phase::Commands),
        )
        .add_systems(
            Update,
            persist
                .in_set(crate::app::schedule::Phase::Present)
                .before(crate::ui::update_ui),
        );
}

pub(crate) fn persist(
    mut settings: ResMut<Settings>,
    storage: Option<Res<Persistence>>,
    mut ui: ResMut<crate::ui::UiState>,
) {
    if !settings.dirty {
        return;
    }
    let Some(storage) = storage else {
        return;
    };
    match storage.0.save(&settings) {
        Ok(()) => settings.dirty = false,
        Err(error) => ui.show(format!(
            "Einstellungen konnten nicht gespeichert werden: {error}"
        )),
    }
}

pub(crate) fn handle_actions(
    mut actions: ResMut<crate::input::intents::Actions>,
    mut settings: ResMut<Settings>,
    mut ui: ResMut<crate::ui::UiState>,
) {
    use crate::input::intents::Action;
    for action in actions.take(|a| {
        matches!(
            a,
            Action::Sensitivity(_)
                | Action::Speed(_)
                | Action::Fov(_)
                | Action::Path
                | Action::Shadows
                | Action::Invert
                | Action::Defaults
        )
    }) {
        let edit = match action {
            Action::Sensitivity(v) => Edit::Sensitivity(v),
            Action::Speed(v) => Edit::Speed(v),
            Action::Fov(v) => Edit::Fov(v),
            Action::Path => Edit::Path,
            Action::Shadows => Edit::Shadows,
            Action::Invert => Edit::Invert,
            Action::Defaults => Edit::Defaults,
            _ => unreachable!(),
        };
        if let Err(error) = settings.apply(edit) {
            ui.show(error);
        }
    }
}
