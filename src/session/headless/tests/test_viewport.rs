use super::*;

#[test]
fn requested_viewport_has_hidpi_geometry_image_identity_and_no_native_handle() {
    use bevy::{asset::AssetApp, camera::NormalizedRenderTarget, window::RawHandleWrapper};
    let configuration = Headless {
        width: 1440,
        height: 900,
        scale_factor: 2.0,
    };
    configuration.validate().unwrap();
    for invalid in [
        Headless {
            width: 0,
            ..configuration.clone()
        },
        Headless {
            height: 8193,
            ..configuration.clone()
        },
        Headless {
            scale_factor: f32::NAN,
            ..configuration.clone()
        },
        Headless {
            scale_factor: 0.0,
            ..configuration.clone()
        },
    ] {
        assert_eq!(invalid.validate().unwrap_err().code(), "invalid_config");
    }
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        WindowPlugin {
            primary_window: None,
            ..default()
        },
    ))
    .init_asset::<Image>()
    .add_plugins(Plugin(configuration.clone()));
    let camera = app
        .world_mut()
        .spawn((Camera::default(), RenderTarget::default()))
        .id();
    app.update(); // Explicit caller-owned update; no renderer or native backend is installed.
    let viewport = app.world().resource::<Viewport>();
    assert_eq!(viewport.configuration(), configuration);
    let window = app.world().get::<Window>(viewport.window).unwrap();
    assert_eq!((window.width(), window.height()), (720.0, 450.0));
    assert!(
        app.world()
            .get::<RawHandleWrapper>(viewport.window)
            .is_none()
    );
    let target = app
        .world()
        .get::<RenderTarget>(camera)
        .unwrap()
        .normalize(None)
        .unwrap();
    assert!(matches!(&target, NormalizedRenderTarget::Image(image) if image.scale_factor == 2.0));
    assert_eq!(Some(target), viewport.target().normalize(None));
    let image = app
        .world()
        .resource::<Assets<Image>>()
        .get(&viewport.image)
        .unwrap();
    assert_eq!((image.width(), image.height()), (1440, 900));
    assert!(
        image
            .texture_descriptor
            .usage
            .contains(TextureUsages::COPY_SRC)
    );
    let value = serde_json::to_value(super::super::protocol::Capabilities {
        offscreen: true,
        headless: Some(configuration.clone()),
        ..default()
    })
    .unwrap();
    assert_eq!(
        value["headless"],
        serde_json::json!({"width":1440,"height":900,"scale_factor":2.0})
    );
    // Multiple pre-existing primary windows are not an empty headless setup.
    let mut invalid = App::new();
    invalid.init_resource::<Assets<Image>>();
    invalid
        .world_mut()
        .spawn((Window::default(), PrimaryWindow));
    invalid
        .world_mut()
        .spawn((Window::default(), PrimaryWindow));
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            invalid.add_plugins(Plugin(configuration));
        }))
        .is_err()
    );
}
