//! Application-level transitions; gameplay changes stay with their owners.
use crate::{
    input::{
        PointerCapture,
        intents::{Action, Actions},
    },
    level::{
        mission::{Mission, inventory::CoinRun},
        objects::shooting_range::ShootingRange,
    },
    player::{Player, camera::ViewState},
    ui::UiState,
};
use bevy::{input_focus::InputFocus, prelude::*};

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct GameFlow {
    menu: bool,
}
impl GameFlow {
    pub fn menu(&self) -> bool {
        self.menu
    }
    pub fn open_menu(&mut self) {
        self.menu = true;
    }
    pub fn resume(&mut self) {
        self.menu = false;
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Restart coordinates the independently owned gameplay resources."
)]
pub fn handle_actions(
    mut actions: ResMut<Actions>,
    mut flow: ResMut<GameFlow>,
    mut capture: ResMut<PointerCapture>,
    mut player: ResMut<Player>,
    mut view: ResMut<ViewState>,
    mut coins: ResMut<CoinRun>,
    mut mission: ResMut<Mission>,
    mut range: ResMut<ShootingRange>,
    mut blaster: ResMut<crate::player::blaster::Blaster>,
    mut ui: ResMut<UiState>,
    mut focus: ResMut<InputFocus>,
    mut exit: MessageWriter<AppExit>,
) {
    for action in actions.take(|a| {
        matches!(
            a,
            Action::Settings | Action::Resume | Action::Restart | Action::Quit
        )
    }) {
        focus.clear();
        match action {
            Action::Settings => {
                flow.open_menu();
                capture.set(false);
                player.clear_route();
            }
            Action::Resume => {
                flow.resume();
                capture.set(view.first_person());
            }
            Action::Restart => {
                player.reset();
                view.reset();
                coins.reset();
                mission.reset();
                range.reset();
                blaster.reset();
                flow.resume();
                capture.set(false);
                ui.show("Neue Runde. Erkunde die Räume und finde die drei Energiezellen.".into());
            }
            Action::Quit => {
                exit.write(AppExit::Success);
            }
            _ => unreachable!(),
        }
    }
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<GameFlow>().add_systems(
        Update,
        handle_actions.in_set(super::schedule::Phase::Commands),
    );
}
