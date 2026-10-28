use super::protocol::{self, Diagnostic, Message};
use crate::command::{
    Command,
    tick::{self, warp},
};
use bevy::{
    app::{AppExit, MainScheduleOrder},
    ecs::schedule::{InternedScheduleLabel, ScheduleLabel},
    prelude::*,
};
use std::{
    io::{BufRead, Write},
    sync::{Mutex, mpsc},
    time::Instant,
};

/// Experimental v3 bridge. Install after the application's simulation plugins.
/// It preserves the configured schedule order; a warp or configured technical startup runs that order.
/// Application time is preserved unless the session selects a fixed tick clock.
/// Native mouse, keyboard, touch and IME input is discarded in controlled sessions.
/// Enable the `ui` feature for applications using Bevy UI's `Interaction` components.
/// Enable `screenshot` for PNG capture with an application-installed renderer.
/// Renderer extraction and updates wait for the first completed counted tick; startup
/// alone does not prepare camera/light data. Automatic startup is opt-in and counts its application ticks.
#[derive(Default)]
pub struct Plugin;

#[derive(Resource)]
struct Bridge {
    input: Mutex<mpsc::Receiver<String>>,
    output: mpsc::Sender<(Message, Option<mpsc::Sender<()>>)>,
    schedules: Vec<InternedScheduleLabel>,
    pace: warp::Pace,
    clock: tick::Clock,
    active: Option<Active>,
    ticks: u64,
    pointer: super::input::pointer::State,
    keyboard: super::input::keyboard::State,
    text: super::input::text::State,
    input_gate: super::input::Gate,
}

struct Active {
    id: u64,
    requested: u64,
    executed: u64,
    pace: warp::Pace,
    last_tick: Option<Instant>,
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct Control;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let root = std::env::var_os("WOODPECKER_ARTIFACT_DIR")
            .expect("session requires WOODPECKER_ARTIFACT_DIR");
        assert!(
            std::path::Path::new(&root).is_dir(),
            "invalid artifact directory"
        );
        crate::report::capture::install_hook();
        let pace = std::env::var("WOODPECKER_TICK_PACE")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        let clock: tick::Clock = std::env::var("WOODPECKER_TICK_CLOCK")
            .map(|value| serde_json::from_str(&value).expect("invalid tick clock"))
            .unwrap_or_default();
        assert!(clock.is_valid(), "invalid tick clock");
        let (input, rx) = mpsc::channel();
        std::thread::spawn(move || {
            for line in std::io::stdin().lock().lines() {
                match line {
                    Ok(line) => {
                        if input.send(line).is_err() {
                            break;
                        }
                    }
                    _ => break,
                }
            }
        });
        let (output, messages) = mpsc::channel::<(Message, Option<mpsc::Sender<()>>)>();
        std::thread::spawn(move || {
            let mut stdout = std::io::stdout().lock();
            for (message, flushed) in messages {
                if serde_json::to_writer(&mut stdout, &message).is_err()
                    || stdout.write_all(b"\n").is_err()
                    || stdout.flush().is_err()
                {
                    break;
                }
                if let Some(flushed) = flushed {
                    let _ = flushed.send(());
                }
            }
        });
        install(app, rx, output, tick::Config { pace, clock });
        let startup = std::env::var("WOODPECKER_STARTUP")
            .map(|value| serde_json::from_str(&value).expect("invalid startup configuration"))
            .unwrap_or_default();
        super::startup::install(app, startup);
    }

    fn finish(&self, app: &mut App) {
        let root = std::env::var_os("WOODPECKER_ARTIFACT_DIR")
            .expect("session requires WOODPECKER_ARTIFACT_DIR");
        super::screenshot::install(app, std::path::Path::new(&root));
        crate::report::capture::layer_status();
        // Startup schedules precede the first control iteration, so Ready is sent there.
        app.add_systems(PostStartup, ready);
    }
}

