use super::{Diagnostic, Message, destination};
use crate::{command::screenshot::Output, session::protocol};
use bevy::{
    ecs::schedule::ScheduleCleanupPolicy,
    prelude::*,
    render::{
        RenderApp,
        renderer::RenderDevice,
        view::screenshot::{
            Captured, CapturedScreenshots, Screenshot, ScreenshotCaptured, trigger_screenshots,
        },
    },
    window::RawHandleWrapper,
};
use cap_std::fs::Dir;
use std::{
    collections::VecDeque,
    io::Cursor,
    path::Path,
    sync::{Arc, Mutex, OnceLock, mpsc},
    time::{Duration, Instant},
};

mod surface;

const READBACK_TIMEOUT: Duration = Duration::from_secs(30);
const COMMAND: &str = "screenshot.capture";

struct Job {
    id: u64,
    path: String,
    window: Entity,
}

struct Active {
    job: Job,
    entity: Entity,
    since: Instant,
    writing: bool,
    surface: Arc<OnceLock<bool>>,
}

#[derive(Resource)]
pub(super) struct Service {
    root: Arc<Dir>,
    queue: VecDeque<Job>,
    active: Option<Active>,
    sender: mpsc::Sender<Result<Output, Diagnostic>>,
    receiver: Mutex<mpsc::Receiver<Result<Output, Diagnostic>>>,
}

impl Service {
    pub(super) fn pending(&self, id: u64) -> bool {
        self.active.as_ref().is_some_and(|a| a.job.id == id)
            || self.queue.iter().any(|job| job.id == id)
    }
}

pub(super) fn install(app: &mut App, root: &Path) {
    if app.get_sub_app(RenderApp).is_none()
        || !app.world().contains_resource::<RenderDevice>()
        || !app.world().contains_resource::<CapturedScreenshots>()
    {
        return;
    }
    let root = match Dir::open_ambient_dir(root, cap_std::ambient_authority()) {
        Ok(root) => root,
        Err(error) => {
            eprintln!("woodpecker screenshot unavailable: {error}");
            return;
        }
    };
    // Readback delivery must progress without running Update or any simulation system.
    app.world_mut().schedule_scope(Update, |world, schedule| {
        schedule
            .remove_systems_in_set(
                trigger_screenshots,
                world,
                ScheduleCleanupPolicy::RemoveSystemsOnly,
            )
            .expect("remove tick-bound screenshot delivery");
    });
    let (sender, receiver) = mpsc::channel();
    app.insert_resource(Service {
        root: Arc::new(root),
        queue: VecDeque::new(),
        active: None,
        sender,
        receiver: Mutex::new(receiver),
    });
    surface::install(app);
}

fn window(world: &World) -> Result<Entity, Diagnostic> {
    super::super::window::primary(world)
        .filter(|(id, window)| {
            world.get::<RawHandleWrapper>(*id).is_some()
                && window.physical_width() > 0
                && window.physical_height() > 0
        })
        .map(|(id, _)| id)
        .ok_or_else(|| {
            Diagnostic::new(
                "screenshot_window_unavailable",
                "expected one primary rendered window",
            )
        })
}

pub(super) fn start(world: &mut World, id: u64, path: String) -> Result<(), Diagnostic> {
    let service = world.get_resource::<Service>().ok_or_else(|| {
        Diagnostic::new(
            "screenshot_unavailable",
            "renderer and screenshot service are not installed",
        )
    })?;
    let window = window(world)?;
    destination::prepare(&service.root, &path)?;
    world
        .resource_mut::<Service>()
        .queue
        .push_back(Job { id, path, window });
    Ok(())
}

fn response(id: u64, result: Result<Output, Diagnostic>) -> Message {
    match result {
        Ok(output) => protocol::completed(id, COMMAND, output),
        Err(error) => Message::Rejected {
            request_id: id,
            command: COMMAND.into(),
            error,
        },
    }
}

fn encode(root: &Dir, path: String, image: Image) -> Result<Output, Diagnostic> {
    let image = image
        .try_into_dynamic()
        .map_err(destination::failed)?
        .to_rgb8();
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 {
        return Err(destination::failed("empty screenshot image"));
    }
    let mut bytes = Cursor::new(Vec::new());
    image
        .write_to(&mut bytes, image::ImageFormat::Png)
        .map_err(destination::failed)?;
    let overwritten = destination::write(root, &path, bytes.get_ref())?;
    Ok(Output {
        path,
        width,
        height,
        overwritten,
    })
}

pub(super) fn poll(world: &mut World) -> Vec<Message> {
    if !world.contains_resource::<Service>() {
        return Vec::new();
    }
    let images: Vec<_> = world
        .resource::<CapturedScreenshots>()
        .0
        .lock()
        .unwrap()
        .try_iter()
        .collect();
    world.resource_scope(|world, mut service: Mut<Service>| {
        let mut responses = Vec::new();
        if service
            .active
            .as_ref()
            .is_some_and(|active| !active.writing && active.surface.get() == Some(&false))
        {
            let active = service.active.take().unwrap();
            world.despawn(active.entity);
            responses.push(response(
                active.job.id,
                Err(Diagnostic::new(
                    "screenshot_window_unavailable",
                    "primary window has no render surface for this capture frame",
                )),
            ));
        }
        // The deadline gates readback consumption, not just an empty poll.
        // A queued image must not start encoding after the request expired.
        if service
            .active
            .as_ref()
            .is_some_and(|a| !a.writing && a.since.elapsed() >= READBACK_TIMEOUT)
        {
            let active = service.active.take().unwrap();
            world.despawn(active.entity);
            responses.push(response(
                active.job.id,
                Err(destination::failed("GPU readback timed out")),
            ));
        }
        for (entity, image) in images {
            if let Some(active) = service
                .active
                .as_mut()
                .filter(|a| a.entity == entity && !a.writing)
            {
                if active.surface.get() != Some(&true) {
                    responses.push(response(
                        active.job.id,
                        Err(Diagnostic::new(
                            "screenshot_failed",
                            "capture frame render surface was not verified",
                        )),
                    ));
                    world.despawn(entity);
                    service.active = None;
                    continue;
                }
                active.writing = true;
                let path = active.job.path.clone();
                let root = service.root.clone();
                let sender = service.sender.clone();
                world.despawn(entity);
                // Encoding and disk IO must not block Control, command intake, or rendering.
                std::thread::spawn(move || {
                    let _ = sender.send(encode(&root, path, image));
                });
            } else if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                entity_mut.insert(Captured);
                world.trigger(ScreenshotCaptured { entity, image });
            }
        }
        let completion = service.receiver.lock().unwrap().try_recv().ok();
        if let Some(result) = completion {
            let active = service.active.take().expect("one active writer");
            responses.push(response(active.job.id, result));
        }
        if service.active.is_none() {
            while let Some(job) = service.queue.pop_front() {
                if window(world).ok() != Some(job.window) {
                    responses.push(response(
                        job.id,
                        Err(Diagnostic::new(
                            "screenshot_window_unavailable",
                            "primary window changed before capture",
                        )),
                    ));
                    continue;
                }
                // Bevy drops concurrent screenshots for the same render target. Serialize them.
                let entity = world.spawn(Screenshot::window(job.window)).id();
                service.active = Some(Active {
                    job,
                    entity,
                    since: Instant::now(),
                    writing: false,
                    surface: Arc::default(),
                });
                break;
            }
        }
        responses
    })
}
