use bevy::{
    camera::NormalizedRenderTarget,
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
}

impl Input {
    pub(super) fn pointer(pointer: PointerInput) -> Self {
        Self {
            pointer,
            relative: None,
        }
    }

    pub(super) fn relative(pointer: PointerInput, delta: Vec2) -> Self {
        Self {
            pointer,
            relative: Some(delta),
        }
    }

    pub(super) fn emit(self, world: &mut World) {
        // Gate has removed native input before this tick. The regular Bevy input
        // systems now accumulate these complete deltas, regardless of OS focus.
        if let Some(delta) = self.relative {
            world.write_message(MouseMotion { delta });
        }
        let NormalizedRenderTarget::Window(window) = self.pointer.location.target else {
            unreachable!("virtual pointer is window-backed")
        };
        match self.pointer.action {
            PointerAction::Press(button) | PointerAction::Release(button) => {
                world.write_message(MouseButtonInput {
                    window: window.entity(),
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
                    window: window.entity(),
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
