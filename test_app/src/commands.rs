//! Coordinates user commands that span the demo's modules.

use crate::{
    expedition::Expedition, items::coins::CoinRun, level::LevelLoad, level::layout,
    level::navigation::Navigation, player::Player, player::camera::ViewState, practice::Practice,
    settings::Settings, ui::UiState,
};
use bevy::{input_focus::InputFocus, prelude::*};

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

#[expect(
    clippy::too_many_arguments,
    reason = "User commands coordinate the independent demo resources."
)]
pub fn handle_actions(
    mut actions: ResMut<Actions>,
    mut player: ResMut<Player>,
    mut view: ResMut<ViewState>,
    mut run: ResMut<CoinRun>,
    mut ui: ResMut<UiState>,
    load: Res<LevelLoad>,
    mut settings: ResMut<Settings>,
    mut focus: ResMut<InputFocus>,
    mut exit: MessageWriter<AppExit>,
    nav: Res<Navigation>,
    mut practice: ResMut<Practice>,
    mut expedition: ResMut<Expedition>,
) {
    for action in actions.0.drain(..) {
        focus.clear();
        let save_settings = matches!(
            action,
            Action::Sensitivity(_)
                | Action::Speed(_)
                | Action::Fov(_)
                | Action::Path
                | Action::Shadows
                | Action::Invert
                | Action::Defaults
        );
        match action {
            Action::View => {
                view.toggle();
                player.route.clear();
            }
            Action::Settings => {
                ui.menu = true;
                view.captured = false;
                player.route.clear();
            }
            Action::Resume => {
                ui.menu = false;
                view.captured = view.first_person;
            }
            Action::Restart => {
                *player = Player::default();
                *view = ViewState::default();
                *run = CoinRun::default();
                *expedition = Expedition::default();
                *ui = UiState {
                    status: "Neue Runde. Erkunde die Räume und finde die drei Energiezellen."
                        .into(),
                    ..default()
                };
                practice.reset();
            }
            Action::Map => {
                view.first_person = false;
                view.captured = false;
                view.map = !view.map;
            }
            Action::Interact => {
                if !load.is_ready() || ui.menu {
                    continue;
                }
                let action = expedition.interaction(&player, &nav, &view);
                if action != crate::expedition::Interaction::None {
                    ui.status = expedition.apply(action);
                }
            }
            Action::Checkpoint => {
                let p = if expedition.checkpoint {
                    layout::CHECKPOINT
                } else {
                    layout::START
                };
                player.respawn(p.xz(), p.y);
                ui.status = "Zurück am sicheren Punkt. Dein Fortschritt bleibt erhalten.".into();
            }
            Action::Objective | Action::Range => {
                if (player.height - nav.support(player.position, player.height + 0.02)).abs() > 0.15
                {
                    ui.status = "Lande zuerst, bevor du einen Laufweg auswählst.".into();
                    continue;
                }
                let goal = if matches!(action, Action::Range) {
                    layout::PRACTICE
                } else {
                    expedition.destination()
                };
                if let Some(route) = nav.path(player.feet(), goal) {
                    view.first_person = false;
                    view.captured = false;
                    player.route = route;
                    ui.status = "Unterwegs. Pfeiltasten übernehmen jederzeit die Steuerung.".into();
                } else {
                    ui.status =
                        "Kein Laufweg gefunden. Wähle einen freien Boden oder eine Treppe.".into();
                }
            }
            Action::Sensitivity(delta) => {
                settings.sensitivity = (settings.sensitivity + delta).clamp(0.2, 3.0)
            }
            Action::Speed(delta) => settings.speed = (settings.speed + delta).clamp(2.0, 8.0),
            Action::Fov(delta) => settings.fov = (settings.fov + delta).clamp(55.0, 105.0),
            Action::Path => settings.show_path = !settings.show_path,
            Action::Shadows => settings.shadows = !settings.shadows,
            Action::Invert => settings.invert_y = !settings.invert_y,
            Action::Defaults => {
                let name = settings.player_name.clone();
                let notes = settings.notes.clone();
                *settings = Settings {
                    player_name: name,
                    notes,
                    ..default()
                };
            }
            Action::Quit => {
                exit.write(AppExit::Success);
            }
        }
        if save_settings && let Err(error) = settings.save() {
            ui.status = format!("Einstellungen konnten nicht gespeichert werden: {error}");
        }
    }
}
