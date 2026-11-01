use crate::{command::input::text::Input, session::protocol::Diagnostic};
use bevy::prelude::*;
#[cfg(feature = "ui")]
use bevy::{
    input_focus::InputFocus,
    text::{EditableText, TextEdit, TextReadWriteMode},
    ui::InteractionDisabled,
};

#[cfg(all(test, feature = "ui"))]
#[path = "text/tests/test_target_binding.rs"]
mod test_target_binding;

const MAX_BYTES: usize = 16_384;

#[cfg(feature = "ui")]
fn is_editable(world: &World, entity: Entity) -> bool {
    world.get::<EditableText>(entity).is_some()
        && world
            .get::<TextReadWriteMode>(entity)
            .is_none_or(|mode| *mode == TextReadWriteMode::Editable)
        && world.get::<InteractionDisabled>(entity).is_none()
}

#[derive(Default)]
pub(crate) struct State {
    #[cfg(feature = "ui")]
    pending: Vec<(Entity, String)>,
}

impl State {
    pub(crate) fn input(&mut self, world: &World, input: Input) -> Result<(), Diagnostic> {
        if input.text.len() > MAX_BYTES {
            return Err(Diagnostic::new(
                "text_too_large",
                "text exceeds 16384 UTF-8 bytes",
            ));
        }
        super::primary_window(world).ok_or_else(|| {
            Diagnostic::new(
                "text_window_unavailable",
                "expected exactly one primary window",
            )
        })?;
        #[cfg(feature = "ui")]
        {
            let entity = world
                .get_resource::<InputFocus>()
                .and_then(InputFocus::get)
                .filter(|entity| is_editable(world, *entity))
                .ok_or_else(|| {
                    Diagnostic::new(
                        "text_focus_unavailable",
                        "expected a live focused EditableText entity",
                    )
                })?;
            // Pin the validated target. Another queued input may change focus during
            // the tick; it must not redirect this text into a different field.
            self.pending.push((entity, input.text));
            Ok(())
        }
        #[cfg(not(feature = "ui"))]
        Err(Diagnostic::new(
            "text_focus_unavailable",
            "text input requires woodpecker/ui",
        ))
    }

    pub(crate) fn flush(&mut self, _world: &mut World) {
        #[cfg(feature = "ui")]
        for (entity, text) in self.pending.drain(..) {
            if !is_editable(_world, entity) {
                continue;
            }
            if let Some(mut editable) = _world.get_mut::<EditableText>(entity) {
                // Bevy 0.20's ImeCommit counts selected text against max_characters.
                // Clear composition, then use its selection-aware insertion path.
                // Both edits still run through Bevy's normal filter and edit system.
                editable.queue_edit(TextEdit::ImeSetCompose {
                    value: "".into(),
                    cursor: None,
                });
                editable.queue_edit(TextEdit::Insert(text.into()));
            }
        }
    }
}
