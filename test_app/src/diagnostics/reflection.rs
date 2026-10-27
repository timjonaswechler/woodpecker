//! Opt-in Inspect data and an explicit F10 lifecycle; no automatic ticks or render objects.
use bevy::{
    asset::{UntypedHandle, uuid::Uuid},
    prelude::*,
};
use std::collections::{HashMap, HashSet};

// All fixture types opt out of Bevy's global auto-registration. Ordinary runs
// and the F11/F12-only diagnostics must not acquire these extra registrations.
#[derive(Reflect)]
#[reflect(no_auto_register)]
enum Choice {
    Idle,
    Count(u32),
    Named { enabled: bool },
}

#[derive(Resource, Reflect)]
#[reflect(Resource, no_auto_register)]
struct Values {
    map: HashMap<String, u32>,
    integer_keys: HashMap<i32, String>,
    set: HashSet<String>,
    variants: Vec<Choice>,
    path_handle: Handle<Image>,
    ephemeral_handle: Handle<Image>,
    untyped_handle: UntypedHandle,
    other_ephemeral_handle: Handle<Image>,
    other_untyped_handle: UntypedHandle,
    uuid_handle: Handle<Image>,
}

#[derive(Resource, Reflect)]
#[reflect(Resource, no_auto_register)]
struct NonFinite {
    value: f64,
}

#[derive(Resource, Reflect)]
#[reflect(Resource, no_auto_register)]
struct UnsupportedMapKey {
    value: HashMap<Vec<i32>, u32>,
}

#[derive(Clone, Resource, Reflect)]
#[reflect(Resource, opaque, no_auto_register)]
struct OpaqueValue;

#[derive(Clone, Resource, Reflect, serde::Serialize)]
#[reflect(Resource, opaque, Serialize, no_auto_register)]
struct OpaqueNull;

#[derive(Resource, Reflect)]
#[reflect(no_auto_register)]
struct NotReflectableResource {
    value: u32,
}

#[derive(Resource, Reflect)]
#[reflect(Resource, no_auto_register)]
struct AbsentResource;

#[derive(Component, Reflect)]
#[reflect(Component, no_auto_register)]
struct Marker {
    label: String,
}

#[derive(Component)]
struct Unregistered;

#[derive(Component, Reflect)]
#[reflect(no_auto_register)]
struct NotReflectableComponent {
    value: u32,
}

#[derive(Component, Reflect)]
#[reflect(Component, no_auto_register)]
struct AbsentComponent;

#[derive(Component, Reflect)]
#[reflect(Component, no_auto_register)]
struct HierarchyNode {
    label: String,
}

#[derive(Resource)]
struct HierarchyState(Entity);

pub(super) fn install(app: &mut App) {
    app.register_type::<Choice>()
        .register_type::<Values>()
        .register_type::<NonFinite>()
        .register_type::<UnsupportedMapKey>()
        .register_type::<OpaqueValue>()
        .register_type::<OpaqueNull>()
        .register_type::<NotReflectableResource>()
        .register_type::<AbsentResource>()
        .register_type::<Marker>()
        .register_type::<NotReflectableComponent>()
        .register_type::<AbsentComponent>()
        .register_type::<HierarchyNode>();

    // Reuse a real shipped image for a path handle. Reserving the ephemeral
    // handle adds no image to Assets and needs no GPU preparation.
    let path_handle = app
        .world()
        .resource::<AssetServer>()
        .load("textures/prototypebits_texture.png");
    let ephemeral_handle = app.world().resource::<Assets<Image>>().reserve_handle();
    let other_ephemeral_handle = app.world().resource::<Assets<Image>>().reserve_handle();
    // Independent oracle: the allocator's native ID bits, not Inspect's serializer.
    // The file is complete before Ready; no stable index across sessions is assumed.
    let bits = |handle: &Handle<Image>| match handle.id() {
        bevy::asset::AssetId::Index { index, .. } => index.to_bits(),
        _ => unreachable!("reserve_handle must produce an indexed handle"),
    };
    let root = std::path::PathBuf::from(std::env::var_os("WOODPECKER_ARTIFACT_DIR").unwrap());
    std::fs::write(
        root.join("reflection-handle-bits.json"),
        serde_json::json!({
            "ephemeral_handle": bits(&ephemeral_handle),
            "other_ephemeral_handle": bits(&other_ephemeral_handle),
        })
        .to_string(),
    )
    .unwrap();
    app.insert_resource(Values {
        map: HashMap::from([("beta".into(), 2), ("alpha".into(), 1)]),
        integer_keys: HashMap::from([(-2, "negative".into()), (7, "positive".into())]),
        set: HashSet::from(["z".into(), "a".into(), "m".into()]),
        variants: vec![
            Choice::Idle,
            Choice::Count(7),
            Choice::Named { enabled: true },
        ],
        path_handle,
        untyped_handle: ephemeral_handle.clone().untyped(),
        ephemeral_handle,
        other_untyped_handle: other_ephemeral_handle.clone().untyped(),
        other_ephemeral_handle,
        uuid_handle: Handle::Uuid(
            Uuid::from_u128(0x12345678_1234_5678_90ab_cdef01234567),
            default(),
        ),
    })
    .insert_resource(NonFinite { value: f64::NAN })
    .insert_resource(UnsupportedMapKey {
        value: HashMap::from([(vec![1, 2], 3)]),
    })
    .insert_resource(OpaqueValue)
    .insert_resource(OpaqueNull)
    .insert_resource(NotReflectableResource { value: 23 });

    // Absent* are registered but never inserted; Unregistered is inserted but
    // never registered. Find the entity by its name or the reflected Marker.
    app.world_mut().spawn((
        Name::new("Woodpecker reflection fixture"),
        Marker {
            label: "reflection".into(),
        },
        Unregistered,
        NotReflectableComponent { value: 42 },
    ));
    let root = spawn_hierarchy(app.world_mut());
    app.insert_resource(HierarchyState(root)).add_systems(
        Update,
        replace_hierarchy.in_set(crate::app::schedule::Phase::WorldActions),
    );
}

fn spawn_hierarchy(world: &mut World) -> Entity {
    let root = world
        .spawn((
            Name::new("Inspect root"),
            HierarchyNode {
                label: "root".into(),
            },
        ))
        .id();
    let first = world.spawn_empty().id();
    let second = world.spawn_empty().id();
    let (child_a, child_z) = if first.index_u32() < second.index_u32() {
        (first, second)
    } else {
        (second, first)
    };
    world.entity_mut(child_a).insert((
        Name::new("Inspect child A"),
        HierarchyNode {
            label: "child-a".into(),
        },
    ));
    world.entity_mut(child_z).insert((
        Name::new("Inspect child Z"),
        HierarchyNode {
            label: "child-z".into(),
        },
    ));
    let leaf = world
        .spawn(HierarchyNode {
            label: "leaf".into(),
        })
        .id();
    // Descending index order deliberately, regardless of allocator reuse order.
    world.entity_mut(root).add_children(&[child_z, child_a]);
    world.entity_mut(child_z).add_child(leaf);
    root
}

fn replace_hierarchy(world: &mut World) {
    if !world
        .resource::<ButtonInput<KeyCode>>()
        .just_pressed(KeyCode::F10)
    {
        return;
    }
    let root = world.resource::<HierarchyState>().0;
    world.entity_mut(root).despawn();
    let replacement = spawn_hierarchy(world);
    world.resource_mut::<HierarchyState>().0 = replacement;
}
