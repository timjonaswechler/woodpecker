use super::*;

#[test]
fn shared_assets_are_rooted_at_the_game_package() {
    let assets = std::path::Path::new(test_app::ASSET_DIR);
    assert!(assets.is_absolute());
    assert!(assets.join("fonts/Barlow-Regular.ttf").is_file());
    assert!(assets.join("ui/stations/board.png").is_file());
}

fn app() -> App {
    app_with_capture(false)
}

fn app_with_capture(capture: bool) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin {
            file_path: test_app::ASSET_DIR.into(),
            ..default()
        },
    ))
    .init_asset::<Mesh>()
    .init_asset::<StandardMaterial>()
    .init_asset::<Image>()
    .init_asset::<Font>()
    .init_asset::<Shader>()
    .init_asset_loader::<bevy::shader::ShaderLoader>()
    .init_resource::<LevelAssets>()
    .init_resource::<Expedition>()
    .init_resource::<PreviewState>()
    .init_resource::<Zoo>()
    .init_resource::<Views>()
    .init_resource::<ButtonInput<KeyCode>>()
    .init_resource::<ButtonInput<MouseButton>>()
    .init_resource::<AccumulatedMouseMotion>()
    .init_resource::<AccumulatedMouseScroll>()
    .add_message::<AppExit>()
    .add_plugins(GroundPlugin);
    if capture {
        let image = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(Image::new_target_texture(
                1440,
                900,
                TextureFormat::Rgba8UnormSrgb,
                None,
            ));
        app.insert_resource(Capture {
            directory: PathBuf::new(),
            image,
            shot: 0,
            frames: 0,
            pending: false,
            loading_frames: 0,
        });
    }
    app.world_mut().spawn(Window {
        resolution: WindowResolution::new(1440, 900).with_scale_factor_override(1.),
        ..default()
    });
    app.world_mut().run_system_cached(setup).unwrap();
    app.world_mut()
        .run_system_cached(crate::ground::spawn)
        .unwrap();
    app
}

#[test]
fn zoo_uses_shared_station_models_and_preview_states_without_starting_the_game() {
    let mut app = app();
    let world = app.world_mut();
    let exhibits: Vec<_> = world
        .query::<(&Exhibit, &Name, &Transform)>()
        .iter(world)
        .collect();
    assert_eq!(exhibits.len(), MODELS.len());
    for (exhibit, name, transform) in exhibits {
        assert_eq!(name.as_str(), MODELS[exhibit.0].0.name);
        assert_eq!(transform.translation, position(exhibit.0));
    }
    assert_eq!(world.query::<&SamplePickup>().iter(world).count(), 5);
    assert_eq!(world.query::<&Camera3d>().iter(world).count(), 4);
    assert_eq!(world.query::<&IsDefaultUiCamera>().iter(world).count(), 1);
    let ground = world
        .query_filtered::<Entity, With<crate::ground::ShadowGround>>()
        .single(world)
        .unwrap();
    assert!(
        world
            .get::<MeshMaterial3d<crate::ground::ShadowCatcher>>(ground)
            .is_some()
    );
    assert!(
        world
            .get::<MeshMaterial3d<StandardMaterial>>(ground)
            .is_none()
    );
    let mut opaque = world.query::<(&Mesh3d, &MeshMaterial3d<StandardMaterial>)>();
    let meshes = world.resource::<Assets<Mesh>>();
    assert!(
        !opaque.iter(world).any(|(mesh, _)| {
            let Some(bevy::mesh::VertexAttributeValues::Float32x3(positions)) = meshes
                .get(&mesh.0)
                .and_then(|mesh| mesh.attribute(Mesh::ATTRIBUTE_POSITION))
            else {
                return false;
            };
            positions
                .iter()
                .any(|point| point[0].abs() > 50. || point[2].abs() > 50.)
        }),
        "the zoo must not contain an opaque ground underneath the shadow catcher"
    );

    world.run_system_cached(stations::update_stations).unwrap();
    world.run_system_cached(update_view).unwrap();
    assert!(
        world
            .query_filtered::<&Visibility, With<SamplePickup>>()
            .iter(world)
            .all(|visibility| *visibility == Visibility::Inherited)
    );

    *world.resource_mut::<PreviewState>() = PreviewState::Used;
    world.resource_mut::<Zoo>().selected = Some(1);
    world.run_system_cached(stations::update_stations).unwrap();
    world.run_system_cached(update_view).unwrap();
    for (exhibit, visibility) in world.query::<(&Exhibit, &Visibility)>().iter(world) {
        assert_eq!(
            *visibility,
            if exhibit.0 == 1 {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            }
        );
    }
    assert!(
        world
            .query_filtered::<&Visibility, With<SamplePickup>>()
            .iter(world)
            .all(|visibility| *visibility == Visibility::Hidden)
    );
}

