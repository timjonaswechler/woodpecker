//! Interactive props and feedback for the room tasks.
use crate::{
    blueprint::{self as map, DATA},
    expedition::{CellState, Expedition},
    items::ItemKind,
    level::LevelAssets,
    navigation::Navigation,
    player::Player,
    scenery::Scenery,
};
use bevy::prelude::*;
pub const CHEST_CENTER: Vec3 = map::point(46., 51., 0.4);
pub const CHEST_SIZE: Vec3 = Vec3::new(1.6, 0.8, 1.2);

#[derive(Component)]
pub enum Pickup {
    Part(usize),
    Cell(usize),
    Socket(usize),
    Blueprint(usize),
}
#[derive(Component)]
pub enum Feedback {
    Water,
    Signal,
}
#[derive(Component)]
pub struct OriginalPose(pub Transform);
pub fn spawn_course(
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
    for (i, p) in DATA.gears.into_iter().enumerate() {
        let e = s.item(
            ItemKind::Part,
            Transform::from_translation(Vec3::from(p) + Vec3::Y * 0.94)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(1.15)),
        );
        s.commands.entity(e).insert(Pickup::Part(i));
    }
    for (i, p) in map::CELL_ITEMS.into_iter().enumerate() {
        let e = s.item(ItemKind::Cell, Transform::from_translation(p));
        s.commands.entity(e).insert(Pickup::Cell(i));
    }
    for (i, p) in DATA.blueprints.into_iter().enumerate() {
        let e = s.item(
            ItemKind::Blueprint,
            if i == 5 {
                Transform::from_translation(Vec3::from(p) + Vec3::Y * 0.72)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            } else {
                Transform::from_translation(Vec3::from(p) + Vec3::Y * 0.85)
            },
        );
        s.commands.entity(e).insert(Pickup::Blueprint(i));
    }
    for (i, x) in [56.85, 58., 59.15].into_iter().enumerate() {
        let e = s.item(
            ItemKind::Cell,
            Transform::from_translation(map::point(x, 6.7, 1.72)).with_scale(Vec3::splat(0.9)),
        );
        s.commands.entity(e).insert(Pickup::Socket(i));
    }
    // Water appears in the planted beds after both valves have been opened.
    for (x, y) in [(66., 46.), (72., 46.), (67., 51.)] {
        let e = s.block(
            map::point(x, y, 0.68),
            Vec3::new(3.6, 0.04, 1.9),
            Color::srgb(0.2, 0.65, 0.85),
        );
        s.commands.entity(e).insert(Feedback::Water);
    }
    let e = s.block(
        map::point(73., 12.5, 6.6),
        Vec3::new(2., 0.4, 0.15),
        Color::srgb(1., 0.8, 0.2),
    );
    s.commands.entity(e).insert(Feedback::Signal);
}
#[expect(
    clippy::too_many_arguments,
    reason = "Props reflect task state in the scene and collision world."
)]
pub fn update_course(
    mut commands: Commands,
    time: Res<Time>,
    mission: Res<Expedition>,
    mut nav: ResMut<Navigation>,
    mut player: ResMut<Player>,
    mut previous_rack: Local<bool>,
    mut pickups: Query<(&Pickup, &mut Transform, &mut Visibility)>,
    mut feedback: Query<
        (
            &Feedback,
            &mut Visibility,
            &MeshMaterial3d<StandardMaterial>,
        ),
        Without<Pickup>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut models: Query<(Entity, &Name, &mut Transform, Option<&OriginalPose>), Without<Pickup>>,
) {
    for (kind, mut transform, mut visibility) in &mut pickups {
        let (visible, base) = match *kind {
            Pickup::Part(i) => (
                !mission.parts[i] && (i == 0 || mission.rack_open),
                DATA.gears[i][1] + 0.94,
            ),
            Pickup::Cell(i) => (
                mission.cells[i] == CellState::Available,
                map::CELL_ITEMS[i].y,
            ),
            Pickup::Socket(i) => (mission.cells[i] == CellState::Installed, 1.72),
            Pickup::Blueprint(i) => (
                !mission.blueprints[i]
                    && (i != 4 || mission.rack_open)
                    && (i != 5 || mission.practice_reward),
                DATA.blueprints[i][1] + 0.85,
            ),
        };
        visibility.set_if_neq(if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
        if matches!(kind, Pickup::Blueprint(i) if *i != 5) {
            transform.translation.y = base + (time.elapsed_secs() * 2.).sin() * 0.1;
            transform.rotation = Quat::from_rotation_y(time.elapsed_secs() * 0.8);
        }
    }
    for (kind, mut visibility, handle) in &mut feedback {
        let active = match kind {
            Feedback::Water => mission.valves.iter().all(|&v| v),
            Feedback::Signal => mission.finished,
        };
        visibility.set_if_neq(if !active {
            Visibility::Hidden
        } else {
            Visibility::Visible
        });
        if let Some(mut mat) = materials.get_mut(&handle.0) {
            let color = match kind {
                Feedback::Water => ItemKind::Blueprint.color(),
                Feedback::Signal => crate::level::GOLD,
            };
            let emissive = if active {
                LinearRgba::new(0.25, 0.4, 0.35, 1.)
            } else {
                LinearRgba::BLACK
            };
            if mat.base_color != color {
                mat.base_color = color;
            }
            if mat.emissive != emissive {
                mat.emissive = emissive;
            }
        }
    }
    for (entity, name, mut transform, original) in &mut models {
        if name.as_str() != "Movable storage rack" {
            continue;
        }
        let base = original.map_or(*transform, |p| p.0);
        if original.is_none() {
            commands.entity(entity).insert(OriginalPose(base));
        }
        transform.translation =
            base.translation + Vec3::NEG_Z * if mission.rack_open { 3. } else { 0. };
    }
    if mission.rack_open != *previous_rack {
        if let Some(obstacle) = nav
            .obstacles
            .iter_mut()
            .find(|o| o.name == "Movable storage rack")
        {
            obstacle.center[2] += if mission.rack_open { -3. } else { 3. };
            nav.rebuild();
            player.route.clear();
        }
        *previous_rack = mission.rack_open;
    }
}

/// Runtime props replace the generic planning consoles and invisible trigger areas.
pub fn hide_planning_markers(
    ready: On<bevy::world_serialization::WorldInstanceReady>,
    children: Query<&Children>,
    mut nodes: Query<(&Name, &mut Visibility)>,
) {
    for entity in children.iter_descendants(ready.entity) {
        if let Ok((name, mut visibility)) = nodes.get_mut(entity)
            && (name.as_str().contains("interaction ring")
                || crate::stations::replaces(name.as_str()))
        {
            *visibility = Visibility::Hidden;
        }
    }
}
