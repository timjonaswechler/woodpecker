//! Navigation commands run before movement; interactions use the current eye pose.
use super::{
    definition as map,
    mission::{Interaction, Mission},
    navigation::Navigation,
};
use crate::{
    input::{
        PointerCapture,
        intents::{Action, Actions},
        policy::{GameplayAction, InputGate},
    },
    player::{
        Player, RouteOutcome,
        camera::{ViewMode, ViewState},
    },
    ui::UiState,
};
use bevy::prelude::*;

#[expect(
    clippy::too_many_arguments,
    reason = "Navigation commands coordinate player, view and mission destination."
)]
pub(crate) fn navigate(
    mut actions: ResMut<Actions>,
    mut player: ResMut<Player>,
    mut view: ResMut<ViewState>,
    mut capture: ResMut<PointerCapture>,
    nav: Res<Navigation>,
    mission: Res<Mission>,
    gate: InputGate,
    mut ui: ResMut<UiState>,
) {
    for action in
        actions.take(|a| matches!(a, Action::Checkpoint | Action::Objective | Action::Range))
    {
        if !gate.allows(GameplayAction::Navigate, &view, &capture) {
            continue;
        }
        if matches!(action, Action::Checkpoint) {
            let p = if mission.checkpoint() {
                map::CHECKPOINT
            } else {
                map::START
            };
            player.respawn(p.xz(), p.y);
            ui.show("Zurück am sicheren Punkt. Dein Fortschritt bleibt erhalten.".into());
        } else {
            let goal = if matches!(action, Action::Range) {
                map::RANGE_ENTRY
            } else {
                mission.destination()
            };
            let outcome = player.move_to(&nav, goal);
            if outcome == RouteOutcome::Started {
                view.set_mode(ViewMode::Overview);
                capture.set(false);
            }
            ui.show(crate::ui::messages::route(outcome));
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Interactions resolve current visibility, mission state and collision together."
)]
pub(crate) fn handle_actions(
    mut actions: ResMut<Actions>,
    mut player: ResMut<Player>,
    view: Res<ViewState>,
    capture: Res<PointerCapture>,
    mut nav: ResMut<Navigation>,
    mut mission: ResMut<Mission>,
    gate: InputGate,
    mut ui: ResMut<UiState>,
) {
    for _ in actions.take(|a| matches!(a, Action::Interact)) {
        if !gate.allows(GameplayAction::Interact, &view, &capture) {
            continue;
        }
        let action = mission.interaction(&player, &nav, &view);
        if action != Interaction::None {
            ui.show(crate::ui::messages::mission(mission.try_apply(action)));
            super::objects::sync_collision(&mission, &mut nav, &mut player);
        }
    }
}
