//! Room tasks, inventory and optional discoveries. No rendering or movement rules here.
use crate::{
    blueprint::{self as map, DATA},
    camera::ViewState,
    level::LevelLoad,
    navigation::Navigation,
    player::Player,
    shooting::Practice,
    ui::UiState,
};
use bevy::{input_focus::InputFocus, prelude::*, text::EditableText, window::PrimaryWindow};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CellState {
    #[default]
    Locked,
    Available,
    Carried,
    Installed,
}
#[derive(Resource, Default)]
pub struct Expedition {
    pub parts: [bool; 2],
    pub valves: [bool; 2],
    pub scans: [bool; 3],
    pub cells: [CellState; 3],
    pub blueprints: [bool; 6],
    pub secrets: [bool; 3],
    pub rack_open: bool,
    pub practice_reward: bool,
    pub chest_claimed: bool,
    pub checkpoint: bool,
    pub elapsed: f32,
    pub finished: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interaction {
    Part(usize),
    Rack,
    Valve(usize),
    Scan(usize),
    Craft,
    Cell(usize),
    Install,
    Signal,
    Chest,
    Checkpoint,
    Board,
    None,
}
impl Interaction {
    pub fn prompt(self) -> &'static str {
        match self {
            Self::Part(_) => "E · Bauteil aufnehmen",
            Self::Rack => "E · Regal zur Seite fahren",
            Self::Valve(_) => "E · Bewässerungsventil öffnen",
            Self::Scan(_) => "E · Archivtafel scannen",
            Self::Craft => "E · Aus zwei Bauteilen eine Energiezelle bauen",
            Self::Cell(_) => "E · Energiezelle ins Inventar nehmen",
            Self::Install => "E · Gesammelte Energiezellen in den Generator einsetzen",
            Self::Signal => "E · Sendemast einschalten",
            Self::Chest => "E · Bauplan aus der Truhe nehmen",
            Self::Checkpoint => "E · Checkpoint aktivieren",
            Self::Board => "E · Auftrag lesen",
            Self::None => "",
        }
    }
}
fn near(player: &Player, point: Vec3, nav: &Navigation) -> bool {
    player.position.distance(point.xz()) < 1.75
        && (player.height - point.y).abs() < 0.35
        && nav.clear_segment(player.feet(), point)
}
impl Expedition {
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
            Interaction::Part(i) => self.parts.get(i) == Some(&false) && (i == 0 || self.rack_open),
            Interaction::Rack => !self.rack_open,
            Interaction::Valve(i) => self.valves.get(i) == Some(&false),
            Interaction::Scan(i) => self.scans.get(i) == Some(&false),
            Interaction::Craft => {
                self.parts.iter().all(|&v| v) && self.cells[0] == CellState::Locked
            }
            Interaction::Cell(i) => self.cells.get(i) == Some(&CellState::Available),
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
            Interaction::Part(i) => self.parts[i],
            Interaction::Rack => self.rack_open,
            Interaction::Valve(i) => self.valves[i],
            Interaction::Scan(i) => self.scans[i],
            Interaction::Craft => self.cells[0] != CellState::Locked,
            Interaction::Cell(i) => {
                matches!(self.cells[i], CellState::Carried | CellState::Installed)
            }
            Interaction::Install => self.installed_cells() == 3,
            Interaction::Signal => self.finished,
            Interaction::Chest => self.chest_claimed,
            Interaction::Checkpoint => self.checkpoint,
            Interaction::Board | Interaction::None => false,
        }
    }
    pub fn interaction(&self, player: &Player, nav: &Navigation, view: &ViewState) -> Interaction {
        crate::interactions::select(self, player, nav, view)
    }
    /// State transitions enforce prerequisites even outside the UI dispatcher.
    pub fn apply(&mut self, action: Interaction) -> String {
        if !self.available(action) {
            return "Hier gibt es gerade nichts zu tun.".into();
        }
        match action {
            Interaction::Part(i) if i < 2 && !self.parts[i] && (i == 0 || self.rack_open) => {
                self.parts[i] = true;
                "Bauteil verstaut. Mit beiden Bauteilen geht es zur Werkbank in B.".into()
            }
            Interaction::Rack if !self.rack_open => {
                self.rack_open = true;
                "Regal geöffnet. Das zweite Bauteil und der Lagerfund sind frei.".into()
            }
            Interaction::Valve(i) if i < 2 && !self.valves[i] => {
                self.valves[i] = true;
                if self.valves.iter().all(|&v| v) {
                    self.cells[1] = CellState::Available;
                    "Wasser läuft! Nimm die Zelle am Verteiler im Gewächshaus auf.".into()
                } else {
                    "Erstes Ventil offen. Folge dem Rohr zum zweiten Ventil.".into()
                }
            }
            Interaction::Scan(i) if i < 3 && !self.scans[i] => {
                self.scans[i] = true;
                if self.scans.iter().all(|&v| v) {
                    self.cells[2] = CellState::Available;
                    "Archiv vollständig. Auf dem Ausgabetisch liegt eine Energiezelle bereit.".into()
                } else {
                    "Tafel gescannt. Suche nach weiteren markierten Tafeln.".into()
                }
            }
            Interaction::Craft
                if self.parts.iter().all(|&v| v) && self.cells[0] == CellState::Locked =>
            {
                self.cells[0] = CellState::Available;
                "Zelle gebaut. Sie liegt links auf dem Montagetisch bereit.".into()
            }
            Interaction::Cell(i) if self.cells.get(i) == Some(&CellState::Available) => {
                self.cells[i] = CellState::Carried;
                "Energiezelle verstaut.".into()
            }
            Interaction::Install => {
                for cell in &mut self.cells {
                    if *cell == CellState::Carried {
                        *cell = CellState::Installed;
                    }
                }
                "Zellen eingesetzt. Das Signal steht auf dem Balkon in H.".into()
            }
            Interaction::Signal if self.installed_cells() == 3 => {
                self.finished = true;
                "Signal aktiv! Auftrag geschafft. Die übrigen Fundstücke bleiben erreichbar.".into()
            }
            Interaction::Chest if self.practice_reward => {
                self.chest_claimed = true;
                self.blueprints[5] = true;
                "Bauplan aus der Truhe genommen.".into()
            }
            Interaction::Checkpoint => {
                self.checkpoint = true;
                "Checkpoint gesetzt. Backspace bringt dich hierher zurück.".into()
            }
            Interaction::Board => "Drei Zellen starten das Signal: Bauteile in B und D, Ventile in E, Archivtafeln in F. Die Reihenfolge ist frei.".into(),
            _ => "Hier gibt es gerade nichts zu tun.".into(),
        }
    }
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
            return map::BENCH;
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
    pub fn objective(&self) -> &'static str {
        if self.finished {
            "Signal aktiv. Erkunde die Nebenräume und sammle die übrigen Münzen und Baupläne."
        } else if !self.parts[0] {
            "Finde Bauteile in Werkhalle B und Lagerhalle D. Du kannst auch zuerst E oder F erkunden."
        } else if !self.rack_open {
            "In D gibt ein Hebel das zweite Bauteil hinter dem Regal frei."
        } else if !self.parts[1] {
            "Nimm das zweite Bauteil hinter dem verschobenen Regal in D auf."
        } else if self.cells[0] == CellState::Locked {
            "Baue an der Werkbank in B aus beiden Bauteilen eine Energiezelle."
        } else if self.cells[0] == CellState::Available {
            "Nimm die gebaute Energiezelle an der Werkbank mit E auf."
        } else if !self.valves.iter().all(|&v| v) {
            "Öffne beide Ventile im Gewächshaus E. Die Rohre verbinden sie."
        } else if self.cells[1] == CellState::Available {
            "Im Gewächshaus liegt die Energiezelle am Verteiler bereit."
        } else if !self.scans.iter().all(|&v| v) {
            "Scanne die drei markierten Tafeln im Archiv F."
        } else if self.cells[2] == CellState::Available {
            "Nimm die Energiezelle vom Ausgabetisch im Archiv."
        } else if self.installed_cells() < 3 {
            "Setze deine Energiezellen in den Generator im Raum H ein."
        } else {
            "Nimm in H die Treppe zum Balkon und starte das Signal mit E."
        }
    }
}
#[expect(
    clippy::too_many_arguments,
    reason = "Mission updates share the gameplay input gates."
)]
pub fn tick_expedition(
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
    focus: Res<InputFocus>,
    inputs: Query<(), With<EditableText>>,
    load: Res<LevelLoad>,
    mut mission: ResMut<Expedition>,
    mut player: ResMut<Player>,
    view: Res<ViewState>,
    practice: Res<Practice>,
    nav: Res<Navigation>,
    mut ui: ResMut<UiState>,
) {
    if !load.is_ready()
        || ui.menu
        || !window.focused
        || focus.get().is_some_and(|e| inputs.contains(e))
        || (view.first_person && !view.captured)
    {
        return;
    }
    if !mission.finished {
        mission.elapsed += time.delta_secs().min(0.05);
    }
    if !mission.practice_reward && practice.targets_hit.iter().all(|&v| v) {
        mission.practice_reward = true;
        ui.status = "Alle drei Ziele getroffen! Die Truhe hinten rechts ist entriegelt.".into();
    }
    for (i, p) in DATA.blueprints.into_iter().map(Vec3::from).enumerate() {
        if i == 5 || (i == 4 && !mission.rack_open) {
            continue;
        }
        if !mission.blueprints[i] && near(&player, p, &nav) {
            mission.blueprints[i] = true;
            ui.status = "Bauplan eingesammelt.".into();
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
    if player.height < -2. {
        let p = if mission.checkpoint {
            map::CHECKPOINT
        } else {
            map::START
        };
        player.respawn(p.xz(), p.y);
    }
}
