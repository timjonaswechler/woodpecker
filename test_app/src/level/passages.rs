//! Ordinary secret passages: physical sliding leaves, exploration, route access and cutaways.
use super::{
    definition as map,
    mission::{Mission, PortalId},
    navigation::{BODY_HALF, BODY_HEIGHT, Navigation, WalkRegion},
};
use crate::player::{Player, camera::ViewState};
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize)]
pub(super) struct PortalDefinition {
    pub name: String,
    pub anchor: [f32; 3],
    pub center: [f32; 3],
    pub half: [f32; 3],
    pub inward: [f32; 3],
    pub slide: [f32; 3],
    pub outside: bool,
}
#[derive(Deserialize)]
pub(super) struct PassageDefinition {
    pub id: String,
    pub floor: f32,
    pub volumes: Vec<[f32; 4]>,
    pub portals: [PortalDefinition; 2],
}
impl PassageDefinition {
    fn regions(&self, margin: f32, height: f32) -> impl Iterator<Item = WalkRegion> + '_ {
        self.volumes.iter().map(move |[x0, y0, x1, y1]| WalkRegion {
            center: map::point((x0 + x1) / 2., (y0 + y1) / 2., height),
            half: Vec2::new((x1 - x0) / 2. + margin, (y1 - y0) / 2. + margin),
        })
    }
    fn contains(&self, feet: Vec3, margin: f32) -> bool {
        self.regions(margin, self.floor).any(|r| r.contains(feet))
    }
    fn contains_body(&self, feet: Vec3, margin: f32) -> bool {
        // A legal jump is on this floor, unlike walking below the upper passage.
        feet.y >= self.floor - 0.1
            && feet.y < self.floor + 1.
            && self.contains(Vec3::new(feet.x, self.floor, feet.z), margin)
    }
}
#[derive(Deserialize)]
pub(super) struct CutawayDefinition {
    pub name: String,
    pub secret: usize,
    pub portal: i8,
    pub surface: Option<[f32; 4]>,
}

pub(super) fn portal(id: usize) -> &'static PortalDefinition {
    &map::DATA.passages[id / 2].portals[id % 2]
}
fn threshold(id: usize) -> Vec3 {
    let p = portal(id);
    Vec3::from(p.center) - Vec3::Y * p.half[1]
}
pub(super) fn accessible(id: PortalId, feet: Vec3) -> bool {
    let i = id.index();
    let p = portal(i);
    p.outside
        || (map::DATA.passages[i / 2].contains(feet, 0.)
            && (feet - threshold(i)).dot(Vec3::from(p.inward)) > 0.1)
}

