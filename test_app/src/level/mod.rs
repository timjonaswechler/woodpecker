mod environment;
pub(crate) use environment::{hide_planning_markers, spawn_level, update_lighting};
pub(crate) mod actions;
pub(crate) mod definition;
mod import;
pub(crate) mod interactions;
pub mod mission;
pub(crate) mod navigation;
pub mod objects;
pub mod scene;

use crate::{app::settings::Settings, ui::UiState};
use bevy::{light::CascadeShadowConfigBuilder, prelude::*};
use navigation::Navigation;
use scene::Scenery;

pub const TEAL: Color = Color::srgb(0.15, 0.65, 0.59);
pub const GOLD: Color = Color::srgb(1.0, 0.73, 0.25);
#[derive(Resource, Default, PartialEq, Eq, Reflect)]
#[reflect(Resource)]
pub enum LevelLoad {
    #[default]
    Loading,
    Ready,
    Failed,
}
impl LevelLoad {
    pub fn is_ready(&self) -> bool {
        *self == Self::Ready
    }
}
#[derive(Resource, Default)]
pub struct LevelAssets(Vec<UntypedHandle>);
impl LevelAssets {
    pub fn track(&mut self, handle: UntypedHandle) {
        self.0.push(handle);
    }
    pub fn track_all(&mut self, handles: impl IntoIterator<Item = UntypedHandle>) {
        self.0.extend(handles);
    }
    pub fn handles(&self) -> &[UntypedHandle] {
        &self.0
    }
}
pub fn ground(point: Vec2, height: f32) -> Vec3 {
    Vec3::new(point.x, height, point.y)
}
pub fn navigation_world() -> Navigation {
    let mut obstacles: Vec<_> = definition::DATA
        .obstacles()
        .iter()
        .filter(|o| !crate::level::objects::replaces(o.name()))
        .cloned()
        .collect();
    obstacles.extend(crate::level::objects::colliders());
    for (i, center) in crate::level::objects::shooting_range::TARGETS
        .into_iter()
        .enumerate()
    {
        for (part, (size, position)) in crate::level::objects::shooting_range::stand_boxes(center)
            .into_iter()
            .enumerate()
        {
            obstacles.push(
                crate::level::navigation::Obstacle::new(
                    format!("Target support {i}/{part}"),
                    position.to_array(),
                    (size * 0.5).to_array(),
                )
                .expect("valid obstacle"),
            );
        }
    }
    Navigation::new(
        definition::DATA.surfaces().to_vec(),
        obstacles,
        Vec2::new(-39., -27.),
        Vec2::new(39., 27.),
    )
}

pub fn check_assets(
    assets: Res<AssetServer>,
    tracked: Res<LevelAssets>,
    mut load: ResMut<LevelLoad>,
    mut ui: ResMut<UiState>,
) {
    if *load != LevelLoad::Loading {
        return;
    }
    for handle in tracked.handles() {
        if let Some(bevy::asset::LoadState::Failed(error)) = assets.get_load_state(handle.id()) {
            ui.show(format!("Asset konnte nicht geladen werden: {error}"));
            *load = LevelLoad::Failed;
            error!("{}", ui.status());
            return;
        }
    }
    if !tracked.handles().is_empty()
        && tracked
            .0
            .iter()
            .all(|h| assets.is_loaded_with_dependencies(h.id()))
    {
        *load = LevelLoad::Ready;
        ui.show(
            "Erkunde die acht Räume. Drei Energiezellen starten das Signal. E bedient Stationen."
                .into(),
        );
    }
}

pub(crate) fn install(app: &mut App) {
    use crate::app::schedule::Phase;
    use objects::{pickups, shooting_range};
    use pickups::coins;
    app.init_resource::<LevelLoad>()
        .init_resource::<LevelAssets>()
        .init_resource::<mission::Mission>()
        .init_resource::<mission::inventory::CoinRun>()
        .init_resource::<shooting_range::ShootingRange>()
        .insert_resource(navigation_world())
        .add_systems(
            Startup,
            (
                spawn_level,
                objects::spawn_objects,
                pickups::spawn,
                coins::spawn_coins,
                shooting_range::spawn_range,
            ),
        )
        .add_observer(hide_planning_markers)
        .add_systems(Update, check_assets.in_set(Phase::Input))
        .add_systems(
            Update,
            (objects::update_collision, shooting_range::tick_feedback).in_set(Phase::PrepareWorld),
        )
        .add_systems(
            Update,
            actions::navigate
                .in_set(Phase::Commands)
                .after(crate::app::flow::handle_actions)
                .after(crate::player::actions::handle_actions),
        )
        .add_systems(
            Update,
            actions::handle_actions
                .in_set(Phase::WorldActions)
                .after(crate::input::devices::click_to_move),
        )
        .add_systems(
            Update,
            shooting_range::resolve_shot.in_set(Phase::ResolveHits),
        )
        .add_systems(
            Update,
            (coins::tick_run, mission::update_progress)
                .chain()
                .in_set(Phase::Progress),
        )
        .add_systems(
            Update,
            (
                pickups::update,
                objects::update_rack_model,
                objects::update_visuals,
                coins::update_coins,
                update_lighting,
                shooting_range::update_visuals,
            )
                .in_set(Phase::Present),
        );
}
