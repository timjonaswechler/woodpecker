//! Dependencies shared by the gameplay modules.
use bevy::prelude::*;
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Phase {
    Input,
    Commands,
    PrepareWorld,
    SampleInput,
    Movement,
    View,
    WorldActions,
    Fire,
    ResolveHits,
    Progress,
    Present,
}
pub(super) fn install(app: &mut App) {
    app.configure_sets(
        Update,
        (
            Phase::Input,
            Phase::Commands,
            Phase::PrepareWorld,
            Phase::SampleInput,
            Phase::Movement,
            Phase::View,
            Phase::WorldActions,
            Phase::Fire,
            Phase::ResolveHits,
            Phase::Progress,
            Phase::Present,
        )
            .chain(),
    );
}
