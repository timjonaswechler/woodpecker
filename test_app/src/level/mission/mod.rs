//! Room tasks, inventory and optional discoveries. No rendering or movement rules here.
mod ids;
pub mod inventory;
mod objectives;
pub use ids::{CellId, CoinId, PartId, ScanId, TargetId, ValveId};
pub use objectives::Objective;

use crate::{
    level::definition::{self as map, DATA},
    level::navigation::Navigation,
    level::objects::shooting_range::ShootingRange,
    player::Player,
    player::camera::ViewState,
    ui::UiState,
};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub enum CellState {
    #[default]
    Locked,
    Available,
    Carried,
    Installed,
}
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct Mission {
    parts: [bool; 2],
    valves: [bool; 2],
    scans: [bool; 3],
    cells: [CellState; 3],
    blueprints: [bool; 6],
    secrets: [bool; 3],
    rack_open: bool,
    practice_reward: bool,
    chest_claimed: bool,
    checkpoint: bool,
    elapsed: f32,
    finished: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interaction {
    Part(PartId),
    Rack,
    Valve(ValveId),
    Scan(ScanId),
    Craft,
    Cell(CellId),
    Install,
    Signal,
    Chest,
    Checkpoint,
    Board,
    None,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    PartCollected,
    RackOpened,
    WaterStarted,
    ValveOpened,
    ArchiveCompleted,
    PageScanned,
    CellProduced,
    CellCollected,
    CellsInstalled,
    SignalEnabled,
    ChestClaimed,
    CheckpointSet,
    Briefing,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionRejected {
    Unavailable,
}

fn near(player: &Player, point: Vec3, nav: &Navigation) -> bool {
    player.position().distance(point.xz()) < 1.75
        && (player.height() - point.y).abs() < 0.35
        && nav.clear_segment(player.feet(), point)
}
impl Mission {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn finished(&self) -> bool {
        self.finished
    }

    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    pub fn checkpoint(&self) -> bool {
        self.checkpoint
    }

    pub fn chest_claimed(&self) -> bool {
        self.chest_claimed
    }

    pub fn practice_reward(&self) -> bool {
        self.practice_reward
    }

    pub fn rack_open(&self) -> bool {
        self.rack_open
    }

    pub fn secrets(&self) -> [bool; 3] {
        self.secrets
    }

    pub fn blueprints(&self) -> [bool; 6] {
        self.blueprints
    }

    pub fn cells(&self) -> [CellState; 3] {
        self.cells
    }

    pub fn scans(&self) -> [bool; 3] {
        self.scans
    }

    pub fn valves(&self) -> [bool; 2] {
        self.valves
    }

    pub fn complete_range(&mut self, _: super::objects::shooting_range::RangeCompleted) -> bool {
        if self.practice_reward {
            false
        } else {
            self.practice_reward = true;
            true
        }
    }
    pub fn parts(&self) -> [bool; 2] {
        self.parts
    }

    /// Crafted parts and installed cells are no longer in the backpack.
    pub fn part_count(&self) -> usize {
        if self.cells[0] == CellState::Locked {
            self.parts.iter().filter(|&&v| v).count()
        } else {
            0
        }
    }
    pub fn carried_cells(&self) -> usize {
        self.cells
            .iter()
            .filter(|&&s| s == CellState::Carried)
            .count()
    }
    pub fn collected_cells(&self) -> usize {
        self.cells
            .iter()
            .filter(|&&s| matches!(s, CellState::Carried | CellState::Installed))
            .count()
    }
    pub fn installed_cells(&self) -> usize {
        self.cells
            .iter()
            .filter(|&&s| s == CellState::Installed)
            .count()
    }
    pub fn blueprint_count(&self) -> usize {
        self.blueprints.iter().filter(|&&v| v).count()
    }
    pub fn available(&self, action: Interaction) -> bool {
        match action {
            Interaction::Part(i) => !self.parts[i.index()] && (i.index() == 0 || self.rack_open),
            Interaction::Rack => !self.rack_open,
            Interaction::Valve(i) => !self.valves[i.index()],
            Interaction::Scan(i) => !self.scans[i.index()],
            Interaction::Craft => super::objects::can_produce(self.parts, self.cells[0]),
            Interaction::Cell(i) => self.cells[i.index()] == CellState::Available,
            Interaction::Install => self.carried_cells() > 0,
            Interaction::Signal => !self.finished && self.installed_cells() == 3,
            Interaction::Chest => self.practice_reward && !self.chest_claimed,
            Interaction::Checkpoint => !self.checkpoint,
            Interaction::Board => true,
            Interaction::None => false,
        }
    }
    pub fn completed(&self, action: Interaction) -> bool {
        match action {
            Interaction::Part(i) => self.parts[i.index()],
            Interaction::Rack => self.rack_open,
            Interaction::Valve(i) => self.valves[i.index()],
            Interaction::Scan(i) => self.scans[i.index()],
            Interaction::Craft => self.cells[0] != CellState::Locked,
            Interaction::Cell(i) => {
                matches!(
                    self.cells[i.index()],
                    CellState::Carried | CellState::Installed
                )
            }
            Interaction::Install => self.installed_cells() == 3,
            Interaction::Signal => self.finished,
            Interaction::Chest => self.chest_claimed,
            Interaction::Checkpoint => self.checkpoint,
            Interaction::Board | Interaction::None => false,
        }
    }
    pub fn interaction(&self, player: &Player, nav: &Navigation, view: &ViewState) -> Interaction {
        crate::level::interactions::select(self, player, nav, view)
    }
    /// State transitions enforce prerequisites even outside the UI dispatcher.
    pub fn try_apply(&mut self, action: Interaction) -> Result<Outcome, ActionRejected> {
        if !self.available(action) {
            return Err(ActionRejected::Unavailable);
        }
        let outcome = match action {
            Interaction::Part(i) => {
                self.parts[i.index()] = true;
                Outcome::PartCollected
            }
            Interaction::Rack => {
                self.rack_open = true;
                Outcome::RackOpened
            }
            Interaction::Valve(i) => {
                self.valves[i.index()] = true;
                if self.valves.iter().all(|&v| v) {
                    self.cells[1] = CellState::Available;
                    Outcome::WaterStarted
                } else {
                    Outcome::ValveOpened
                }
            }
            Interaction::Scan(i) => {
                self.scans[i.index()] = true;
                if self.scans.iter().all(|&v| v) {
                    self.cells[2] = CellState::Available;
                    Outcome::ArchiveCompleted
                } else {
                    Outcome::PageScanned
                }
            }
            Interaction::Craft => {
                self.cells[0] = CellState::Available;
                Outcome::CellProduced
            }
            Interaction::Cell(i) => {
                self.cells[i.index()] = CellState::Carried;
                Outcome::CellCollected
            }
            Interaction::Install => {
                for cell in &mut self.cells {
                    if *cell == CellState::Carried {
                        *cell = CellState::Installed;
                    }
                }
                Outcome::CellsInstalled
            }
            Interaction::Signal => {
                self.finished = true;
                Outcome::SignalEnabled
            }
            Interaction::Chest => {
                self.chest_claimed = true;
                self.blueprints[5] = true;
                Outcome::ChestClaimed
            }
            Interaction::Checkpoint => {
                self.checkpoint = true;
                Outcome::CheckpointSet
            }
            Interaction::Board => Outcome::Briefing,
            _ => return Err(ActionRejected::Unavailable),
        };
        Ok(outcome)
    }
}
#[expect(
    clippy::too_many_arguments,
    reason = "Mission updates share the gameplay input gates."
)]
pub fn update_progress(
    time: Res<Time>,
    mut mission: ResMut<Mission>,
    mut player: ResMut<Player>,
    view: Res<ViewState>,
    capture: Res<crate::input::PointerCapture>,
    shooting_range: Res<ShootingRange>,
    nav: Res<Navigation>,
    mut ui: ResMut<UiState>,
    gate: crate::input::policy::InputGate,
) {
    if !gate.allows(
        crate::input::policy::GameplayAction::Collect,
        &view,
        &capture,
    ) {
        return;
    }
    if !mission.finished {
        mission.elapsed += time.delta_secs().min(0.05);
    }
    if shooting_range
        .completion()
        .is_some_and(|completed| mission.complete_range(completed))
    {
        ui.show("Alle drei Ziele getroffen! Die Truhe hinten rechts ist entriegelt.".into());
    }
    for (i, p) in DATA.blueprints().into_iter().map(Vec3::from).enumerate() {
        if i == 5 || (i == 4 && !mission.rack_open) {
            continue;
        }
        if !mission.blueprints[i] && near(&player, p, &nav) {
            mission.blueprints[i] = true;
            ui.show("Bauplan eingesammelt.".into());
        }
    }
    for (i, p) in [
        map::point(34., 5., 0.),
        map::point(21., 47., 3.),
        map::point(6.5, 41., 0.),
    ]
    .into_iter()
    .enumerate()
    {
        if near(&player, p, &nav) {
            mission.secrets[i] = true;
        }
    }
    if player.height() < -2. {
        let p = if mission.checkpoint {
            map::CHECKPOINT
        } else {
            map::START
        };
        player.respawn(p.xz(), p.y);
    }
}