fn active_views(world: &mut World) -> Vec<View> {
    let mut views: Vec<_> = world
        .query::<(&View, &Camera)>()
        .iter(world)
        .filter_map(|(view, camera)| camera.is_active.then_some(*view))
        .collect();
    views.sort_by_key(|view| view.index());
    views
}

fn key(world: &mut World, key: KeyCode) {
    world.resource_mut::<ButtonInput<KeyCode>>().press(key);
    world.run_system_cached(input).unwrap();
    world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
    world.run_system_cached(update_view).unwrap();
}

#[test]
fn detail_cameras_and_buttons_switch_between_four_and_single_views() {
    use bevy::ui_widgets::Activate;

    let mut app = app();
    let world = app.world_mut();
    world.run_system_cached(update_view).unwrap();
    assert_eq!(active_views(world), [View::Orbit]);

    world.resource_mut::<Zoo>().selected = Some(2);
    world.run_system_cached(update_view).unwrap();
    assert_eq!(active_views(world), View::ALL);
    let focus = position(2) + Vec3::Y * 0.9;
    for (view, transform) in world.query::<(&View, &Transform)>().iter(world) {
        let expected = world.resource::<Views>().transform(*view, focus, false);
        assert_eq!(*transform, expected);
    }
    let controls: Vec<_> = world
        .query::<(Entity, &ViewControl)>()
        .iter(world)
        .map(|(entity, control)| (entity, *control))
        .collect();
    for (entity, control) in controls {
        match control {
            ViewControl::Single(view) => {
                world.trigger(Activate { entity });
                world.run_system_cached(update_view).unwrap();
                assert_eq!(active_views(world), [view]);
                assert_eq!(
                    world
                        .query::<(&ViewControl, &Node)>()
                        .iter(world)
                        .filter(|(_, node)| node.display == Display::Flex)
                        .count(),
                    2
                );
                world.trigger(Activate { entity });
                world.run_system_cached(update_view).unwrap();
                assert_eq!(active_views(world), View::ALL);
            }
            ViewControl::All => {
                world.resource_mut::<Views>().toggle(View::Orbit);
                world.trigger(Activate { entity });
                world.run_system_cached(update_view).unwrap();
                assert_eq!(active_views(world), View::ALL);
            }
        }
    }
}

#[test]
fn shortcuts_open_all_views_and_keep_single_view_zoom_when_toggling() {
    let mut app = app();
    let world = app.world_mut();
    key(world, KeyCode::ArrowRight);
    assert_eq!(active_views(world), View::ALL);
    for (key_code, view) in [KeyCode::F1, KeyCode::F2, KeyCode::F3, KeyCode::F4]
        .into_iter()
        .zip(View::ALL)
    {
        key(world, key_code);
        assert_eq!(active_views(world), [view]);
        key(world, key_code);
        assert_eq!(active_views(world), View::ALL);
    }
    world.resource_mut::<Views>().zoom(Some(View::Orbit), 2.);
    let zoom = world.resource::<Views>().height(View::Orbit, 3.2);
    key(world, KeyCode::F4);
    key(world, KeyCode::F5);
    assert_eq!(active_views(world), View::ALL);
    assert_eq!(world.resource::<Views>().height(View::Orbit, 3.2), zoom);
    key(world, KeyCode::KeyO);
    assert_eq!(active_views(world), [View::Orbit]);
    key(world, KeyCode::ArrowRight);
    assert_eq!(active_views(world), View::ALL);
}

