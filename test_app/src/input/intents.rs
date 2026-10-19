//! Commands produced by UI and input; their owners perform the changes.
use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub enum Action {
    View,
    Map,
    Settings,
    Resume,
    Restart,
    Range,
    Interact,
    Objective,
    Checkpoint,
    Sensitivity(f32),
    Speed(f32),
    Fov(f32),
    Path,
    Shadows,
    Invert,
    Defaults,
    Quit,
}
#[derive(Resource, Default)]
pub struct Actions(pub Vec<Action>);

impl Actions {
    pub fn take(&mut self, matches: impl Fn(Action) -> bool) -> Vec<Action> {
        let mut selected = Vec::new();
        self.0.retain(|action| {
            if matches(*action) {
                selected.push(*action);
                false
            } else {
                true
            }
        });
        selected
    }
}

#[derive(Clone, Copy)]
pub struct Movement {
    pub axis: Vec2,
    pub sprint: bool,
    pub jump: bool,
}
#[derive(Resource, Default)]
pub struct FrameInput {
    pub(super) movement: Option<Movement>,
    pub(super) look: Vec2,
    pub(super) fire: bool,
}
impl FrameInput {
    pub fn movement(&self) -> Option<Movement> {
        self.movement
    }
    pub fn look(&self) -> Vec2 {
        self.look
    }
    pub fn fire(&self) -> bool {
        self.fire
    }
}
