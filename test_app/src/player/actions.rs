use super::{
    Player,
    camera::{ViewMode, ViewState},
};
use crate::input::{
    PointerCapture,
    intents::{Action, Actions},
};
use bevy::prelude::*;
pub(crate) fn handle_actions(
    mut actions: ResMut<Actions>,
    mut player: ResMut<Player>,
    mut view: ResMut<ViewState>,
    mut capture: ResMut<PointerCapture>,
) {
    for action in actions.take(|a| matches!(a, Action::View | Action::Map)) {
        match action {
            Action::View => {
                view.toggle();
                capture.set(view.first_person());
            }
            Action::Map => {
                let mode = if view.map() {
                    ViewMode::Overview
                } else {
                    ViewMode::Map
                };
                view.set_mode(mode);
                capture.set(false);
            }
            _ => unreachable!(),
        }
        player.route.clear();
    }
}
