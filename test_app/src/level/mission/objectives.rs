//! Mission guidance derived from the single progress state.
use super::*;
#[derive(Clone, Copy)]
pub enum Objective {
    ExploreRemainder,
    FindParts,
    OpenRack,
    CollectPart,
    CraftCell,
    CollectCraftedCell,
    OpenValves,
    CollectWaterCell,
    ScanArchive,
    CollectArchiveCell,
    InstallCells,
    StartSignal,
}
impl Mission {
    pub fn destination(&self) -> Vec3 {
        if !self.parts[0] {
            return map::PART_APPROACH[0];
        }
        if !self.parts[1] {
            return if self.rack_open {
                map::PART_APPROACH[1]
            } else {
                map::RACK
            };
        }
        if self.cells[0] == CellState::Locked {
            return *map::BENCH;
        }
        if self.cells[0] == CellState::Available {
            return map::REWARDS[0];
        }
        if let Some(i) = self.valves.iter().position(|&v| !v) {
            return map::VALVES[i];
        }
        if self.cells[1] == CellState::Available {
            return map::REWARDS[1];
        }
        if let Some(i) = self.scans.iter().position(|&v| !v) {
            return map::TABLETS[i];
        }
        if self.cells[2] == CellState::Available {
            return map::REWARDS[2];
        }
        if self.installed_cells() < 3 {
            return map::GENERATOR;
        }
        map::SIGNAL
    }
    pub fn objective(&self) -> Objective {
        if self.finished {
            Objective::ExploreRemainder
        } else if !self.parts[0] {
            Objective::FindParts
        } else if !self.rack_open {
            Objective::OpenRack
        } else if !self.parts[1] {
            Objective::CollectPart
        } else if self.cells[0] == CellState::Locked {
            Objective::CraftCell
        } else if self.cells[0] == CellState::Available {
            Objective::CollectCraftedCell
        } else if !self.valves.iter().all(|&v| v) {
            Objective::OpenValves
        } else if self.cells[1] == CellState::Available {
            Objective::CollectWaterCell
        } else if !self.scans.iter().all(|&v| v) {
            Objective::ScanArchive
        } else if self.cells[2] == CellState::Available {
            Objective::CollectArchiveCell
        } else if self.installed_cells() < 3 {
            Objective::InstallCells
        } else {
            Objective::StartSignal
        }
    }
}
