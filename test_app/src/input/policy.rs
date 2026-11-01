//! Common input conditions, evaluated against the current application state.
use super::{InputMode, PointerCapture};
use crate::{app::flow::GameFlow, level::LevelLoad, player::camera::ViewState};
use bevy::{
    ecs::system::SystemParam, input_focus::InputFocus, prelude::*, text::EditableText,
    window::PrimaryWindow,
};

pub enum GameplayAction {
    Move,
    Look,
    Interact,
    Fire,
    Collect,
    Navigate,
}

#[derive(SystemParam)]
pub struct InputGate<'w, 's> {
    mode: Res<'w, InputMode>,
    flow: Res<'w, GameFlow>,
    load: Res<'w, LevelLoad>,
    focus: Res<'w, InputFocus>,
    inputs: Query<'w, 's, (), With<EditableText>>,
    window: Single<'w, 's, &'static Window, With<PrimaryWindow>>,
}

impl InputGate<'_, '_> {
    pub fn pointer(&self) -> bool {
        pointer_allowed(*self.mode, &self.flow, &self.load, &self.window)
    }

    pub fn allows(
        &self,
        action: GameplayAction,
        view: &ViewState,
        capture: &PointerCapture,
    ) -> bool {
        if !self.pointer() || self.focus.get().is_some_and(|e| self.inputs.contains(e)) {
            return false;
        }
        match action {
            GameplayAction::Navigate => true,
            GameplayAction::Look | GameplayAction::Fire => view.first_person() && capture.active(),
            GameplayAction::Move | GameplayAction::Collect | GameplayAction::Interact => {
                !view.first_person() || capture.active()
            }
        }
    }
}

pub(super) fn pointer_allowed(
    mode: InputMode,
    flow: &GameFlow,
    load: &LevelLoad,
    window: &Window,
) -> bool {
    load.is_ready() && !flow.menu() && mode.accepts_input(window)
}
