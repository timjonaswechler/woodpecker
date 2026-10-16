//! Native play follows window focus; controlled sessions own their input independently.
use crate::{
    commands::{Action, Actions},
    player::camera::ViewState,
    ui::UiState,
};
use bevy::{
    camera::NormalizedRenderTarget,
    ecs::system::SystemParam,
    picking::pointer::{PointerId, PointerLocation},
};
use bevy::{input_focus::InputFocus, prelude::*, text::EditableText, window::PrimaryWindow};

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Native,
    Controlled(PointerId),
}

impl InputMode {
    pub fn accepts_input(self, window: &Window) -> bool {
        matches!(self, Self::Controlled(_)) || window.focused
    }
}

/// Reads the active device without changing the native window cursor.
#[derive(SystemParam)]
pub struct Cursor<'w, 's> {
    mode: Res<'w, InputMode>,
    pointers: Query<'w, 's, (&'static PointerId, &'static PointerLocation)>,
}

impl Cursor<'_, '_> {
    pub fn position(&self, window_entity: Entity, window: &Window) -> Option<Vec2> {
        match *self.mode {
            InputMode::Native => window.cursor_position(),
            InputMode::Controlled(id) => self.pointers.iter().find_map(|(pointer, location)| {
                if *pointer != id {
                    return None;
                }
                let location = location.location.as_ref()?;
                let position = location.position;
                (matches!(location.target, NormalizedRenderTarget::Window(target)
                    if target.entity() == window_entity)
                    && position.is_finite()
                    && position.cmpge(Vec2::ZERO).all()
                    && position
                        .cmplt(Vec2::new(window.width(), window.height()))
                        .all())
                .then_some(position)
            }),
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Shortcuts read the input mode, focus and gameplay state."
)]
pub fn keyboard_shortcuts(
    mode: Res<InputMode>,
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut focus: ResMut<InputFocus>,
    inputs: Query<&EditableText>,
    mut view: ResMut<ViewState>,
    ui: Res<UiState>,
    mut actions: ResMut<Actions>,
) {
    if !mode.accepts_input(&window) {
        view.captured = false;
        return;
    }
    if let Some(input) = focus.get().and_then(|entity| inputs.get(entity).ok()) {
        view.captured = false;
        if keys.just_pressed(KeyCode::Escape)
            || (keys.just_pressed(KeyCode::Enter) && !input.allow_newlines)
        {
            focus.clear();
        }
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        focus.clear();
        if ui.menu {
            actions.0.push(Action::Resume);
        } else if view.captured {
            view.captured = false;
        } else {
            actions.0.push(Action::Settings);
        }
        return;
    }
    if !ui.menu && keys.just_pressed(KeyCode::KeyV) {
        actions.0.push(Action::View);
    }
    if !ui.menu && keys.just_pressed(KeyCode::KeyM) {
        actions.0.push(Action::Settings);
    }
    if !ui.menu && keys.just_pressed(KeyCode::KeyG) {
        actions.0.push(Action::Map);
    }
    if !ui.menu && keys.just_pressed(KeyCode::KeyR) {
        actions.0.push(Action::Restart);
    }
    if !ui.menu && keys.just_pressed(KeyCode::Backspace) {
        actions.0.push(Action::Checkpoint);
    }
    if !ui.menu && keys.just_pressed(KeyCode::KeyE) {
        actions.0.push(Action::Interact);
    }
}

#[cfg(test)]
mod tests;