pub(super) fn install(
    app: &mut App,
    input: mpsc::Receiver<String>,
    output: mpsc::Sender<(Message, Option<mpsc::Sender<()>>)>,
    tick: tick::Config,
) {
    super::clock::validate(app, &tick.clock);
    super::input::install(app);
    let schedules = {
        let mut order = app.world_mut().resource_mut::<MainScheduleOrder>();
        std::mem::replace(&mut order.labels, vec![Control.intern()])
    };
    app.insert_resource(Bridge {
        input: Mutex::new(input),
        output,
        schedules,
        pace: tick.pace,
        clock: tick.clock,
        active: None,
        ticks: 0,
        pointer: Default::default(),
        keyboard: Default::default(),
        text: Default::default(),
        input_gate: Default::default(),
    })
    .add_systems(Control, run);
    render::install(app);
}

#[cfg(feature = "screenshot")]
pub(super) fn render_waiting(world: &World) -> bool {
    render::waiting(world)
}

fn ready(world: &World) {
    let bridge = world.resource::<Bridge>();
    let _ = bridge.output.send((
        Message::Ready {
            version: protocol::VERSION,
            capabilities: protocol::Capabilities {
                screenshot: super::screenshot::available(world),
                fixed_clock: bridge.clock.duration().is_some(),
                engine_startup: super::startup::enabled(world),
            },
        },
        None,
    ));
}

fn run(world: &mut World) {
    world.resource_scope(|world, mut bridge: Mut<Bridge>| {
        bridge.input_gate.capture(world);
        // Bound both command intake and tick execution so neither can starve the other.
        for _ in 0..64 {
            let line = bridge.input.lock().unwrap().try_recv();
            match line {
                Ok(line) => {
                    let decoded = protocol::decode(&line);
                    let response = match decoded {
                        Ok((id, command))
                            if bridge.active.as_ref().is_some_and(|a| a.id == id)
                                || super::screenshot::pending(world, id) =>
                        {
                            Some(Message::ProtocolError {
                                request_id: Some(id),
                                error: Diagnostic::new("duplicate_request_id", command.name()),
                            })
                        }
                        Ok((id, command)) => dispatch(world, &mut bridge, id, command),
                        Err(message) => Some(message),
                    };
                    if let Some(response) = response {
                        let _ = bridge.output.send((response, None));
                    }
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    world.write_message(AppExit::Success);
                    return;
                }
            }
        }
        for response in super::screenshot::poll(world) {
            let _ = bridge.output.send((response, None));
        }
        if let Some(message) = super::startup::poll(world, bridge.ticks) {
            let _ = bridge.output.send((message, None));
        }
        if super::startup::needs_tick(world) {
            run_tick(world, &mut bridge);
            if let Some(message) = super::startup::after_tick(world, bridge.ticks) {
                let _ = bridge.output.send((message, None));
            }
        }
        let budget = Instant::now();
        for _ in 0..64 {
            let Some(active) = bridge.active.as_ref() else {
                break;
            };
            if let warp::Pace::TicksPerSecond { target } = active.pace
                && active
                    .last_tick
                    .is_some_and(|t| t.elapsed().as_secs_f64() < 1.0 / target as f64)
            {
                break;
            }
            run_tick(world, &mut bridge);
            let active = bridge.active.as_mut().unwrap();
            active.executed += 1;
            active.last_tick = Some(Instant::now());
            if active.executed == active.requested {
                finish(&mut bridge, warp::Outcome::Completed);
                break;
            }
            if budget.elapsed().as_millis() >= 2 {
                break;
            }
        }
    });
    // Rendering continues between warps. Its bounded clock channel must be drained
    // even when the simulation's time_system does not run. Never update Time here.
    if let Some(receiver) = world.get_resource::<bevy::time::TimeReceiver>() {
        for _ in receiver.0.try_iter() {}
    }
}

fn run_tick(world: &mut World, bridge: &mut Bridge) {
    bridge.input_gate.capture(world);
    bridge.input_gate.swap_window_events(world);
    bridge.pointer.flush(world);
    bridge.keyboard.flush(world);
    bridge.text.flush(world);
    super::clock::before_tick(world, &bridge.clock);
    for label in &bridge.schedules {
        let _ = world.try_run_schedule(*label);
    }
    bridge.input_gate.swap_window_events(world);
    bridge.ticks = bridge.ticks.checked_add(1).expect("tick counter exhausted");
    if bridge.ticks == 1 {
        render::start(world);
    }
}

