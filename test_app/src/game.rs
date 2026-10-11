//! Coordinates user commands that span the demo's modules.
use crate::{
    camera::ViewState,
    coins::CoinRun,
    level::LevelLoad,
    navigation::Navigation,
    player::Player,
    settings::Settings,
    shooting::{self, Practice},
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
) {
    for action in actions.0.drain(..) {
        focus.clear();
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
                *ui = UiState {
                    status: "Neue Runde. Sammle alle sechs Münzen.".into(),
                    ..default()
                };
                practice.reset();
            }
            Action::Range => {
                if let Some(route) = nav.path(player.position, shooting::ENTRY) {
                    view.first_person = false;
                    view.captured = false;
                    player.route = route;
                    ui.status = "Unterwegs zum Schießstand. Dort startet E die Zielübung.".into();
                }
            }
            Action::Practice => {
                if !load.is_ready() {
                    continue;
                }
                if shooting::in_range(player.position) {
                    practice.start();
                    view.first_person = true;
                    view.captured = true;
                    player.yaw = 0.0;
                    view.pitch = 0.0;
                    player.route.clear();
                    ui.status =
                        "30 Sekunden! Linksklick schießt. Die Mitte zählt 50 Punkte.".into();
                } else {
                    ui.status =
                        "Betritt zuerst den Schießstand im Norden. Der Button bringt dich hin."
                            .into();
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
                *settings = Settings {
                    player_name: name,
                    ..default()
                };
            }
            Action::Quit => {
                exit.write(AppExit::Success);
            }
        }
        if let Err(error) = settings.save() {
            ui.status = format!("Einstellungen konnten nicht gespeichert werden: {error}");
        }
    }
}

pub fn keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut focus: ResMut<InputFocus>,
    inputs: Query<(), With<EditableText>>,
    mut view: ResMut<ViewState>,
    ui: Res<UiState>,
    mut actions: ResMut<Actions>,
) {
    if !window.focused {
        view.captured = false;
        return;
    }
    let editing = focus.get().is_some_and(|e| inputs.contains(e));
    if editing {
        view.captured = false;
        if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::Enter) {
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
    if !ui.menu && keys.just_pressed(KeyCode::KeyR) {
        actions.0.push(Action::Restart);
    }
    if !ui.menu && keys.just_pressed(KeyCode::KeyE) {
        actions.0.push(Action::Practice);
    }
}
