//! Shared identities for inventory, pickups and station signs.

use crate::level::mission::Mission;
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
    pub fn count(self, coins: &CoinRun, mission: &Mission) -> usize {
        match self {
            Self::Coin => coins.count(),
            Self::Part => mission.part_count(),
            Self::Cell => mission.carried_cells(),
            Self::Blueprint => mission.blueprint_count(),
        }
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct CoinRun {
    collected: [bool; 24],
}

impl CoinRun {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn collect(&mut self, id: super::CoinId) -> bool {
        if self.collected[id.index()] {
            return false;
        }
        self.collected[id.index()] = true;
        true
    }
    pub fn was_collected(&self, id: super::CoinId) -> bool {
        self.collected[id.index()]
    }
    pub fn collected(&self) -> [bool; 24] {
        self.collected
    }

    pub fn count(&self) -> usize {
        self.collected
            .iter()
            .filter(|&&collected| collected)
            .count()
    }
}
