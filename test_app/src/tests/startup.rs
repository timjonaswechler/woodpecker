use super::*;
#[test]
fn level_player_coins_and_camera_spawn_through_their_own_modules() {
    use crate::{level::objects::pickups::coins, player, player::camera};
    use bevy::world_serialization::WorldAsset;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<WorldAsset>()
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .init_resource::<level::LevelAssets>()
        .init_resource::<crate::app::settings::Settings>();
    let world = app.world_mut();

    world.run_system_cached(level::spawn_level).unwrap();
    assert_eq!(world.query::<&WorldAssetRoot>().iter(world).count(), 1);
    assert_eq!(world.query::<&player::PlayerModel>().iter(world).count(), 0);
    assert_eq!(world.query::<&coins::Coin>().iter(world).count(), 0);
    assert_eq!(world.query::<&camera::GameCamera>().iter(world).count(), 0);

    world.run_system_cached(player::spawn_player).unwrap();
    let transform = world
        .query_filtered::<&Transform, With<player::PlayerModel>>()
        .single(world)
        .unwrap();
    assert_eq!(transform.translation, map::START);
    assert_eq!(transform.scale, Vec3::splat(player::MODEL_SCALE));

    world.run_system_cached(coins::spawn_coins).unwrap();
    let mut pickups: Vec<_> = world
        .query::<(&coins::Coin, &Transform)>()
        .iter(world)
        .map(|(coin, transform)| (coin.0.index(), transform.translation))
        .collect();
    pickups.sort_by_key(|(index, _)| *index);
    assert_eq!(pickups.len(), DATA.coins().len());
    for (index, position) in pickups {
        assert_eq!(position, Vec3::from(DATA.coins()[index]) + Vec3::Y);
    }
    assert_eq!(world.resource::<level::LevelAssets>().handles().len(), 26);

    world.run_system_cached(camera::spawn_camera).unwrap();
    let transform = world
        .query_filtered::<&Transform, With<camera::GameCamera>>()
        .single(world)
        .unwrap();
    assert_eq!(transform.translation, map::START + Vec3::new(10., 32., 25.));
}

#[test]
fn reflected_resource_paths_match_the_session_contract() {
    use bevy::reflect::TypePath;

    assert_eq!(
        [
            level::LevelLoad::type_path(),
            Player::type_path(),
            crate::player::camera::ViewState::type_path(),
            Mission::type_path(),
            crate::level::objects::shooting_range::ShootingRange::type_path(),
            crate::level::mission::inventory::CoinRun::type_path(),
            crate::ui::UiState::type_path(),
            crate::app::settings::Settings::type_path(),
            crate::input::PointerCapture::type_path(),
            crate::app::flow::GameFlow::type_path(),
        ],
        [
            "test_app::level::LevelLoad",
            "test_app::player::Player",
            "test_app::player::camera::ViewState",
            "test_app::level::mission::Mission",
            "test_app::level::objects::shooting_range::ShootingRange",
            "test_app::level::mission::inventory::CoinRun",
            "test_app::ui::UiState",
            "test_app::app::settings::Settings",
            "test_app::input::PointerCapture",
            "test_app::app::flow::GameFlow",
        ]
    );
}
