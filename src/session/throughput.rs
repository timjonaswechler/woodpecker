//! Opt-in Warp batching with a separate, frame-lived stream of render asset events.
use bevy::{
    asset::{Asset, AssetEvent},
    ecs::message::MessageCursor,
    prelude::*,
    render::{RenderApp, storage::ShaderBuffer},
    shader::Shader,
};

#[cfg(test)]
#[path = "throughput/test_asset_messages.rs"]
mod test_asset_messages;

/// Opt into larger maximum-speed Warp batches after installing [`super::Plugin`].
/// Mesh, image, shader and shader-buffer events are retained until extraction.
/// Register every additional renderer-consumed asset type with [`Self::with_asset`]
/// (for example `StandardMaterial` and custom materials). Application message
/// readers and message aging retain their normal per-tick behavior.
pub struct ThroughputPlugin {
    assets: Vec<fn(&mut World)>,
}

impl Default for ThroughputPlugin {
    fn default() -> Self {
        Self {
            assets: vec![
                register::<Mesh>,
                register::<Image>,
                register::<Shader>,
                register::<ShaderBuffer>,
            ],
        }
    }
}

impl ThroughputPlugin {
    /// Retain this asset type's events for the renderer without changing the
    /// application's event stream. Install its asset/render plugin first.
    pub fn with_asset<A: Asset>(mut self) -> Self {
        self.assets.push(register::<A>);
        self
    }
}

impl bevy::app::Plugin for ThroughputPlugin {
    fn build(&self, app: &mut App) {
        assert!(
            app.is_plugin_added::<super::Plugin>(),
            "install session::Plugin before ThroughputPlugin"
        );
        assert!(
            app.get_sub_app(RenderApp).is_some(),
            "ThroughputPlugin requires an application-installed renderer"
        );
        app.init_resource::<Buffers>();
        for register in &self.assets {
            register(app.world_mut());
        }
    }
}

#[derive(Clone, Copy)]
struct Buffer {
    capture: fn(&mut World),
    swap: fn(&mut World),
    finish: fn(&mut World),
}

#[derive(Resource, Default)]
struct Buffers(Vec<Buffer>);

#[derive(Resource)]
struct AssetMessages<A: Asset> {
    cursor: MessageCursor<AssetEvent<A>>,
    render: Messages<AssetEvent<A>>,
}

impl<A: Asset> Default for AssetMessages<A> {
    fn default() -> Self {
        Self {
            cursor: default(),
            render: default(),
        }
    }
}

fn register<A: Asset>(world: &mut World) {
    assert!(
        world.contains_resource::<Messages<AssetEvent<A>>>(),
        "install the asset plugin for {} before ThroughputPlugin",
        std::any::type_name::<A>()
    );
    if world.contains_resource::<AssetMessages<A>>() {
        return;
    }
    world.init_resource::<AssetMessages<A>>();
    world.resource_mut::<Buffers>().0.push(Buffer {
        capture: capture_asset::<A>,
        swap: swap_asset::<A>,
        finish: finish_asset::<A>,
    });
}

fn capture_asset<A: Asset>(world: &mut World) {
    world.resource_scope(|world, mut buffer: Mut<AssetMessages<A>>| {
        let events: Vec<_> = buffer
            .cursor
            .read(world.resource::<Messages<AssetEvent<A>>>())
            .copied()
            .collect();
        buffer.render.write_batch(events);
    });
}

fn swap_asset<A: Asset>(world: &mut World) {
    world.resource_scope(|world, mut buffer: Mut<AssetMessages<A>>| {
        std::mem::swap(
            world
                .resource_mut::<Messages<AssetEvent<A>>>()
                .bypass_change_detection(),
            &mut buffer.render,
        );
    });
}

fn finish_asset<A: Asset>(world: &mut World) {
    swap_asset::<A>(world);
    world.resource_mut::<AssetMessages<A>>().render.update();
}

pub(super) fn enabled(world: &World) -> bool {
    world.contains_resource::<Buffers>()
}

pub(super) fn capture(world: &mut World) {
    if let Some(buffers) = world
        .get_resource::<Buffers>()
        .map(|buffers| buffers.0.clone())
    {
        for buffer in buffers {
            (buffer.capture)(world);
        }
    }
}

pub(super) fn begin_extract(world: &mut World) {
    if let Some(buffers) = world
        .get_resource::<Buffers>()
        .map(|buffers| buffers.0.clone())
    {
        for buffer in buffers {
            (buffer.capture)(world);
            (buffer.swap)(world);
        }
    }
}

pub(super) fn end_extract(world: &mut World) {
    if let Some(buffers) = world
        .get_resource::<Buffers>()
        .map(|buffers| buffers.0.clone())
    {
        for buffer in buffers {
            (buffer.finish)(world);
        }
    }
}
