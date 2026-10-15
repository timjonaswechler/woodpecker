mod models;
use crate::{
    course::{CHEST_CENTER, CHEST_SIZE},
    game::level::LevelAssets,
    player::actions::expedition::{Expedition, Interaction},
    player::actions::interactions::{STATIONS, TABLES},
    player::actions::navigation::Obstacle,
    scenery::Scenery,
};
use bevy::prelude::*;
pub use models::spawn_model;

const FRAME: Color = Color::srgb(0.12, 0.20, 0.22);
const SURFACE: Color = Color::srgb(0.80, 0.83, 0.78);
const BLUE: Color = Color::srgb(0.20, 0.57, 0.85);
const READY: Color = Color::srgb(1., 0.64, 0.12);
const DONE: Color = Color::srgb(0.18, 0.85, 0.43);

pub fn replaces(name: &str) -> bool {
    [
        "Mission board",
        "Workbench controls",
        "Cell assembly workbench",
        "Hall map",
        "Storage rack lever",
        "Lever handle",
        "Valve pipe",
        "Valve handwheel",
        "Archive scan tablet",
        "Archive reward terminal",
        "Greenhouse reward stand",
        "Generator controls",
        "Hall checkpoint",
        "Practice start",
    ]
    .iter()
    .any(|prefix| name.starts_with(prefix))
}

pub fn colliders() -> impl Iterator<Item = Obstacle> {
    STATIONS.into_iter().chain(TABLES).map(|target| {
        let (center, half) = if target.action == Interaction::Chest {
            (CHEST_CENTER, CHEST_SIZE * 0.5)
        } else if matches!(target.action, Interaction::Part(_) | Interaction::Cell(_)) {
            let height = if matches!(target.action, Interaction::Part(_)) {
                0.84
            } else {
                1.
            };
            (
                Vec3::new(target.center.x, target.floor + height / 2., target.center.z),
                Vec3::new(target.half.x, height / 2., target.half.z),
            )
        } else {
            (target.center, target.half)
        };
        Obstacle {
            name: target.name.into(),
            center: center.to_array(),
            half: half.to_array(),
        }
    })
}

#[derive(Component)]
pub struct StationPlaque {
    action: Interaction,
    textures: [Handle<Image>; 3],
}
#[derive(Component)]
pub struct StationLamp(pub Interaction);
#[derive(Component)]
pub struct CompletionMark(pub Interaction);
#[derive(Component)]
pub struct UnreadPage(pub usize);
#[derive(Component)]
pub struct Mechanism {
    action: Interaction,
    rest: Transform,
    used: Transform,
}

pub fn spawn_stations(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut tracked: ResMut<LevelAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut s = Scenery {
        commands: &mut commands,
        assets: &assets,
        tracked: &mut tracked,
        meshes: &mut meshes,
        materials: &mut materials,
        parent: None,
    };
    for t in STATIONS {
        spawn_model(&mut s, t);
    }
    for t in TABLES {
        spawn_model(&mut s, t);
    }
}

/// Model-review override. Normal gameplay always derives visuals from Expedition.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreviewState {
    Locked,
    #[default]
    Ready,
    Used,
}

impl PreviewState {
    pub fn index(self) -> usize {
        match self {
            Self::Locked => 0,
            Self::Ready => 1,
            Self::Used => 2,
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Visual queries share mission or zoo state."
)]
pub fn update_stations(
    mission: Res<Expedition>,
    preview: Option<Res<PreviewState>>,
    lamps: Query<(&StationLamp, &MeshMaterial3d<StandardMaterial>)>,
    plaques: Query<(&StationPlaque, &MeshMaterial3d<StandardMaterial>)>,
    mut marks: Query<(&CompletionMark, &mut Visibility), Without<UnreadPage>>,
    mut pages: Query<(&UnreadPage, &mut Visibility), Without<CompletionMark>>,
    mut mechanisms: Query<(&Mechanism, &mut Transform)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let state = |action| {
        if action == Interaction::Board {
            return 1;
        }
        preview.as_ref().map_or_else(
            || {
                if mission.completed(action) {
                    2
                } else if mission.available(action) {
                    1
                } else {
                    0
                }
            },
            |p| p.index(),
        )
    };
    for (plaque, handle) in &plaques {
        if let Some(mut material) = materials.get_mut(&handle.0) {
            let state = state(plaque.action);
            if material.base_color_texture.as_ref() != Some(&plaque.textures[state]) {
                material.base_color_texture = Some(plaque.textures[state].clone());
            }
        }
    }
    for (lamp, material) in &lamps {
        if let Some(mut material) = materials.get_mut(&material.0) {
            let color = if state(lamp.0) == 2 {
                DONE
            } else if state(lamp.0) == 1 {
                READY
            } else {
                FRAME
            };
            if material.base_color != color {
                material.base_color = color;
            }
        }
    }
    for (mark, mut visible) in &mut marks {
        visible.set_if_neq(if state(mark.0) == 2 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    for (page, mut visible) in &mut pages {
        visible.set_if_neq(if state(Interaction::Scan(page.0)) == 2 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }
    for (mechanism, mut transform) in &mut mechanisms {
        let used = if mechanism.action == Interaction::Chest {
            preview
                .as_ref()
                .map_or(mission.practice_reward, |p| **p != PreviewState::Locked)
        } else {
            state(mechanism.action) == 2
        };
        transform.set_if_neq(if used { mechanism.used } else { mechanism.rest });
    }
}