#[derive(Resource, Default)]
pub(crate) struct Passages {
    travel: [f32; 6],
    previous: Option<Vec3>,
}
impl Passages {
    fn pose(&self, id: usize) -> Vec3 {
        let p = portal(id);
        Vec3::from(p.center) + Vec3::from(p.slide) * self.travel[id]
    }
    fn clear(&self, id: usize, nav: &Navigation) -> bool {
        let normal = Vec3::from(portal(id).inward);
        nav.clear_segment(threshold(id) - normal, threshold(id) + normal)
    }
    fn active(&self, i: usize, mission: &Mission, player: &Player, view: &ViewState) -> bool {
        !view.first_person()
            && mission.secrets()[i]
            && map::DATA.passages[i].contains_body(player.feet(), 1. + BODY_HALF)
    }
    fn cover_hidden(
        &self,
        cut: &CutawayDefinition,
        mission: &Mission,
        player: &Player,
        view: &ViewState,
        nav: &Navigation,
    ) -> bool {
        !view.first_person()
            && (self.active(cut.secret, mission, player, view)
                || (cut.portal >= 0 && self.clear(cut.portal as usize, nav)))
    }
    pub(super) fn sync_access(
        &self,
        mission: &Mission,
        player: &mut Player,
        view: &ViewState,
        nav: &mut Navigation,
    ) {
        let mut restricted = Vec::new();
        let mut entries = Vec::new();
        for (i, passage) in map::DATA.passages.iter().enumerate() {
            if mission.secrets()[i] {
                continue;
            }
            restricted.extend(passage.regions(0., passage.floor));
            for j in 0..2 {
                let id = i * 2 + j;
                if mission.portals()[id] {
                    let normal = Vec3::from(portal(id).inward);
                    entries.push(WalkRegion {
                        center: threshold(id) + normal * 0.6,
                        half: if normal.x.abs() > 0.5 {
                            Vec2::new(0.6, 1.5)
                        } else {
                            Vec2::new(1.5, 0.6)
                        },
                    });
                }
            }
        }
        if nav.restrict_routes(restricted, entries) {
            player.clear_route();
        }
        let mut names = HashSet::new();
        let mut floors = Vec::new();
        for cut in &map::DATA.cutaways {
            if !self.cover_hidden(cut, mission, player, view, nav) {
                continue;
            }
            names.insert(cut.name.clone());
            if let Some([x0, y0, x1, y1]) = cut.surface {
                floors.push(WalkRegion {
                    center: map::point(
                        (x0 + x1) / 2.,
                        (y0 + y1) / 2.,
                        map::DATA.passages[cut.secret].floor + 3.,
                    ),
                    half: Vec2::new((x1 - x0) / 2., (y1 - y0) / 2.),
                });
            }
        }
        nav.set_cutaway(names, floors);
    }
    pub(crate) fn reset(
        &mut self,
        mission: &Mission,
        player: &mut Player,
        view: &ViewState,
        nav: &mut Navigation,
    ) {
        *self = Self::default();
        for i in 0..6 {
            nav.set_obstacle_pose(&portal(i).name, self.pose(i))
                .expect("checked portal");
        }
        self.sync_access(mission, player, view, nav);
    }
    pub(super) fn advance(
        &mut self,
        dt: f32,
        mission: &Mission,
        nav: &mut Navigation,
        player: &mut Player,
    ) -> bool {
        let mut blocked = false;
        for i in 0..6 {
            if !mission.portals()[i] || self.travel[i] >= 1. {
                continue;
            }
            let p = portal(i);
            let next = (self.travel[i] + dt.clamp(0., 0.05) * 1.4).min(1.);
            let before = self.pose(i);
            let after = Vec3::from(p.center) + Vec3::from(p.slide) * next;
            let swept_center = (before + after) / 2.;
            let swept_half = Vec3::from(p.half) + (after - before).abs() / 2.;
            let body_center = player.feet() + Vec3::Y * BODY_HEIGHT / 2.;
            if (body_center - swept_center)
                .abs()
                .cmplt(swept_half + Vec3::new(BODY_HALF, BODY_HEIGHT / 2., BODY_HALF))
                .all()
            {
                blocked = true;
                continue;
            }
            self.travel[i] = next;
            if nav
                .set_obstacle_pose(&p.name, after)
                .expect("checked portal")
            {
                player.clear_route();
            }
        }
        blocked
    }
    pub(super) fn discover(
        &mut self,
        mission: &mut Mission,
        player: &Player,
        nav: &Navigation,
    ) -> bool {
        let current = player.feet();
        let mut found = false;
        if let Some(previous) = self.previous {
            for i in 0..3 {
                let passage = &map::DATA.passages[i];
                if mission.secrets()[i] || !passage.contains_body(current, 0.) {
                    continue;
                }
                for j in 0..2 {
                    let id = i * 2 + j;
                    let p = portal(id);
                    let normal = Vec3::from(p.inward);
                    let plane = threshold(id) + normal * 0.8;
                    let a = (previous - plane).dot(normal);
                    let b = (current - plane).dot(normal);
                    let sideways = (current - threshold(id)) * (Vec3::ONE - normal.abs());
                    let in_opening = sideways.xz().length() < 1.5 - BODY_HALF + 0.01;
                    if mission.portals()[id]
                        && a < 0.
                        && b >= 0.
                        && in_opening
                        && passage.contains_body(previous, 1.)
                        && nav.clear_body_segment(previous, current)
                    {
                        mission.discover(i);
                        found = true;
                        break;
                    }
                }
            }
        }
        self.previous = Some(current);
        found
    }
}

pub(super) fn initialize(nav: &mut Navigation) {
    let regions = map::DATA
        .passages
        .iter()
        .flat_map(|p| p.regions(0., p.floor))
        .collect();
    nav.restrict_routes(regions, Vec::new());
}

#[derive(Component)]
pub(super) struct PortalModel(usize);
#[derive(Component)]
pub(super) struct PassageCover(pub usize);

