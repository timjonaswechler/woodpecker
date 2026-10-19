//! Exercise the exported level with the same navigation and body used by the game.
mod assets;
mod interactions;
mod mission;
mod movement;
mod startup;

use crate::{
    level,
    level::definition::{self as map, DATA, point},
    level::mission::{CellState, Interaction, Mission},
    level::navigation::{BODY_HEIGHT, Navigation},
    player::Player,
};
use bevy::prelude::*;