fn finish(bridge: &mut Bridge, outcome: warp::Outcome) {
    if let Some(active) = bridge.active.take() {
        let _ = bridge.output.send((
            protocol::completed(
                active.id,
                "tick.warp.start",
                warp::Completion {
                    requested_ticks: active.requested,
                    executed_ticks: active.executed,
                    outcome,
                },
            ),
            None,
        ));
    }
}

fn dispatch(world: &mut World, bridge: &mut Bridge, id: u64, command: Command) -> Option<Message> {
    let name = command.name();
    let reject = |code, message: &str| {
        Some(Message::Rejected {
            request_id: id,
            command: name.into(),
            error: Diagnostic::new(code, message),
        })
    };
    let input_response = |result| {
        Some(match result {
            Ok(()) => protocol::completed(id, name, ()),
            Err(error) => Message::Rejected {
                request_id: id,
                command: name.into(),
                error,
            },
        })
    };
    if super::startup::pending(world)
        && !matches!(
            command,
            Command::Shutdown(_) | Command::Stop(_) | Command::Inspect(_)
        )
    {
        return reject(
            "startup_in_progress",
            "wait for EngineReady before controlling the simulation",
        );
    }
    let value = match command {
        Command::RecordingStart(_)
        | Command::RecordingStop(_)
        | Command::ReplayStart(_)
        | Command::ReplayStop(_) => {
            return reject(
                "unknown_command",
                "recording and replay are owned by the session coordinator",
            );
        }
        Command::Screenshot(capture) => {
            return match super::screenshot::start(world, id, capture.path) {
                Ok(()) => None,
                Err(error) => Some(Message::Rejected {
                    request_id: id,
                    command: name.into(),
                    error,
                }),
            };
        }
        Command::TextInput(input) => {
            return input_response(bridge.text.input(world, input));
        }
        Command::KeyboardPress(input) => {
            return input_response(bridge.keyboard.key(world, &input.key, true));
        }
        Command::KeyboardRelease(input) => {
            return input_response(bridge.keyboard.key(world, &input.key, false));
        }
        Command::PointerMoveTo(input) => {
            return input_response(bridge.pointer.move_to(world, input.position));
        }
        Command::PointerMoveBy(input) => {
            return input_response(bridge.pointer.move_by(world, input.delta));
        }
        Command::PointerPress(input) => {
            return input_response(bridge.pointer.button(world, &input.button, true));
        }
        Command::PointerRelease(input) => {
            return input_response(bridge.pointer.button(world, &input.button, false));
        }
        Command::PointerScroll(input) => {
            return input_response(bridge.pointer.scroll(world, input.delta));
        }
        Command::Start(start) => {
            if start.ticks == 0 {
                return reject("invalid_tick_count", "ticks must be positive");
            }
            let pace = start.pace.unwrap_or_else(|| bridge.pace.clone());
            if !pace.is_valid() {
                return reject("invalid_pace", "pace must be finite and positive");
            }
            if bridge.active.is_some() {
                return reject("warp_already_running", "warp is active");
            }
            bridge.active = Some(Active {
                id,
                requested: start.ticks,
                executed: 0,
                pace,
                last_tick: None,
            });
            return None;
        }
        Command::SetPace(change) => {
            if !change.pace.is_valid() {
                return reject("invalid_pace", "pace must be finite and positive");
            }
            bridge.pace = change.pace.clone();
            if let Some(active) = &mut bridge.active {
                active.pace = change.pace.clone();
                active.last_tick = None;
            }
            serde_json::json!({"pace": change.pace})
        }
        Command::Stop(_) => {
            let was_running = bridge.active.is_some();
            finish(bridge, warp::Outcome::Stopped);
            serde_json::json!({"was_running": was_running})
        }
        Command::Inspect(query) => match super::inspect::query(world, query) {
            Ok(output) => serde_json::to_value(output).unwrap(),
            Err(error) => {
                return Some(Message::Rejected {
                    request_id: id,
                    command: name.into(),
                    error,
                });
            }
        },
        Command::Shutdown(_) => {
            // All output uses one writer; shutdown waits for its flush acknowledgement.
            let response = protocol::completed(id, name, ());
            let (tx, rx) = mpsc::channel();
            let _ = bridge.output.send((response, Some(tx)));
            let _ = rx.recv();
            world.write_message(AppExit::Success);
            return None;
        }
    };
    Some(protocol::completed(id, name, value))
}

mod render;
