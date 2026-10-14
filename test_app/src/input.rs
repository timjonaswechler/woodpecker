//! Native play follows window focus; controlled sessions own their input independently.
use bevy::prelude::*;
use bevy::{
    camera::NormalizedRenderTarget,
    ecs::system::SystemParam,
    picking::pointer::{PointerId, PointerLocation},
};

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Native,
    #[cfg_attr(
        not(any(feature = "automation", test)),
        expect(
            dead_code,
            reason = "Only the optional automation integration selects a virtual device."
        )
    )]
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

#[cfg(test)]
mod tests;
