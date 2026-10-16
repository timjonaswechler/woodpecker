//! Shared identities for inventory, pickups and station signs.

pub(crate) mod coins;

use crate::{expedition::Expedition, items::coins::CoinRun, level::scenery::Scenery};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum ItemKind {
    Coin,
    Part,
    Cell,
    Blueprint,
}
impl ItemKind {
    pub const ALL: [Self; 4] = [Self::Coin, Self::Part, Self::Cell, Self::Blueprint];
    pub fn color(self) -> Color {
        match self {
            Self::Coin => Color::srgb_u8(255, 193, 62),
            Self::Part => Color::srgb_u8(255, 139, 62),
            Self::Cell => Color::srgb_u8(68, 215, 153),
            Self::Blueprint => Color::srgb_u8(83, 170, 255),
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Coin => "Münzen",
            Self::Part => "Bauteile",
            Self::Cell => "Energiezellen",
            Self::Blueprint => "Baupläne",
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            Self::Coin => "ui/coin.png",
            Self::Part => "ui/part.png",
            Self::Cell => "ui/cell.png",
            Self::Blueprint => "ui/blueprint.png",
        }
    }
    pub fn count(self, coins: &CoinRun, expedition: &Expedition) -> usize {
        match self {
            Self::Coin => coins.count(),
            Self::Part => expedition.part_count(),
            Self::Cell => expedition.carried_cells(),
            Self::Blueprint => expedition.blueprint_count(),
        }
    }
}

impl Scenery<'_, '_, '_> {
    /// The same silhouette identifies a pickup and the station that produces it.
    pub fn item(&mut self, kind: ItemKind, transform: Transform) -> Entity {
        let color = kind.color();
        let white = Color::srgb(0.94, 1., 0.98);
        let mut parts = Vec::new();
        match kind {
            ItemKind::Coin | ItemKind::Part => {
                let ring = self
                    .commands
                    .spawn((
                        Mesh3d(self.meshes.add(Torus::new(0.17, 0.33))),
                        MeshMaterial3d(self.materials.add(color)),
                        Transform::from_rotation(Quat::from_rotation_x(
                            std::f32::consts::FRAC_PI_2,
                        )),
                    ))
                    .id();
                parts.push(ring);
                if matches!(kind, ItemKind::Part) {
                    for i in 0..8 {
                        let angle = i as f32 * std::f32::consts::TAU / 8.;
                        let tooth = self.block(
                            Vec3::new(angle.sin(), angle.cos(), 0.) * 0.34,
                            Vec3::new(0.17, 0.22, 0.14),
                            color,
                        );
                        self.commands.entity(tooth).insert(
                            Transform::from_xyz(angle.sin() * 0.34, angle.cos() * 0.34, 0.)
                                .with_rotation(Quat::from_rotation_z(-angle)),
                        );
                        parts.push(tooth);
                    }
                }
            }
            ItemKind::Cell => {
                parts.push(self.block(Vec3::ZERO, Vec3::new(0.44, 0.64, 0.26), color));
                parts.push(self.block(Vec3::Y * 0.37, Vec3::new(0.20, 0.10, 0.18), white));
                for z in [-0.136, 0.136] {
                    parts.push(self.block(
                        Vec3::new(0., 0., z),
                        Vec3::new(0.24, 0.07, 0.012),
                        white,
                    ));
                    parts.push(self.block(
                        Vec3::new(0., 0., z),
                        Vec3::new(0.07, 0.24, 0.012),
                        white,
                    ));
                }
            }
            ItemKind::Blueprint => {
                parts.push(self.block(Vec3::ZERO, Vec3::new(0.60, 0.74, 0.065), color));
                for z in [-0.04, 0.04] {
                    for y in [-0.19, 0., 0.19] {
                        parts.push(self.block(
                            Vec3::new(-0.03, y, z),
                            Vec3::new(0.36, 0.035, 0.01),
                            white,
                        ));
                    }
                    parts.push(self.block(
                        Vec3::new(0.21, 0.28, z),
                        Vec3::new(0.16, 0.16, 0.015),
                        white,
                    ));
                }
            }
        }
        for &part in &parts {
            self.commands
                .entity(part)
                .insert(crate::practice::ShotIgnore);
        }
        let root = self.spawn((
            Name::new(kind.label()),
            transform,
            Visibility::Inherited,
            crate::practice::ShotIgnore,
        ));
        self.commands.entity(root).add_children(&parts);
        root
    }
}
