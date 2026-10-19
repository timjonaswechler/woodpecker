use crate::{
    level::LevelLoad, level::layout::DATA, player::Player, player::camera::ViewState, ui::UiState,
};
use bevy::{input_focus::InputFocus, prelude::*, text::EditableText, window::PrimaryWindow};

#[derive(Component)]
pub struct Coin(pub usize);

pub fn spawn_coins(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut tracked: ResMut<crate::level::LevelAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut scene = crate::level::scenery::Scenery {
        commands: &mut commands,
        assets: &assets,
        tracked: &mut tracked,
        meshes: &mut meshes,
        materials: &mut materials,
        parent: None,
    };
    for (i, pos) in DATA.coins.into_iter().enumerate() {
        let coin = scene.model(
            "meshes/collectibles/Coin_A.gltf".into(),
            Transform::from_translation(Vec3::from(pos) + Vec3::Y).with_scale(Vec3::splat(0.65)),
        );
        scene.commands.entity(coin).insert(Coin(i));
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct CoinRun {
    pub collected: [bool; 24],
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
    reason = "Coin collection observes the same input gates as movement."
)]
pub fn tick_run(
    mode: Res<crate::input::InputMode>,
    window: Single<&Window, With<PrimaryWindow>>,
    focus: Res<InputFocus>,
    inputs: Query<(), With<EditableText>>,
    load: Res<LevelLoad>,
    player: Res<Player>,
    view: Res<ViewState>,
    mut run: ResMut<CoinRun>,
    mut ui: ResMut<UiState>,
) {
    if !load.is_ready()
        || ui.menu
        || !mode.accepts_input(&window)
        || focus.get().is_some_and(|e| inputs.contains(e))
        || (view.first_person && !view.captured)
    {
        return;
    }
    for (index, coin) in DATA.coins.into_iter().map(Vec3::from).enumerate() {
        if !run.collected[index]
            && player.position.distance(coin.xz()) < 0.85
            && (player.height - coin.y).abs() < 0.4
        {
            run.collected[index] = true;
            ui.status = "Münze eingesammelt.".into();
        }
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
        transform.translation.y =
            DATA.coins[coin.0][1] + 1.0 + (time.elapsed_secs() * 2.0 + coin.0 as f32).sin() * 0.14;
    }
}