#[test]
fn native_pointer_rotates_only_orbit_and_scrolls_only_the_hovered_pane() {
    let mut app = app();
    let world = app.world_mut();
    world.resource_mut::<Zoo>().selected = Some(0);
    let frame = world
        .resource::<Views>()
        .frame(UVec2::new(1440, 900), 1., true);
    let front = frame.rect(View::Front).unwrap().center().as_vec2();
    let orbit = frame.rect(View::Orbit).unwrap().center().as_vec2();
    let yaw = world.resource::<Views>().yaw;
    world
        .query::<&mut Window>()
        .single_mut(world)
        .unwrap()
        .set_cursor_position(Some(front));
    world
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    world.resource_mut::<AccumulatedMouseMotion>().delta = Vec2::new(20., 10.);
    world.resource_mut::<AccumulatedMouseScroll>().delta.y = 2.;
    world.run_system_cached(input).unwrap();
    assert_eq!(world.resource::<Views>().yaw, yaw);
    assert!(world.resource::<Views>().height(View::Front, 3.2) < 3.2);
    assert_eq!(world.resource::<Views>().height(View::Orbit, 3.2), 3.2);

    world.resource_mut::<ButtonInput<MouseButton>>().reset_all();
    world.resource_mut::<AccumulatedMouseScroll>().delta = Vec2::ZERO;
    world.run_system_cached(input).unwrap();
    world
        .query::<&mut Window>()
        .single_mut(world)
        .unwrap()
        .set_cursor_position(Some(orbit));
    world
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    world.run_system_cached(input).unwrap();
    assert_ne!(world.resource::<Views>().yaw, yaw);
}

#[test]
fn resized_viewports_follow_physical_resolution_and_capture_stays_single_orbit() {
    let mut app = app();
    let world = app.world_mut();
    world.resource_mut::<Zoo>().selected = Some(0);
    world
        .query::<&mut Window>()
        .single_mut(world)
        .unwrap()
        .resolution = WindowResolution::new(2002, 1406).with_scale_factor_override(2.);
    world.run_system_cached(update_view).unwrap();
    let frame = world
        .resource::<Views>()
        .frame(UVec2::new(2002, 1406), 2., true);
    for (view, camera) in world.query::<(&View, &Camera)>().iter(world) {
        let viewport = camera.viewport.as_ref().unwrap();
        let rect = frame.rect(*view).unwrap();
        assert_eq!(viewport.physical_position, rect.min);
        assert_eq!(viewport.physical_size, rect.size());
    }

    let mut capture_app = app_with_capture(true);
    let world = capture_app.world_mut();
    world.resource_mut::<Zoo>().selected = Some(0);
    world.resource_mut::<Views>().toggle(View::Front);
    world.run_system_cached(update_view).unwrap();
    assert_eq!(active_views(world), [View::Orbit]);
    let image = world.resource::<Capture>().image.clone();
    for (camera, target) in world.query::<(&Camera, &RenderTarget)>().iter(world) {
        assert_eq!(target.as_image(), Some(&image));
        if camera.is_active
            && let Some(viewport) = &camera.viewport
        {
            assert_eq!(viewport.physical_size, UVec2::new(1440, 900));
        }
    }
    assert!(
        world
            .query::<(&ViewControl, &Node)>()
            .iter(world)
            .all(|(_, node)| node.display == Display::None)
    );
}

