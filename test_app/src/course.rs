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
pub struct Feedback(pub usize);
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
    };
    for (i, p) in DATA.gears.into_iter().enumerate() {
        let e = s.item(
            ItemKind::Part,
            Transform::from_translation(Vec3::from(p) + Vec3::Y).with_scale(Vec3::splat(1.15)),
        );
        s.commands.entity(e).insert(Pickup::Part(i));
    }
    for (i, p) in map::REWARDS.into_iter().enumerate() {
        let e = s.item(
            ItemKind::Cell,
            Transform::from_translation(p + Vec3::Y * 1.1),
        );
        s.commands.entity(e).insert(Pickup::Cell(i));
    }
    for (i, p) in DATA.blueprints.into_iter().enumerate() {
        let e = s.item(
            ItemKind::Blueprint,
            Transform::from_translation(Vec3::from(p) + Vec3::Y * 0.85),
        );
        s.commands.entity(e).insert(Pickup::Blueprint(i));
    }
    for (i, x) in [56.85, 58., 59.15].into_iter().enumerate() {
        let e = s.item(
            ItemKind::Cell,
            Transform::from_translation(map::point(x, 6.7, 1.52)).with_scale(Vec3::splat(0.9)),
        );
        s.commands.entity(e).insert(Pickup::Socket(i));
    }
    for (i, p) in map::VALVES
        .into_iter()
        .chain(map::TABLETS)
        .chain([map::BENCH, map::GENERATOR, map::SIGNAL])
        .enumerate()
    {
        let e = s.disc(p + Vec3::Y * 0.07, 0.5, ItemKind::Cell.color());
        s.item(
            ItemKind::Cell,
            Transform::from_translation(p + Vec3::new(0., 2.2, -0.8)).with_scale(Vec3::splat(0.7)),
        );
        s.commands.entity(e).insert(Feedback(i));
    }
    // Orange gears mark the part source and the workbench input; green batteries
    // above valves, archive consoles and generator identify the cell task chain.
    for p in [map::RACK, map::BENCH + Vec3::NEG_X * 0.8] {
        s.item(
            ItemKind::Part,
            Transform::from_translation(p + Vec3::new(0., 2.2, -0.8)).with_scale(Vec3::splat(0.7)),
        );
        s.disc(p + Vec3::Y * 0.055, 0.38, ItemKind::Part.color());
    }
    s.item(
        ItemKind::Blueprint,
        Transform::from_translation(map::point(46., 50.35, 1.2)),
    );
    // Water appears in the planted beds after both valves have been opened.
    for (x, y) in [(66., 46.), (72., 46.), (67., 51.)] {
        let e = s.block(
            map::point(x, y, 0.68),
            Vec3::new(3.6, 0.04, 1.9),
            Color::srgb(0.2, 0.65, 0.85),
        );
        s.commands.entity(e).insert(Feedback(8));
    }
    let e = s.block(CHEST_CENTER, CHEST_SIZE, ItemKind::Blueprint.color());
    s.commands.entity(e).insert(Feedback(9));
    let e = s.block(
        map::point(73., 12.5, 6.6),
        Vec3::new(2., 0.4, 0.15),
        Color::srgb(1., 0.8, 0.2),
    );
    s.commands.entity(e).insert(Feedback(10));
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
                DATA.gears[i][1] + 1.,
            ),
            Pickup::Cell(i) => (mission.cells[i] == CellState::Available, 1.1),
            Pickup::Socket(i) => (mission.cells[i] == CellState::Installed, 1.52),
            Pickup::Blueprint(i) => (
                !mission.blueprints[i]
                    && (i != 4 || mission.rack_open)
                    && (i != 5 || mission.practice_reward),
                DATA.blueprints[i][1] + 0.85,
            ),
        };
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if !matches!(kind, Pickup::Socket(_)) {
            transform.translation.y = base + (time.elapsed_secs() * 2.).sin() * 0.1;
            transform.rotation = Quat::from_rotation_y(time.elapsed_secs() * 0.8);
        }
    }
    for (kind, mut visibility, handle) in &mut feedback {
        let active = match kind.0 {
            0..=1 => mission.valves[kind.0],
            2..=4 => mission.scans[kind.0 - 2],
            5 => mission.cells[0] != CellState::Locked,
            6 => mission.installed_cells() == 3,
            7 | 10 => mission.finished,
            8 => mission.valves.iter().all(|&v| v),
            9 => mission.chest_open,
            _ => false,
        };
        *visibility = if (kind.0 == 8 || kind.0 == 10) && !active {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        if let Some(mut mat) = materials.get_mut(&handle.0) {
            mat.base_color = match kind.0 {
                8 | 9 => ItemKind::Blueprint.color(),
                10 => crate::level::GOLD,
                _ => ItemKind::Cell.color(),
            };
            mat.emissive = if active {
                LinearRgba::new(0.25, 0.4, 0.35, 1.)
            } else {
                LinearRgba::BLACK
            };
        }
    }
    for (entity, name, mut transform, original) in &mut models {
        if name.as_str() != "Movable storage rack" && !name.as_str().starts_with("Valve handwheel")
        {
            continue;
        }
        let base = original.map_or(*transform, |p| p.0);
        if original.is_none() {
            commands.entity(entity).insert(OriginalPose(base));
        }
        if name.as_str() == "Movable storage rack" {
            transform.translation =
                base.translation + Vec3::NEG_Z * if mission.rack_open { 3. } else { 0. };
        } else {
            let i = usize::from(name.as_str().ends_with(".001"));
            transform.rotation = base.rotation
                * Quat::from_rotation_z(if mission.valves[i] {
                    std::f32::consts::FRAC_PI_2
                } else {
                    0.
                });
        }
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

/// Runtime category signs replace the plan's generic orange interaction rings.
pub fn hide_planning_markers(
    ready: On<bevy::world_serialization::WorldInstanceReady>,
    children: Query<&Children>,
    mut nodes: Query<(&Name, &mut Visibility)>,
) {
    for entity in children.iter_descendants(ready.entity) {
        if let Ok((name, mut visibility)) = nodes.get_mut(entity)
            && (name.as_str().contains("interaction ring")
                || name.as_str().starts_with("Practice start"))
        {
            *visibility = Visibility::Hidden;
        }
    }
}
