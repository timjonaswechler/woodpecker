use super::{Diagnostic, ID};
use bevy::{
    camera::NormalizedRenderTarget,
    picking::pointer::{Location, PointerAction, PointerInput},
    prelude::*,
    window::WindowRef,
};

/// Accepted virtual geometry, independent of device motion and the OS cursor.
#[derive(Default)]
pub(super) struct State {
    location: Option<(Entity, Vec2)>,
}

impl State {
    pub(super) fn move_to(
        &mut self,
        world: &World,
        position: Vec2,
    ) -> Result<PointerInput, Diagnostic> {
        let (entity, window) = window(world)?;
        if !in_bounds(window, position) {
            return Err(Diagnostic::new(
                "pointer_position_out_of_bounds",
                "position must lie inside the primary window",
            ));
        }
        let previous = self
            .location
            .filter(|(id, _)| *id == entity)
            .map_or(position, |(_, position)| position);
        Ok(self.moved(entity, previous, position))
    }

    pub(super) fn move_by(
        &mut self,
        world: &World,
        delta: Vec2,
    ) -> Result<PointerInput, Diagnostic> {
        let (entity, window) = window(world)?;
        let size = Vec2::new(window.width(), window.height());
        if !size.is_finite() || size.min_element() <= 0.0 {
            return Err(Diagnostic::new(
                "pointer_window_unavailable",
                "the primary window must have finite positive dimensions",
            ));
        }
        // Window coordinates exclude the upper bound, including at HiDPI scales.
        let maximum = Vec2::new(size.x.next_down(), size.y.next_down());
        let previous = self
            .location
            .filter(|(id, _)| *id == entity)
            .map_or(size * 0.5, |(_, position)| position)
            .clamp(Vec2::ZERO, maximum);
        // Add in f64 so even the largest finite f32 deltas cannot overflow.
        let position = (previous.as_dvec2() + delta.as_dvec2())
            .clamp(bevy::math::DVec2::ZERO, maximum.as_dvec2())
            .as_vec2();
        Ok(self.moved(entity, previous, position))
    }

    fn moved(&mut self, entity: Entity, previous: Vec2, position: Vec2) -> PointerInput {
        self.location = Some((entity, position));
        PointerInput::new(
            ID,
            location(entity, position),
            PointerAction::Move {
                delta: position - previous,
            },
        )
    }

    pub(super) fn location(&self, world: &World) -> Result<(Entity, Vec2), Diagnostic> {
        self.location
            .filter(|(entity, position)| {
                window(world)
                    .is_ok_and(|(id, window)| id == *entity && in_bounds(window, *position))
            })
            .ok_or_else(|| {
                Diagnostic::new(
                    "pointer_location_unavailable",
                    "move the pointer into the primary window first",
                )
            })
    }
}

pub(super) fn location(window: Entity, position: Vec2) -> Location {
    Location {
        target: NormalizedRenderTarget::Window(WindowRef::Entity(window).normalize(None).unwrap()),
        position,
    }
}

fn in_bounds(window: &Window, position: Vec2) -> bool {
    position.is_finite()
        && position.x >= 0.0
        && position.y >= 0.0
        && position.x < window.width()
        && position.y < window.height()
}

pub(super) fn window(world: &World) -> Result<(Entity, &Window), Diagnostic> {
    super::super::primary_window(world).ok_or_else(|| {
        Diagnostic::new(
            "pointer_window_unavailable",
            "expected exactly one primary window",
        )
    })
}
