//! Coordinates user commands that span the demo's modules.

pub mod items;
pub mod level;

use crate::{
    camera::ViewState,
    game::items::blueprint,
    game::items::coins::CoinRun,
    game::level::LevelLoad,
    input::InputMode,
    player::Player,
    player::actions::expedition::Expedition,
    player::actions::navigation::Navigation,
    player::actions::shooting::Practice,
    settings::Settings,
    ui::{Action, Actions, UiState},
};
use bevy::{input_focus::InputFocus, prelude::*, text::EditableText, window::PrimaryWindow};

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
                if action != crate::player::actions::expedition::Interaction::None {
                    ui.status = expedition.apply(action);
                }
            }
            Action::Checkpoint => {
                let p = if expedition.checkpoint {
                    blueprint::CHECKPOINT
                } else {
                    blueprint::START
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
                    blueprint::PRACTICE
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

#[expect(
    clippy::too_many_arguments,
    reason = "Shortcuts read the input mode, focus and gameplay state."
)]
pub fn keyboard_shortcuts(
    mode: Res<InputMode>,
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut focus: ResMut<InputFocus>,
    inputs: Query<&EditableText>,
    mut view: ResMut<ViewState>,
    ui: Res<UiState>,
    mut actions: ResMut<Actions>,
) {
    if !mode.accepts_input(&window) {
        view.captured = false;
        return;
    }
    if let Some(input) = focus.get().and_then(|entity| inputs.get(entity).ok()) {
        view.captured = false;
        if keys.just_pressed(KeyCode::Escape)
            || (keys.just_pressed(KeyCode::Enter) && !input.allow_newlines)
        {
            focus.clear();
        }
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        focus.clear();
        if ui.menu {
            actions.0.push(Action::Resume);
        } else if view.captured {
            view.captured = false;
        } else {
            actions.0.push(Action::Settings);
        }
        return;
    }
    if !ui.menu && keys.just_pressed(KeyCode::KeyV) {
        actions.0.push(Action::View);
    }
    if !ui.menu && keys.just_pressed(KeyCode::KeyM) {
        actions.0.push(Action::Settings);
    }
    if !ui.menu && keys.just_pressed(KeyCode::KeyG) {
        actions.0.push(Action::Map);
    }
    if !ui.menu && keys.just_pressed(KeyCode::KeyR) {
        actions.0.push(Action::Restart);
    }
    if !ui.menu && keys.just_pressed(KeyCode::Backspace) {
        actions.0.push(Action::Checkpoint);
    }
    if !ui.menu && keys.just_pressed(KeyCode::KeyE) {
        actions.0.push(Action::Interact);
    }
}
