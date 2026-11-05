//! Logical window identity and image geometry, without a native window backend.
//! The application must disable Winit, system clipboard and pipelined rendering
//! before installing DefaultPlugins. Install this plugin before session::Plugin.
#[cfg(test)]
#[path = "headless/tests/test_viewport.rs"]
mod test_viewport;

use super::{OffscreenTarget, launch::Headless};
use bevy::{
    camera::{CameraUpdateSystems, ImageRenderTarget, RenderTarget},
    prelude::*,
    render::render_resource::{TextureFormat, TextureUsages},
    window::{PrimaryWindow, WindowRef, WindowResolution},
};

/// A logical input identity, not a native window/surface. The image is also the
/// capture target. Applications can use `target()` for explicitly owned cameras.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Viewport {
    width: u32,
    height: u32,
    scale_factor: f32,
    pub(super) window: Entity,
    image: Handle<Image>,
}

impl Viewport {
    pub fn configuration(&self) -> Headless {
        Headless {
            width: self.width,
            height: self.height,
            scale_factor: self.scale_factor,
        }
    }

    pub fn target(&self) -> RenderTarget {
        RenderTarget::Image(ImageRenderTarget {
            handle: self.image.clone(),
            scale_factor: self.scale_factor,
        })
    }
}

/// Install the requested viewport after the application's renderer/asset plugins.
/// Primary-window cameras are routed to the image before camera/UI preparation.
/// The Window entity is metadata for keyboard identity and existing game geometry;
/// no Winit backend or raw window handle may be installed for it.
pub struct Plugin(pub Headless);

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        self.0.validate().expect("invalid headless geometry");
        assert!(
            !app.world()
                .iter_entities()
                .any(|entity| entity.contains::<Window>()),
            "headless requires WindowPlugin.primary_window = None"
        );
        let mut image = Image::new_target_texture(
            self.0.width,
            self.0.height,
            TextureFormat::Rgba8UnormSrgb,
            None,
        );
        image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
        let image = app.world_mut().resource_mut::<Assets<Image>>().add(image);
        let window = app
            .world_mut()
            .spawn((
                Name::new("woodpecker-logical-viewport"),
                PrimaryWindow,
                Window {
                    resolution: WindowResolution::new(self.0.width, self.0.height)
                        .with_scale_factor_override(self.0.scale_factor),
                    focused: true,
                    ..default()
                },
            ))
            .id();
        app.insert_resource(OffscreenTarget(image.clone()))
            .insert_resource(Viewport {
                width: self.0.width,
                height: self.0.height,
                scale_factor: self.0.scale_factor,
                window,
                image,
            })
            .register_type::<Viewport>()
            .add_systems(PostStartup, route_cameras)
            .add_systems(PostUpdate, route_cameras.before(CameraUpdateSystems));
    }
}

fn route_cameras(viewport: Res<Viewport>, mut cameras: Query<&mut RenderTarget, With<Camera>>) {
    for mut target in &mut cameras {
        if matches!(*target, RenderTarget::Window(WindowRef::Primary)) {
            *target = viewport.target();
        }
    }
}
