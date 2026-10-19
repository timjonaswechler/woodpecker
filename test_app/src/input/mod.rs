//! Native play follows window focus; controlled sessions own their input independently.
pub(crate) mod devices;
pub(crate) mod intents;
pub use devices::{Cursor, InputMode};
pub(crate) mod policy;
use crate::{
    input::intents::{Action, Actions},
    player::camera::ViewState,
};
use bevy::{
    camera::NormalizedRenderTarget,
    ecs::system::SystemParam,
    picking::pointer::{PointerId, PointerLocation},
};
use bevy::{input_focus::InputFocus, prelude::*, text::EditableText, window::PrimaryWindow};

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct PointerCapture {
    active: bool,
    #[reflect(ignore)]
    previous: bool,
}
impl PointerCapture {
    pub fn active(&self) -> bool {
        self.active
    }
    pub fn set(&mut self, active: bool) {
        self.active = active;
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
    mut capture: ResMut<PointerCapture>,
    flow: Res<crate::app::flow::GameFlow>,
    mut actions: ResMut<Actions>,
) {
    if !mode.accepts_input(&window) {
        capture.set(false);
        return;
    }
    if let Some(input) = focus.get().and_then(|entity| inputs.get(entity).ok()) {
        capture.set(false);
        if keys.just_pressed(KeyCode::Escape)
            || (keys.just_pressed(KeyCode::Enter) && !input.allow_newlines)
        {
            focus.clear();
        }
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        focus.clear();
        if flow.menu() {
            actions.0.push(Action::Resume);
        } else if capture.active() {
            capture.set(false);
        } else {
            actions.0.push(Action::Settings);
        }
        return;
    }
    if !flow.menu() && keys.just_pressed(KeyCode::KeyV) {
        actions.0.push(Action::View);
    }
    if !flow.menu() && keys.just_pressed(KeyCode::KeyM) {
        actions.0.push(Action::Settings);
    }
    if !flow.menu() && keys.just_pressed(KeyCode::KeyG) {
        actions.0.push(Action::Map);
    }
    if !flow.menu() && keys.just_pressed(KeyCode::KeyR) {
        actions.0.push(Action::Restart);
    }
    if !flow.menu() && keys.just_pressed(KeyCode::Backspace) {
        actions.0.push(Action::Checkpoint);
    }
    if !flow.menu() && keys.just_pressed(KeyCode::KeyE) {
        actions.0.push(Action::Interact);
    }
}

#[cfg(test)]
mod tests;

#[expect(
    clippy::too_many_arguments,
    reason = "Samples keyboard, mouse and action permissions into one frame."
)]
pub fn sample_gameplay(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mouse: Res<bevy::input::mouse::AccumulatedMouseMotion>,
    focus: Res<InputFocus>,
    view: Res<ViewState>,
    mut capture: ResMut<PointerCapture>,
    gate: policy::InputGate,
    mut frame: ResMut<intents::FrameInput>,
) {
    use policy::GameplayAction;
    *frame = intents::FrameInput::default();
    if gate.allows(GameplayAction::Move, &view, &capture) {
        let mut axis = Vec2::ZERO;
        for (arrow, fpv, direction) in [
            (KeyCode::ArrowLeft, KeyCode::KeyA, Vec2::NEG_X),
            (KeyCode::ArrowRight, KeyCode::KeyD, Vec2::X),
            (KeyCode::ArrowUp, KeyCode::KeyW, Vec2::NEG_Y),
            (KeyCode::ArrowDown, KeyCode::KeyS, Vec2::Y),
        ] {
            if keys.pressed(arrow) || (view.first_person() && keys.pressed(fpv)) {
                axis += direction;
            }
        }
        frame.movement = Some(intents::Movement {
            axis,
            sprint: keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
            jump: keys.just_pressed(KeyCode::Space) && focus.get().is_none(),
        });
    }
    if gate.allows(GameplayAction::Look, &view, &capture) && mouse.delta.is_finite() {
        frame.look = mouse.delta;
    }
    let can_fire = gate.allows(GameplayAction::Fire, &view, &capture);
    frame.fire = can_fire && capture.previous && buttons.just_pressed(MouseButton::Left);
    capture.previous = can_fire;
}

pub(crate) fn install(app: &mut App) {
    use crate::app::schedule::Phase;
    app.init_resource::<InputMode>()
        .init_resource::<PointerCapture>()
        .init_resource::<Actions>()
        .init_resource::<intents::FrameInput>()
        .add_systems(
            Update,
            keyboard_shortcuts
                .in_set(Phase::Input)
                .after(crate::ui::buttons)
                .after(crate::ui::sync_text_fields),
        )
        .add_systems(Update, sample_gameplay.in_set(Phase::SampleInput))
        .add_systems(Update, devices::click_to_move.in_set(Phase::WorldActions));
}
