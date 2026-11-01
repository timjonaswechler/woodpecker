mod model_parts;
pub(crate) mod pickups;
pub(crate) mod shooting_range;
use crate::level::mission::inventory::ItemKind;
use model_parts::*;
use std::f32::consts::{FRAC_PI_2, PI};
mod board;
mod checkpoint;
mod chest;
mod generator;
mod scanner;
mod storage_rack;
mod transmitter;
mod valve;
mod workbench;

pub(crate) mod definitions;

use crate::{
    level::mission::{Interaction, Mission},
    level::{LevelAssets, scene::Scenery},
};
use bevy::prelude::*;

pub use definitions::colliders;
use definitions::{INTERACTIVE_OBJECTS, ObjectDefinition, PICKUP_TABLES};

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

#[derive(Component)]
pub struct ObjectPlaque {
    action: Interaction,
    textures: [Handle<Image>; 3],
}
#[derive(Component)]
pub struct ObjectLamp(pub Interaction);
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

pub fn spawn_objects(
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
    for t in *INTERACTIVE_OBJECTS {
        spawn_instance(&mut s, t);
    }
    for t in *PICKUP_TABLES {
        spawn_instance(&mut s, t);
    }
}

/// Model-review override. Normal gameplay always derives visuals from Mission.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Default)]
pub enum VisualState {
    Locked,
    #[default]
    Ready,
    Used,
}

impl VisualState {
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
pub fn update_visuals(
    mission: Option<Res<Mission>>,
    preview: Option<Res<VisualState>>,
    lamps: Query<(&ObjectLamp, &MeshMaterial3d<StandardMaterial>)>,
    plaques: Query<(&ObjectPlaque, &MeshMaterial3d<StandardMaterial>)>,
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
                let mission = mission
                    .as_ref()
                    .expect("game visuals need mission or preview state");
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
        visible.set_if_neq(
            if state(Interaction::Scan(
                crate::level::mission::ScanId::ALL[page.0],
            )) == 2
            {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            },
        );
    }
    for (mechanism, mut transform) in &mut mechanisms {
        let used = if mechanism.action == Interaction::Chest {
            preview.as_ref().map_or_else(
                || mission.as_ref().is_some_and(|m| m.practice_reward()),
                |p| **p != VisualState::Locked,
            )
        } else {
            state(mechanism.action) == 2
        };
        transform.set_if_neq(if used { mechanism.used } else { mechanism.rest });
    }
}
fn spawn_instance(s: &mut Scenery, target: ObjectDefinition) -> Entity {
    let base = Vec3::new(target.center.x, target.floor, target.center.z);
    let root = s.spawn((
        Name::new(target.name),
        Transform::from_translation(base),
        Visibility::Inherited,
    ));
    let previous = s.parent.replace(root);
    match target.action {
        Interaction::Board => board::spawn(s),
        Interaction::Craft => workbench::spawn(s, target.action),
        Interaction::Rack => storage_rack::spawn(s, target.action),
        Interaction::Valve(_) => valve::spawn(s, target.action),
        Interaction::Scan(i) => scanner::spawn(s, i.index(), target.action),
        Interaction::Install => generator::spawn(s, target.action),
        Interaction::Signal => transmitter::spawn(s, target.action),
        Interaction::Checkpoint => checkpoint::spawn(s, target.action),
        Interaction::Chest => chest::spawn(s, target.action),
        Interaction::Part(_) | Interaction::Cell(_) => pickups::spawn_table(s, target),
        Interaction::Portal(_) | Interaction::None => {}
    }
    s.parent = previous;
    root
}

pub(crate) use storage_rack::{
    sync_collision, update_collision, update_model as update_rack_model,
};

/// Model identity is independent of placement and mission state.
#[derive(Clone, Copy)]
pub enum ModelKind {
    Board,
    Workbench,
    StorageLever,
    Valve,
    Scanner,
    Generator,
    Transmitter,
    Checkpoint,
    Chest,
    PartTable,
    AssemblyTable,
    WaterTable,
    ArchiveTable,
}
impl ModelKind {
    fn definition(self) -> ObjectDefinition {
        match self {
            Self::Board => INTERACTIVE_OBJECTS[0],
            Self::Workbench => INTERACTIVE_OBJECTS[1],
            Self::StorageLever => INTERACTIVE_OBJECTS[2],
            Self::Valve => INTERACTIVE_OBJECTS[3],
            Self::Scanner => INTERACTIVE_OBJECTS[5],
            Self::Generator => INTERACTIVE_OBJECTS[8],
            Self::Transmitter => INTERACTIVE_OBJECTS[9],
            Self::Checkpoint => INTERACTIVE_OBJECTS[10],
            Self::Chest => INTERACTIVE_OBJECTS[11],
            Self::PartTable => PICKUP_TABLES[0],
            Self::AssemblyTable => PICKUP_TABLES[2],
            Self::WaterTable => PICKUP_TABLES[3],
            Self::ArchiveTable => PICKUP_TABLES[4],
        }
    }
    pub fn name(self) -> &'static str {
        self.definition().name
    }
    pub fn sample_item(self) -> Option<ItemKind> {
        match self {
            Self::PartTable => Some(ItemKind::Part),
            Self::AssemblyTable | Self::WaterTable | Self::ArchiveTable => Some(ItemKind::Cell),
            Self::Chest => Some(ItemKind::Blueprint),
            _ => None,
        }
    }
}
/// Both game and zoo use the same geometry. Only the game installs its systems.
pub fn spawn_model(scene: &mut Scenery, model: ModelKind, at: Vec3) -> Entity {
    let mut definition = model.definition();
    definition.center = at + Vec3::Y * definition.half.y;
    definition.floor = at.y;
    spawn_instance(scene, definition)
}

pub(crate) use workbench::can_produce;
