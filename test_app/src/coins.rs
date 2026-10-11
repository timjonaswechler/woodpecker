use crate::{
    camera::ViewState,
    level::{COINS, LevelLoad, START},
    player::Player,
    settings::Settings,
    shooting,
    ui::UiState,
};
use bevy::{input_focus::InputFocus, prelude::*, text::EditableText, window::PrimaryWindow};

#[derive(Component)]
pub struct Coin(pub usize);

#[derive(Resource, Default)]
pub struct CoinRun {
    pub collected: [bool; COINS.len()],
    pub elapsed: f32,
    pub finished: bool,
}

impl CoinRun {
    pub fn count(&self) -> usize {
        self.collected
            .iter()
            .filter(|&&collected| collected)
            .count()
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Coin rules observe player input gates and end the round at the start platform."
)]
pub fn tick_run(
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
    focus: Res<InputFocus>,
    inputs: Query<(), With<EditableText>>,
    load: Res<LevelLoad>,
    settings: Res<Settings>,
    mut player: ResMut<Player>,
    mut view: ResMut<ViewState>,
    mut run: ResMut<CoinRun>,
    mut ui: ResMut<UiState>,
) {
    if run.finished
        || !load.is_ready()
        || ui.menu
        || !window.focused
        || focus.get().is_some_and(|e| inputs.contains(e))
        || (view.first_person && !view.captured)
    {
        return;
    }
    if !shooting::in_range(player.position) {
        run.elapsed += time.delta_secs().min(0.05);
    }
    for (index, coin) in COINS.into_iter().enumerate() {
        if !run.collected[index] && player.position.distance(coin) < 0.85 {
            run.collected[index] = true;
            ui.status = if run.count() == COINS.len() {
                "Alle Münzen gefunden! Kehre zur grünen Startplattform zurück.".into()
            } else {
                format!("Münze gefunden. Noch {} übrig.", COINS.len() - run.count())
            };
        }
    }
    if !run.finished && run.count() == COINS.len() && player.position.distance(START) < 1.35 {
        run.finished = true;
        view.captured = false;
        player.route.clear();
        ui.status = format!(
            "Geschafft, {}! Alle Münzen in {:.1} Sekunden. R startet eine neue Runde.",
            settings.player_name, run.elapsed
        );
    }
}

pub fn update_coins(
    time: Res<Time>,
    run: Res<CoinRun>,
    mut coins: Query<(&Coin, &mut Transform, &mut Visibility)>,
) {
    for (coin, mut transform, mut visibility) in &mut coins {
        *visibility = if run.collected[coin.0] {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        transform.rotation = Quat::from_rotation_y(time.elapsed_secs() * 1.4 + coin.0 as f32);
        transform.translation.y = 1.0 + (time.elapsed_secs() * 2.0 + coin.0 as f32).sin() * 0.14;
    }
}
