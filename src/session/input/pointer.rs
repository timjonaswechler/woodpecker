mod cursor;
mod motion;

use crate::session::protocol::Diagnostic;
use bevy::{
    input::{mouse::MouseScrollUnit, touch::TouchPhase},
    picking::pointer::{PointerAction, PointerButton, PointerId, PointerInput},
    prelude::*,
};
use std::collections::HashSet;

/// Identifies Woodpecker's virtual pointer within each Bevy World.
/// Read its `PointerLocation` after picking input processing to obtain the
/// position applied by the latest simulation tick. Queued input is not exposed.
pub const ID: PointerId = PointerId::Custom(bevy::asset::uuid::Uuid::from_u128(
    0x776f6f64_7065_636b_6572_000000000001,
));

/// Accepted state is private: neither input messages nor Window/ButtonInput change before a tick.
#[derive(Default)]
pub(crate) struct State {
    cursor: cursor::State,
    pressed: HashSet<MouseButton>,
    pending: Vec<motion::Input>,
}

impl State {
    pub(crate) fn move_to(&mut self, world: &World, position: [f32; 2]) -> Result<(), Diagnostic> {
        let position = finite(position)?;
        let event = self.cursor.move_to(world, position)?;
        self.pending.push(motion::Input::pointer(event));
        Ok(())
    }

    pub(crate) fn move_by(&mut self, world: &World, delta: [f32; 2]) -> Result<(), Diagnostic> {
        let delta = finite(delta)?;
        let event = self.cursor.move_by(world, delta)?;
        self.pending.push(motion::Input::relative(event, delta));
        Ok(())
    }

    pub(crate) fn button(
        &mut self,
        world: &World,
        token: &str,
        down: bool,
    ) -> Result<(), Diagnostic> {
        let (button, picking_button) = match token {
            "left" => (MouseButton::Left, PointerButton::Primary),
            "right" => (MouseButton::Right, PointerButton::Secondary),
            "middle" => (MouseButton::Middle, PointerButton::Middle),
            _ => return Err(Diagnostic::new("invalid_pointer_button", token)),
        };
        let (window, position) = self.cursor.location(world)?;
        if self.pressed.contains(&button) == down {
            return Err(Diagnostic::new(
                if down {
                    "pointer_button_already_pressed"
                } else {
                    "pointer_button_not_pressed"
                },
                token,
            ));
        }
        self.pending.push(motion::Input::pointer(PointerInput::new(
            ID,
            cursor::location(window, position),
            if down {
                PointerAction::Press(picking_button)
            } else {
                PointerAction::Release(picking_button)
            },
        )));
        if down {
            self.pressed.insert(button);
        } else {
            self.pressed.remove(&button);
        }
        Ok(())
    }

    pub(crate) fn scroll(&mut self, world: &World, delta: [f32; 2]) -> Result<(), Diagnostic> {
        let delta = finite(delta)?;
        cursor::window(world)?;
        let (window, position) = self.cursor.location(world)?;
        self.pending.push(motion::Input::pointer(PointerInput::new(
            ID,
            cursor::location(window, position),
            PointerAction::Scroll {
                unit: MouseScrollUnit::Line,
                x: delta.x,
                y: delta.y,
                phase: TouchPhase::Moved,
            },
        )));
        Ok(())
    }

    pub(crate) fn flush(&mut self, world: &mut World) {
        for input in self.pending.drain(..) {
            input.emit(world);
        }
    }
}

fn finite(pair: [f32; 2]) -> Result<Vec2, Diagnostic> {
    let value = Vec2::from_array(pair);
    if value.is_finite() {
        Ok(value)
    } else {
        Err(Diagnostic::new(
            "invalid_arguments",
            "coordinates and deltas must be finite",
        ))
    }
}

#[cfg(test)]
mod tests;
