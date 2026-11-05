use bevy::{
    input::{
        ButtonState,
        mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    },
    picking::pointer::{PointerAction, PointerButton, PointerInput},
    prelude::*,
};

/// Keeps relative device motion paired with its bounded picking movement in the
/// same command queue as buttons and scrolling. Absolute placement has no motion.
pub(super) struct Input {
    pub(super) pointer: PointerInput,
    relative: Option<Vec2>,
    window: Entity,
}

impl Input {
    pub(super) fn pointer(window: Entity, pointer: PointerInput) -> Self {
        Self {
            pointer,
            window,
            relative: None,
        }
    }

    pub(super) fn relative(window: Entity, pointer: PointerInput, delta: Vec2) -> Self {
        Self {
            pointer,
            window,
            relative: Some(delta),
        }
    }

    pub(super) fn emit(self, world: &mut World) {
        // Gate has removed native input before this tick. The regular Bevy input
        // systems now accumulate these complete deltas, regardless of OS focus.
        if let Some(delta) = self.relative {
            world.write_message(MouseMotion { delta });
        }
        // Keyboard/mouse messages retain their admitted logical identity even
        // when picking targets an Image rather than a native window.
        let window = self.window;
        match self.pointer.action {
            PointerAction::Press(button) | PointerAction::Release(button) => {
                world.write_message(MouseButtonInput {
                    window,
                    button: match button {
                        PointerButton::Primary => MouseButton::Left,
                        PointerButton::Secondary => MouseButton::Right,
                        PointerButton::Middle => MouseButton::Middle,
                    },
                    state: if matches!(self.pointer.action, PointerAction::Press(_)) {
                        ButtonState::Pressed
                    } else {
                        ButtonState::Released
                    },
                });
            }
            PointerAction::Scroll { unit, x, y, phase } => {
                world.write_message(MouseWheel {
                    window,
                    unit,
                    x,
                    y,
                    phase,
                });
            }
            _ => {}
        }
        // Picking uses our custom pointer. Writing Window's cursor position would
        // let Winit forward the change to the operating system.
        world.write_message(self.pointer);
    }
}
