//! Interactive props and feedback for the room tasks.
mod models;
pub(super) use models::spawn_table;
pub(crate) mod coins;
use crate::{
    level::LevelAssets,
    level::definition::{self as map, DATA},
    level::mission::inventory::ItemKind,
    level::mission::{CellState, Mission},
    level::scene::Scenery,
};
use bevy::prelude::*;

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
pub fn spawn(
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
    for (i, p) in DATA.gears().into_iter().enumerate() {
        let e = s.item(
            ItemKind::Part,
            Transform::from_translation(Vec3::from(p) + Vec3::Y * 0.94)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(1.15)),
        );
        s.commands.entity(e).insert(Pickup::Part(i));
    }
    for (i, p) in (*map::CELL_ITEMS).into_iter().enumerate() {
        let e = s.item(ItemKind::Cell, Transform::from_translation(p));
        s.commands.entity(e).insert(Pickup::Cell(i));
    }
    for (i, p) in DATA.blueprints().into_iter().enumerate() {
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
pub fn update(
    time: Res<Time>,
    mission: Res<Mission>,
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
) {
    for (kind, mut transform, mut visibility) in &mut pickups {
        let (visible, base) = match *kind {
            Pickup::Part(i) => (
                !mission.parts()[i] && (i == 0 || mission.rack_open()),
                DATA.gears()[i][1] + 0.94,
            ),
            Pickup::Cell(i) => (
                mission.cells()[i] == CellState::Available,
                map::CELL_ITEMS[i].y,
            ),
            Pickup::Socket(i) => (mission.cells()[i] == CellState::Installed, 1.72),
            Pickup::Blueprint(i) => (
                !mission.blueprints()[i]
                    && (i != 4 || mission.rack_open())
                    && (i != 5 || mission.practice_reward()),
                DATA.blueprints()[i][1] + 0.85,
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
            Feedback::Water => mission.valves().iter().all(|&v| v),
            Feedback::Signal => mission.finished(),
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
}
