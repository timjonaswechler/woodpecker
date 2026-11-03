use super::*;

#[derive(Asset, TypePath)]
struct CustomRenderAsset;

#[test]
fn render_asset_messages_survive_tick_aging_without_changing_application_readers() {
    let mut world = World::new();
    world.init_resource::<Buffers>();
    world.init_resource::<Messages<AssetEvent<CustomRenderAsset>>>();
    world.insert_resource(Time::<Virtual>::default());
    register::<CustomRenderAsset>(&mut world);
    register::<CustomRenderAsset>(&mut world);
    assert_eq!(world.resource::<Buffers>().0.len(), 1);
    let id = Handle::<CustomRenderAsset>::default().id();
    let mut application = MessageCursor::<AssetEvent<CustomRenderAsset>>::default();
    let mut renderer = MessageCursor::<AssetEvent<CustomRenderAsset>>::default();

    world.write_message(AssetEvent::Added { id });
    capture(&mut world);
    assert_eq!(
        application
            .read(world.resource::<Messages<AssetEvent<CustomRenderAsset>>>())
            .count(),
        1
    );
    // The real failure pattern: several First schedules age out an Added event
    // before the next rendered frame. The application stream must still age.
    for _ in 0..4 {
        world
            .resource_mut::<Messages<AssetEvent<CustomRenderAsset>>>()
            .update();
        capture(&mut world);
    }
    assert!(
        world
            .resource::<Messages<AssetEvent<CustomRenderAsset>>>()
            .is_empty()
    );
    world.write_message(AssetEvent::Modified { id });
    capture(&mut world);
    begin_extract(&mut world);
    let extracted: Vec<_> = renderer
        .read(world.resource::<Messages<AssetEvent<CustomRenderAsset>>>())
        .copied()
        .collect();
    assert_eq!(
        extracted,
        [AssetEvent::Added { id }, AssetEvent::Modified { id }]
    );
    end_extract(&mut world);
    let application_events: Vec<_> = application
        .read(world.resource::<Messages<AssetEvent<CustomRenderAsset>>>())
        .copied()
        .collect();
    assert_eq!(
        application_events,
        [AssetEvent::Modified { id }],
        "the renderer must not replay old events into the app"
    );

    world.write_message(AssetEvent::Unused { id });
    capture(&mut world);
    begin_extract(&mut world);
    let extracted: Vec<_> = renderer
        .read(world.resource::<Messages<AssetEvent<CustomRenderAsset>>>())
        .copied()
        .collect();
    assert_eq!(extracted, [AssetEvent::Unused { id }]);
    end_extract(&mut world);
    begin_extract(&mut world);
    assert_eq!(
        renderer
            .read(world.resource::<Messages<AssetEvent<CustomRenderAsset>>>())
            .count(),
        0,
        "idle extraction must not duplicate events"
    );
    end_extract(&mut world);
    assert_eq!(
        application
            .read(world.resource::<Messages<AssetEvent<CustomRenderAsset>>>())
            .count(),
        1
    );
    assert_eq!(
        world.resource::<Time<Virtual>>().elapsed(),
        std::time::Duration::ZERO
    );
}
