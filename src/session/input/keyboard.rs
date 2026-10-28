use crate::{command::input::keyboard::Key, session::protocol::Diagnostic};
use bevy::{
    ecs::schedule::ScheduleCleanupPolicy,
    input::{
        ButtonState, InputSystems,
        keyboard::{Key as LogicalKey, KeyboardInput, keyboard_input_system},
    },
    prelude::*,
    window::WindowEvent,
};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct State {
    pressed: HashSet<KeyCode>,
    pending: Vec<KeyboardInput>,
}

impl State {
    pub(crate) fn key(&mut self, world: &World, key: &Key, down: bool) -> Result<(), Diagnostic> {
        let (key_code, logical_key) = key
            .resolve()
            .ok_or_else(|| Diagnostic::new("invalid_key", key.as_str()))?;
        let window = super::primary_window(world)
            .map(|(entity, _)| entity)
            .ok_or_else(|| {
                Diagnostic::new(
                    "keyboard_window_unavailable",
                    "expected exactly one primary window",
                )
            })?;
        if self.pressed.contains(&key_code) == down {
            return Err(Diagnostic::new(
                if down {
                    "key_already_pressed"
                } else {
                    "key_not_pressed"
                },
                key.as_str(),
            ));
        }
        if down {
            self.pressed.insert(key_code);
        } else {
            self.pressed.remove(&key_code);
        }
        self.pending.push(KeyboardInput {
            key_code,
            logical_key,
            window,
            state: if down {
                ButtonState::Pressed
            } else {
                ButtonState::Released
            },
            repeat: false,
            text: None,
        });
        Ok(())
    }

    pub(crate) fn flush(&mut self, world: &mut World) {
        for event in self.pending.drain(..) {
            world.write_message(event.clone());
            world.write_message(WindowEvent::KeyboardInput(event));
        }
    }
}

pub(super) fn install(app: &mut App) {
    app.world_mut()
        .schedule_scope(PreUpdate, |world, schedule| {
            schedule
                .remove_systems_in_set(
                    keyboard_input_system,
                    world,
                    ScheduleCleanupPolicy::RemoveSystemsOnly,
                )
                .expect("replace native keyboard state updater");
        });
    app.add_systems(PreUpdate, update.in_set(InputSystems));
}

fn update(
    mut events: MessageReader<KeyboardInput>,
    mut physical: ResMut<ButtonInput<KeyCode>>,
    mut logical: ResMut<ButtonInput<LogicalKey>>,
    mut held: Local<HashMap<KeyCode, LogicalKey>>,
) {
    physical.bypass_change_detection().clear();
    logical.bypass_change_detection().clear();
    for event in events.read() {
        match event.state {
            ButtonState::Pressed => {
                physical.press(event.key_code);
                logical.press(event.logical_key.clone());
                held.insert(event.key_code, event.logical_key.clone());
            }
            ButtonState::Released => {
                physical.release(event.key_code);
                held.remove(&event.key_code);
                // Both Shift keys, for example, map to logical Shift. Releasing one
                // physical key must not release a logical key still held by the other.
                if !held.values().any(|key| *key == event.logical_key) {
                    logical.release(event.logical_key.clone());
                }
            }
        }
    }
}
