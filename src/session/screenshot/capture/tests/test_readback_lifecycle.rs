use super::*;
use bevy::{
    asset::RenderAssetUsages,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
#[derive(Resource, Default)]
struct Ticks(u64);

#[test]
fn failed_or_expired_readbacks_never_write_or_complete_a_new_request() {
    let path = crate::test_support::artifacts("readback");
    let root = Arc::new(Dir::open_ambient_dir(&path, cap_std::ambient_authority()).unwrap());
    let mut world = World::new();
    world.init_resource::<Ticks>();
    let mut update = Schedule::new(Update);
    update.add_systems(|mut ticks: ResMut<Ticks>| ticks.0 += 1);
    world.add_schedule(update);
    let (images, readbacks) = mpsc::channel();
    world.insert_resource(CapturedScreenshots(Arc::new(Mutex::new(readbacks))));
    let (sender, receiver) = mpsc::channel();
    world.insert_resource(Service {
        root: root.clone(),
        queue: VecDeque::new(),
        active: None,
        sender,
        receiver: Mutex::new(receiver),
    });
    for (id, surface, expired, code) in [
        (10, Some(false), false, "screenshot_window_unavailable"),
        (11, None, false, "screenshot_failed"),
        (12, Some(true), true, "screenshot_failed"),
    ] {
        let filename = format!("failed-{id}.png");
        std::fs::write(path.join(&filename), b"owned old bytes").unwrap();
        let entity = world.spawn_empty().id();
        let flag = Arc::new(OnceLock::new());
        if let Some(value) = surface {
            flag.set(value).unwrap();
        }
        world.resource_mut::<Service>().active = Some(Active {
            job: Job {
                id,
                path: filename.clone(),
                window: Entity::PLACEHOLDER,
            },
            entity,
            since: if expired {
                Instant::now() - Duration::from_secs(31)
            } else {
                Instant::now()
            },
            writing: false,
            surface: flag,
        });
        // A readback is already queued: the deadline/surface check must win.
        images.send((entity, Image::default())).unwrap();
        let responses = poll(&mut world);
        assert_eq!(responses.len(), 1);
        match &responses[0] {
            Message::Rejected {
                request_id,
                command,
                error,
            } => {
                assert_eq!(*request_id, id);
                assert_eq!(command, "screenshot.capture");
                assert_eq!(error.code, code);
            }
            other => panic!("{other:?}"),
        }
        assert!(!world.resource::<Service>().pending(id));
        assert_eq!(
            std::fs::read(path.join(&filename)).unwrap(),
            b"owned old bytes"
        );
        // The old entity is gone; its delayed second delivery must not settle B.
        let b = world.spawn_empty().id();
        let flag = Arc::new(OnceLock::new());
        flag.set(true).unwrap();
        let b_id = id + 100;
        let b_path = format!("fresh-{b_id}.png");
        world.resource_mut::<Service>().active = Some(Active {
            job: Job {
                id: b_id,
                path: b_path.clone(),
                window: Entity::PLACEHOLDER,
            },
            entity: b,
            since: Instant::now(),
            writing: false,
            surface: flag,
        });
        images.send((entity, Image::default())).unwrap();
        assert!(poll(&mut world).is_empty());
        assert!(world.resource::<Service>().pending(b_id));
        assert!(!path.join(&b_path).exists());
        // Actual CPU PNG encoding/writing of B is the positive control, not a fake completion.
        let image = Image::new_fill(
            Extent3d {
                width: 3,
                height: 2,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[17, 34, 51, 255],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::MAIN_WORLD,
        );
        images.send((b, image)).unwrap();
        let mut result = poll(&mut world);
        let end = Instant::now() + Duration::from_secs(10);
        while result.is_empty() {
            assert!(Instant::now() < end);
            std::thread::sleep(Duration::from_millis(2));
            result = poll(&mut world);
        }
        assert_eq!(result.len(), 1);
        let value = serde_json::to_value(&result[0]).unwrap();
        assert_eq!(
            value,
            serde_json::json!({"status":"completed","request_id":b_id,"command":"screenshot.capture",
            "output":{"path":b_path,"width":3,"height":2,"overwritten":false}})
        );
        let image = image::load_from_memory_with_format(
            &std::fs::read(path.join(&b_path)).unwrap(),
            image::ImageFormat::Png,
        )
        .unwrap()
        .to_rgb8();
        assert_eq!(image.dimensions(), (3, 2));
        assert_eq!(image.get_pixel(0, 0).0, [17, 34, 51]);
        assert!(poll(&mut world).is_empty());
        assert!(!world.resource::<Service>().pending(b_id));
        assert_eq!(
            std::fs::read(path.join(&filename)).unwrap(),
            b"owned old bytes"
        );
    }
    assert_eq!(
        world.resource::<Ticks>().0,
        0,
        "Capture poll must not run Update"
    );
}
