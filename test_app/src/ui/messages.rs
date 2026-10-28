//! Human-readable feedback for structured gameplay outcomes.
use crate::level::mission::{ActionRejected, Outcome};
pub(crate) fn mission(result: Result<Outcome, ActionRejected>) -> String {
    match result {
        Ok(Outcome::PartCollected) => "Bauteil verstaut. Mit beiden Bauteilen geht es zur Werkbank in B.",
        Ok(Outcome::RackOpened) => "Regal geöffnet. Das zweite Bauteil und der Lagerfund sind frei.",
        Ok(Outcome::WaterStarted) => "Wasser läuft! Nimm die Zelle am Verteiler im Gewächshaus auf.",
        Ok(Outcome::ValveOpened) => "Erstes Ventil offen. Folge dem Rohr zum zweiten Ventil.",
        Ok(Outcome::ArchiveCompleted) => "Archiv vollständig. Auf dem Ausgabetisch liegt eine Energiezelle bereit.",
        Ok(Outcome::PageScanned) => "Tafel gescannt. Suche nach weiteren markierten Tafeln.",
        Ok(Outcome::CellProduced) => "Zelle gebaut. Sie liegt links auf dem Montagetisch bereit.",
        Ok(Outcome::CellCollected) => "Energiezelle verstaut.",
        Ok(Outcome::CellsInstalled) => "Zellen eingesetzt. Das Signal steht auf dem Balkon in H.",
        Ok(Outcome::SignalEnabled) => "Signal aktiv! Auftrag geschafft. Die übrigen Fundstücke bleiben erreichbar.",
        Ok(Outcome::ChestClaimed) => "Bauplan aus der Truhe genommen.",
        Ok(Outcome::CheckpointSet) => "Checkpoint gesetzt. Backspace bringt dich hierher zurück.",
        Ok(Outcome::Briefing) => "Drei Zellen starten das Signal: Bauteile in B und D, Ventile in E, Archivtafeln in F. Die Reihenfolge ist frei.",
        Err(ActionRejected::Unavailable) => "Hier gibt es gerade nichts zu tun.",
    }.into()
}

use crate::level::mission::{Interaction, Objective};
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
impl Objective {
    pub fn text(self) -> &'static str {
        match self {
            Self::ExploreRemainder => {
                "Signal aktiv. Erkunde die Nebenräume und sammle die übrigen Münzen und Baupläne."
            }
            Self::FindParts => {
                "Finde Bauteile in Werkhalle B und Lagerhalle D. Du kannst auch zuerst E oder F erkunden."
            }
            Self::OpenRack => "In D gibt ein Hebel das zweite Bauteil hinter dem Regal frei.",
            Self::CollectPart => "Nimm das zweite Bauteil hinter dem verschobenen Regal in D auf.",
            Self::CraftCell => "Baue an der Werkbank in B aus beiden Bauteilen eine Energiezelle.",
            Self::CollectCraftedCell => "Nimm die gebaute Energiezelle an der Werkbank mit E auf.",
            Self::OpenValves => "Öffne beide Ventile im Gewächshaus E. Die Rohre verbinden sie.",
            Self::CollectWaterCell => "Im Gewächshaus liegt die Energiezelle am Verteiler bereit.",
            Self::ScanArchive => "Scanne die drei markierten Tafeln im Archiv F.",
            Self::CollectArchiveCell => "Nimm die Energiezelle vom Ausgabetisch im Archiv.",
            Self::InstallCells => "Setze deine Energiezellen in den Generator im Raum H ein.",
            Self::StartSignal => "Nimm in H die Treppe zum Balkon und starte das Signal mit E.",
        }
    }
}

pub(crate) fn route(outcome: crate::player::RouteOutcome) -> String {
    use crate::player::RouteOutcome;
    match outcome {
        RouteOutcome::Started => "Unterwegs. Pfeiltasten übernehmen jederzeit die Steuerung.",
        RouteOutcome::Airborne => "Lande zuerst, bevor du eine Laufroute auswählst.",
        RouteOutcome::NoSurface => "Wähle einen Boden, eine Treppe oder eine Galerie als Laufziel.",
        RouteOutcome::Unreachable => {
            "Kein Laufweg dorthin. Treppen sind begehbar; Lücken überspringst du mit der Leertaste."
        }
    }
    .into()
}

pub(crate) fn rejected_hit(
    reason: crate::level::objects::shooting_range::HitRejected,
) -> Option<String> {
    use crate::level::objects::shooting_range::{HitRejected, MIN_TARGET_DISTANCE};
    match reason {
        HitRejected::TooClose(distance) => Some(format!(
            "Zu nah: {distance:.1} m. Geh auf mindestens {MIN_TARGET_DISTANCE:.0} m Abstand zurück."
        )),
        _ => None,
    }
}
pub(crate) fn hit(points: u32) -> String {
    format!(
        "{} +{points} Punkte · Triff jede der drei Scheiben.",
        if points == 50 {
            "Volltreffer!"
        } else {
            "Treffer!"
        }
    )
}
