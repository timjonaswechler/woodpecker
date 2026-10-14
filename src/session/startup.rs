//! Counted technical warmup. The application retains ownership of its loading logic.
mod render;

use super::protocol::{Diagnostic, Message};
use bevy::{
    asset::{DependencyLoadState, LoadState, RecursiveDependencyLoadState, UntypedAssetId},
    prelude::*,
    world_serialization::{DynamicWorldRoot, WorldAssetRoot, WorldInstance, WorldInstanceSpawner},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, time::Instant};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Config {
    /// Preserve explicit control from the first application tick.
    Manual {},
    /// Run ordinary application ticks until scoped technical readiness is observed.
    Engine {
        timeout_seconds: u32,
        max_ticks: u64,
    },
}

impl Default for Config {
    fn default() -> Self {
        Self::Manual {}
    }
}

impl Config {
    pub(crate) fn enabled(&self) -> bool {
        matches!(self, Self::Engine { .. })
    }
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Manual {} => true,
            Self::Engine {
                timeout_seconds,
                max_ticks,
            } => *timeout_seconds > 0 && *max_ticks > 0,
        }
    }
}

/// Read-only adapter for the application's existing startup asset list.
/// Return `None` until the list is known, or `Some(vec![])` for a procedural scene.
/// This observes handles; it neither replaces loading systems nor changes game states.
#[derive(Resource)]
pub struct Assets(pub fn(&World) -> Option<Vec<UntypedHandle>>);

#[derive(Resource)]
pub(super) struct State {
    config: Config,
    started: Option<Instant>,
    finished: bool,
    stable: Option<HashSet<UntypedAssetId>>,
    frame: Option<render::Ticket>,
    waiting: String,
}

pub(super) fn install(app: &mut App, config: Config) {
    if !config.enabled() {
        return;
    }
    assert!(config.valid(), "invalid startup configuration");
    render::install(app);
    app.insert_resource(State {
        config,
        started: None,
        finished: false,
        stable: None,
        frame: None,
        waiting: "first application tick".into(),
    });
}

pub(super) fn pending(world: &World) -> bool {
    world
        .get_resource::<State>()
        .is_some_and(|state| !state.finished)
}

pub(super) fn enabled(world: &World) -> bool {
    world.contains_resource::<State>()
}

/// Called between ticks. A completed frame freezes the exact main-world tick it extracted.
pub(super) fn poll(world: &mut World, ticks: u64) -> Option<Message> {
    if !pending(world) {
        return None;
    }
    world.resource_scope(|world, mut state: Mut<State>| {
        let started = *state.started.get_or_insert_with(Instant::now);
        let mut result = None;
        if !world.contains_resource::<Assets>() {
            result = Some(Err(Diagnostic::new(
                "startup_asset_scope_missing",
                "register startup::Assets with the application's startup asset handles",
            )));
        } else if !render::available(world) {
            result = Some(Err(Diagnostic::new(
                "startup_renderer_missing",
                "engine startup requires an application-installed renderer",
            )));
        } else if let Some(frame) = &state.frame {
            result = frame.result();
            state.waiting = frame.waiting();
        }
        if result.is_none() {
            let Config::Engine {
                timeout_seconds,
                max_ticks,
            } = state.config
            else {
                unreachable!()
            };
            if started.elapsed().as_secs() >= u64::from(timeout_seconds) {
                result = Some(Err(Diagnostic::new(
                    "startup_timeout",
                    format!("after {ticks} ticks; waiting for {}", state.waiting),
                )));
            } else if ticks >= max_ticks && state.frame.is_none() {
                result = Some(Err(Diagnostic::new(
                    "startup_tick_limit",
                    format!("after {ticks} ticks; waiting for {}", state.waiting),
                )));
            }
        }
        result.map(|result| {
            state.finished = true;
            state.frame = None;
            match result {
                Ok(()) => Message::EngineReady { ticks },
                Err(error) => Message::StartupFailed { ticks, error },
            }
        })
    })
}

pub(super) fn needs_tick(world: &World) -> bool {
    world
        .get_resource::<State>()
        .is_some_and(|state| !state.finished && state.frame.is_none())
}

pub(super) fn after_tick(world: &mut World, ticks: u64) -> Option<Message> {
    world.resource_scope(|world, mut state: Mut<State>| {
        match prerequisites(world) {
            Err(error) => {
                state.finished = true;
                Some(Message::StartupFailed { ticks, error })
            }
            Ok(Probe::Waiting(reason)) => {
                state.stable = None;
                state.waiting = reason;
                None
            }
            Ok(Probe::Ready(ids)) => {
                // One further ordinary update lets the game's own load transition and
                // deferred scene construction run after the first ready observation.
                if state.stable.as_ref() == Some(&ids) {
                    if let Some((window, _)) = super::window::primary(world) {
                        state.frame = Some(render::Ticket::new(world, window));
                        state.waiting = "prepared pipelines and a completed window frame".into();
                    } else {
                        state.waiting = "one primary window".into();
                    }
                }
                state.stable = Some(ids);
                None
            }
        }
    })
}

enum Probe {
    Waiting(String),
    Ready(HashSet<UntypedAssetId>),
}

fn prerequisites(world: &World) -> Result<Probe, Diagnostic> {
    let Some(handles) = (world.resource::<Assets>().0)(world) else {
        return Ok(Probe::Waiting("startup asset list".into()));
    };
    let mut ids: HashSet<_> = handles.iter().map(UntypedHandle::id).collect();
    let mut scenes_ready = true;
    let spawner = world.get_resource::<WorldInstanceSpawner>();
    for entity in world.iter_entities() {
        let root = if let Some(root) = entity.get::<WorldAssetRoot>() {
            Some(root.0.id().untyped())
        } else {
            entity
                .get::<DynamicWorldRoot>()
                .map(|root| root.0.id().untyped())
        };
        if let Some(id) = root {
            ids.insert(id);
            scenes_ready &= entity.get::<WorldInstance>().is_some_and(|instance| {
                spawner.is_some_and(|spawner| spawner.instance_is_ready(**instance))
            });
        }
    }
    if !ids.is_empty() {
        let server = world.get_resource::<AssetServer>().ok_or_else(|| {
            Diagnostic::new(
                "startup_asset_server_missing",
                "scoped assets require AssetServer",
            )
        })?;
        let mut loading = false;
        for id in &ids {
            match server.get_load_states(*id) {
                Some((LoadState::Failed(error), _, _))
                | Some((_, DependencyLoadState::Failed(error), _))
                | Some((_, _, RecursiveDependencyLoadState::Failed(error))) => {
                    return Err(Diagnostic::new(
                        "startup_asset_failed",
                        format!("{id:?}: {error}"),
                    ));
                }
                Some((
                    LoadState::Loaded,
                    DependencyLoadState::Loaded,
                    RecursiveDependencyLoadState::Loaded,
                )) => {}
                _ => loading = true,
            }
        }
        if loading {
            return Ok(Probe::Waiting("assets and recursive dependencies".into()));
        }
    }
    if !scenes_ready {
        return Ok(Probe::Waiting("scene instances".into()));
    }
    Ok(Probe::Ready(ids))
}

#[cfg(test)]
mod tests;