#[test]
fn active_camera_passes_preserve_previous_viewports() {
    // Attachment clears affect the entire target, even when a camera has a viewport.
    // Exercise the actual camera stack, including the full-window UI pass.
    for capture in [false, true] {
        let mut app = app_with_capture(capture);
        let world = app.world_mut();
        for selected in [Some(0), None] {
            world.resource_mut::<Zoo>().selected = selected;
            for layout in [
                camera::Layout::Four,
                camera::Layout::Single(View::Top),
                camera::Layout::Single(View::Front),
                camera::Layout::Single(View::Right),
                camera::Layout::Single(View::Orbit),
            ] {
                world.resource_mut::<Views>().layout = layout;
                world.run_system_cached(update_view).unwrap();
                let mut passes: Vec<_> = world
                    .query::<(&Camera, &Name)>()
                    .iter(world)
                    .filter(|(camera, _)| camera.is_active)
                    .collect();
                passes.sort_by_key(|(camera, _)| camera.order);
                for (index, (camera, name)) in passes.iter().enumerate() {
                    let clears_target = !matches!(camera.clear_color, ClearColorConfig::None);
                    assert_eq!(
                        clears_target,
                        index == 0,
                        "{name}: later cameras must load previous viewports; capture={capture}, selected={selected:?}, layout={layout:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn mouse_orbit_can_inspect_the_model_from_below_in_four_and_single_views() {
    for single in [false, true] {
        let mut app = app();
        let world = app.world_mut();
        key(world, KeyCode::ArrowRight);
        if single {
            key(world, KeyCode::F4);
        }
        let frame = world
            .resource::<Views>()
            .frame(UVec2::new(1440, 900), 1., true);
        let cursor = frame.rect(View::Orbit).unwrap().center().as_vec2();
        world
            .query::<&mut Window>()
            .single_mut(world)
            .unwrap()
            .set_cursor_position(Some(cursor));
        world
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        world.resource_mut::<AccumulatedMouseMotion>().delta = Vec2::new(0., -300.);
        world.run_system_cached(input).unwrap();
        world.run_system_cached(update_view).unwrap();

        let focus = position(0) + Vec3::Y * 0.9;
        let transform = world
            .query::<(&View, &Transform)>()
            .iter(world)
            .find_map(|(view, transform)| (*view == View::Orbit).then_some(*transform))
            .unwrap();
        let offset = (transform.translation - focus).normalize();
        assert!(
            offset.dot(Vec3::NEG_Y) > 0.95,
            "mouse orbit must reach the underside, not stop near the horizon: offset={offset:?}, single={single}"
        );
        assert!(transform.forward().dot(-offset) > 0.999);
    }
}

#[test]
fn divider_nodes_follow_the_layout_without_intercepting_pointer_input() {
    for capture in [false, true] {
        let mut app = app_with_capture(capture);
        let world = app.world_mut();
        world
            .query::<&mut Window>()
            .single_mut(world)
            .unwrap()
            .resolution = WindowResolution::new(2002, 1406).with_scale_factor_override(2.);
        for (selected, layout) in [
            (None, camera::Layout::Four),
            (Some(0), camera::Layout::Four),
            (Some(0), camera::Layout::Single(View::Front)),
        ] {
            world.resource_mut::<Zoo>().selected = selected;
            world.resource_mut::<Views>().layout = layout;
            world.run_system_cached(update_view).unwrap();
            let (size, scale) = if capture {
                (UVec2::new(1440, 900), 1.)
            } else {
                (UVec2::new(2002, 1406), 2.)
            };
            let frame =
                world
                    .resource::<Views>()
                    .frame(size, scale, selected.is_some() && !capture);
            let dividers: Vec<_> = world
                .query::<(
                    &Separator,
                    &Node,
                    &BackgroundColor,
                    &bevy::picking::Pickable,
                )>()
                .iter(world)
                .collect();
            assert_eq!(dividers.len(), 2);
            for (separator, node, color, picking) in dividers {
                assert!(!picking.is_hoverable && !picking.should_block_lower);
                assert!(color.0.alpha() > 0.);
                if let Some(rect) = frame.separator(*separator) {
                    assert_eq!(node.display, Display::Flex);
                    assert_eq!(node.left, px(rect.min.x as f32 / scale));
                    assert_eq!(node.top, px(rect.min.y as f32 / scale));
                    assert_eq!(node.width, px(rect.size().x as f32 / scale));
                    assert_eq!(node.height, px(rect.size().y as f32 / scale));
                } else {
                    assert_eq!(node.display, Display::None);
                }
            }
        }
    }
}
