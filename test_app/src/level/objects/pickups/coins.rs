use crate::level::mission::inventory::CoinRun;
use crate::{level::definition::DATA, player::Player, player::camera::ViewState, ui::UiState};
use bevy::prelude::*;

#[derive(Component)]
pub struct Coin(pub crate::level::mission::CoinId);

pub fn spawn_coins(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut tracked: ResMut<crate::level::LevelAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut scene = crate::level::scene::Scenery {
        commands: &mut commands,
        assets: &assets,
        tracked: &mut tracked,
        meshes: &mut meshes,
        materials: &mut materials,
        parent: None,
    };
    for (i, pos) in DATA.coins().into_iter().enumerate() {
        let coin = scene.model(
            "meshes/collectibles/Coin_A.gltf".into(),
            Transform::from_translation(Vec3::from(pos) + Vec3::Y).with_scale(Vec3::splat(0.65)),
        );
        scene
            .commands
            .entity(coin)
            .insert(Coin(crate::level::mission::CoinId::ALL[i]));
    }
}

pub fn tick_run(
    player: Res<Player>,
    view: Res<ViewState>,
    capture: Res<crate::input::PointerCapture>,
    mut run: ResMut<CoinRun>,
    mut ui: ResMut<UiState>,
    gate: crate::input::policy::InputGate,
) {
    if !gate.allows(
        crate::input::policy::GameplayAction::Collect,
        &view,
        &capture,
    ) {
        return;
    }
    for (index, coin) in DATA.coins().into_iter().map(Vec3::from).enumerate() {
        if player.position().distance(coin.xz()) < 0.85
            && (player.height() - coin.y).abs() < 0.4
            && run.collect(crate::level::mission::CoinId::ALL[index])
        {
            ui.show("Münze eingesammelt.".into());
        }
    }
}

pub fn update_coins(
    time: Res<Time>,
    run: Res<CoinRun>,
    mut coins: Query<(&Coin, &mut Transform, &mut Visibility)>,
) {
    for (coin, mut transform, mut visibility) in &mut coins {
        *visibility = if run.was_collected(coin.0) {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        transform.rotation =
            Quat::from_rotation_y(time.elapsed_secs() * 1.4 + coin.0.index() as f32);
        transform.translation.y = DATA.coins()[coin.0.index()][1]
            + 1.0
            + (time.elapsed_secs() * 2.0 + coin.0.index() as f32).sin() * 0.14;
    }
}