pub(super) fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for i in 0..6 {
        let p = portal(i);
        let color = match i / 2 {
            0 => Color::srgb(0.32, 0.37, 0.35),
            1 => Color::srgb(0.48, 0.39, 0.28),
            _ => Color::srgb(0.83, 0.83, 0.78),
        };
        let root = commands
            .spawn((
                Name::new(p.name.clone()),
                PortalModel(i),
                Transform::from_translation(p.center.into()),
                Visibility::Inherited,
            ))
            .id();
        let mesh = meshes.add(Cuboid::from_size(Vec3::from(p.half) * 2.));
        let material = materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.85,
            ..default()
        });
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::default(),
            ChildOf(root),
        ));
        // Physical props have ordinary details, not floating secret markers.
        let normal = Vec3::from(p.inward);
        let front = -normal
            * (if normal.x.abs() > 0.5 {
                p.half[0]
            } else {
                p.half[2]
            });
        let tangent = Vec3::new(normal.z, 0., -normal.x);
        if i == 2 {
            let book = meshes.add(Cuboid::new(0.24, 0.5, 0.08));
            let colors = [
                Color::srgb(0.23, 0.4, 0.5),
                Color::srgb(0.6, 0.47, 0.32),
                Color::srgb(0.48, 0.55, 0.45),
            ];
            let book_materials = colors.map(|color| {
                materials.add(StandardMaterial {
                    base_color: color,
                    perceptual_roughness: 0.85,
                    ..default()
                })
            });
            for row in -1..=1 {
                for column in -2..=2 {
                    commands.spawn((
                        Mesh3d(book.clone()),
                        MeshMaterial3d(book_materials[(column + 2) as usize % 3].clone()),
                        Transform::from_translation(
                            front - normal * 0.025
                                + tangent * column as f32 * 0.45
                                + Vec3::Y * row as f32 * 0.7,
                        ),
                        ChildOf(root),
                    ));
                }
            }
        }
        if i < 2 {
            let size = if normal.x.abs() > 0.5 {
                Vec3::new(0.03, 0.025, 2.8)
            } else {
                Vec3::new(2.8, 0.025, 0.03)
            };
            let strip = meshes.add(Cuboid::from_size(size));
            let seam = materials.add(StandardMaterial {
                base_color: Color::srgb(0.17, 0.23, 0.23),
                ..default()
            });
            for height in [-0.65, 0., 0.65] {
                commands.spawn((
                    Mesh3d(strip.clone()),
                    MeshMaterial3d(seam.clone()),
                    Transform::from_translation(front + Vec3::Y * height),
                    ChildOf(root),
                ));
            }
        }
        let size = if normal.x.abs() > 0.5 {
            Vec3::new(0.05, 0.5, 0.08)
        } else {
            Vec3::new(0.08, 0.5, 0.05)
        };
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(size))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.24, 0.23),
                ..default()
            })),
            Transform::from_translation(front + Vec3::Y * 0.1),
            ChildOf(root),
        ));
    }
}

pub(super) fn prepare(
    time: Res<Time>,
    flow: Res<crate::app::flow::GameFlow>,
    mission: Res<Mission>,
    view: Res<ViewState>,
    mut passages: ResMut<Passages>,
    mut nav: ResMut<Navigation>,
    mut player: ResMut<Player>,
) {
    if !flow.menu() {
        passages.advance(time.delta_secs(), &mission, &mut nav, &mut player);
    }
    passages.sync_access(&mission, &mut player, &view, &mut nav);
}
pub(super) fn progress(
    mut passages: ResMut<Passages>,
    mut mission: ResMut<Mission>,
    mut player: ResMut<Player>,
    view: Res<ViewState>,
    mut nav: ResMut<Navigation>,
    mut ui: ResMut<crate::ui::UiState>,
) {
    if passages.discover(&mut mission, &player, &nav) {
        ui.show("Verborgener Weg entdeckt. Der Bauplan liegt in einer Nische; der andere Ausgang lässt sich innen öffnen.".into());
    }
    passages.sync_access(&mission, &mut player, &view, &mut nav);
}
pub(super) fn present(
    passages: Res<Passages>,
    mission: Res<Mission>,
    player: Res<Player>,
    view: Res<ViewState>,
    nav: Res<Navigation>,
    mut doors: Query<(&PortalModel, &mut Transform)>,
    mut covers: Query<(&PassageCover, &mut Visibility)>,
) {
    for (model, mut transform) in &mut doors {
        transform.translation = passages.pose(model.0);
    }
    for (cover, mut visibility) in &mut covers {
        *visibility = if passages.cover_hidden(
            &map::DATA.cutaways[cover.0],
            &mission,
            &player,
            &view,
            &nav,
        ) {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

pub(super) fn targets() -> impl Iterator<Item = super::objects::definitions::ObjectDefinition> {
    (0..6).map(|i| {
        let p = portal(i);
        super::objects::definitions::ObjectDefinition::new(
            &p.name,
            super::mission::Interaction::Portal(PortalId::ALL[i]),
            threshold(i),
            Vec3::from(p.half) * 2.,
        )
    })
}
