use super::*;

#[test]
fn detail_frame_has_four_non_overlapping_viewports_and_scales_with_dpi() {
    let views = Views::default();
    let frame = views.frame(UVec2::new(1440, 900), 1., true);
    let high_dpi = views.frame(UVec2::new(2880, 1800), 2., true);
    for view in View::ALL {
        let rect = frame.rect(view).unwrap();
        assert!(rect.min.cmpge(UVec2::new(12, 110)).all());
        assert!(rect.max.cmple(UVec2::new(1428, 812)).all());
        assert!(rect.size().min_element() > 0);
        let scaled = high_dpi.rect(view).unwrap();
        assert_eq!(scaled.min, rect.min * 2);
        assert_eq!(scaled.max, rect.max * 2);
        for other in View::ALL {
            if other == view {
                continue;
            }
            let other = frame.rect(other).unwrap();
            assert!(rect.max.cmple(other.min).any() || other.max.cmple(rect.min).any());
        }
    }
}

#[test]
fn single_view_toggles_back_to_four_without_resetting_orbit_or_zoom() {
    let mut views = Views::default();
    views.yaw = 1.2;
    views.pitch = 0.8;
    views.zoom(Some(View::Orbit), 2.);
    let orbit_height = views.height(View::Orbit, 3.2);
    for view in View::ALL {
        views.toggle(view);
        assert_eq!(views.layout, Layout::Single(view));
        let frame = views.frame(UVec2::new(1440, 900), 1., true);
        for other in View::ALL {
            assert_eq!(frame.rect(other).is_some(), other == view);
        }
        assert_eq!(
            frame.rect(view).unwrap(),
            URect::from_corners(UVec2::new(12, 110), UVec2::new(1428, 812))
        );
        views.toggle(view);
        assert_eq!(views.layout, Layout::Four);
    }
    assert_eq!(views.yaw, 1.2);
    assert_eq!(views.pitch, 0.8);
    assert_eq!(views.height(View::Orbit, 3.2), orbit_height);
}

#[test]
fn overview_and_capture_use_only_the_full_size_orbit_view() {
    let views = Views {
        layout: Layout::Single(View::Front),
        ..default()
    };
    let frame = views.frame(UVec2::new(1440, 900), 1., false);
    for view in [View::Top, View::Front, View::Right] {
        assert!(frame.rect(view).is_none());
    }
    assert_eq!(
        frame.rect(View::Orbit).unwrap(),
        URect::from_corners(UVec2::ZERO, UVec2::new(1440, 900))
    );
}

#[test]
fn fixed_views_look_along_the_axes_and_top_view_has_a_valid_up_direction() {
    let views = Views::default();
    let focus = Vec3::new(-3., 0.9, 6.);
    for (view, direction, up) in [
        (View::Top, Vec3::NEG_Y, Vec3::NEG_Z),
        (View::Front, Vec3::NEG_Z, Vec3::Y),
        (View::Right, Vec3::NEG_X, Vec3::Y),
    ] {
        let transform = views.transform(view, focus, false);
        assert!(transform.rotation.is_finite());
        assert!(transform.forward().dot(direction) > 0.999);
        assert!(transform.up().dot(up) > 0.999);
        assert!((transform.translation.distance(focus) - 35.).abs() < 0.001);
    }
}

#[test]
fn only_orbit_drags_and_zoom_is_local_to_the_hovered_view() {
    let mut views = Views::default();
    let initial = (views.yaw, views.pitch);
    for view in [View::Top, View::Front, View::Right] {
        views.drag(Some(view), true, true, Vec2::new(20., 10.));
        assert_eq!((views.yaw, views.pitch), initial);
    }
    views.drag(Some(View::Orbit), true, true, Vec2::new(20., 10.));
    assert_ne!((views.yaw, views.pitch), initial);
    views.zoom(Some(View::Front), 3.);
    assert!(views.height(View::Front, 3.2) < 3.2);
    for view in [View::Top, View::Right, View::Orbit] {
        assert_eq!(views.height(view, 3.2), 3.2);
    }
    views.reset();
    assert_eq!((views.yaw, views.pitch), initial);
    assert_eq!(views.height(View::Front, 3.2), 3.2);
}

#[test]
fn drags_must_start_in_orbit_and_pane_titles_do_not_accept_camera_input() {
    let mut views = Views::default();
    let initial = views.yaw;
    views.drag(Some(View::Front), true, true, Vec2::X * 10.);
    views.drag(Some(View::Orbit), true, false, Vec2::X * 10.);
    assert_eq!(views.yaw, initial);
    views.drag(None, false, false, Vec2::ZERO);
    views.drag(Some(View::Orbit), true, true, Vec2::X * 10.);
    assert_ne!(views.yaw, initial);

    let frame = views.frame(UVec2::new(2880, 1800), 2., true);
    let orbit = frame.rect(View::Orbit).unwrap();
    let title = orbit.min.as_vec2() / 2. + Vec2::splat(12.);
    assert_eq!(frame.hovered(Some(title)), None);
    let center = orbit.center().as_vec2() / 2.;
    assert_eq!(frame.hovered(Some(center)), Some(View::Orbit));
    assert_eq!(frame.hovered(None), None);
    assert_eq!(frame.hovered(Some(Vec2::splat(-1.))), None);
    assert_eq!(frame.hovered(Some(Vec2::splat(f32::NAN))), None);
}

#[test]
fn resized_odd_and_minimized_surfaces_never_produce_invalid_viewports() {
    for size in [
        UVec2::new(1001, 703),
        UVec2::new(320, 240),
        UVec2::ZERO,
        UVec2::ONE,
    ] {
        let frame = Views::default().frame(size, 1., true);
        for view in View::ALL {
            if let Some(viewport) = frame.viewport(view) {
                assert!(viewport.physical_size.min_element() > 0);
                assert!(
                    (viewport.physical_position + viewport.physical_size)
                        .cmple(size)
                        .all()
                );
            }
        }
    }
}

#[test]
fn orbit_stays_continuous_when_crossing_above_and_below_the_model() {
    for pole in [std::f32::consts::FRAC_PI_2, -std::f32::consts::FRAC_PI_2] {
        let focus = Vec3::new(-3., 0.9, 6.);
        let mut views = Views::default();
        views.pitch = pole - 0.0001;
        let before = views.transform(View::Orbit, focus, false);
        views.pitch = pole + 0.0001;
        let after = views.transform(View::Orbit, focus, false);
        assert!(
            before.rotation.angle_between(after.rotation).abs() < 0.01,
            "crossing a pole must not flip the camera's up direction"
        );
        views.pitch = pole;
        let at_pole = views.transform(View::Orbit, focus, false);
        assert!(at_pole.rotation.is_finite());
        assert!(
            at_pole
                .forward()
                .dot((focus - at_pole.translation).normalize())
                > 0.999
        );
    }
}

#[test]
fn mouse_orbit_can_complete_a_full_vertical_revolution() {
    let mut views = Views::default();
    let focus = Vec3::new(-3., 0.9, 6.);
    let initial = views.transform(View::Orbit, focus, false);
    views.drag(
        Some(View::Orbit),
        true,
        true,
        Vec2::new(0., std::f32::consts::TAU / 0.006),
    );
    let final_pose = views.transform(View::Orbit, focus, false);
    assert!(
        final_pose.translation.distance(initial.translation) < 0.001,
        "a full vertical turn must return to the initial orbit position"
    );
    assert!(final_pose.rotation.angle_between(initial.rotation).abs() < 0.001);
}

#[test]
fn separators_stay_in_the_viewport_gaps_and_scale_with_the_window() {
    for (size, scale) in [(UVec2::new(1440, 900), 1.), (UVec2::new(2002, 1406), 2.)] {
        let frame = Views::default().frame(size, scale, true);
        let top = frame.rect(View::Top).unwrap();
        let front = frame.rect(View::Front).unwrap();
        let right = frame.rect(View::Right).unwrap();
        let orbit = frame.rect(View::Orbit).unwrap();
        let vertical = frame.separator(Separator::Vertical).unwrap();
        assert_eq!(vertical.size().x, (2. * scale) as u32);
        assert!(vertical.min.x >= top.max.x && vertical.max.x <= front.min.x);
        assert_eq!(vertical.min.y, top.min.y);
        assert_eq!(vertical.max.y, orbit.max.y);

        let horizontal = frame.separator(Separator::Horizontal).unwrap();
        assert_eq!(horizontal.size().y, (2. * scale) as u32);
        assert!(horizontal.min.y >= top.max.y && horizontal.max.y <= right.min.y);
        assert_eq!(horizontal.min.x, top.min.x);
        assert_eq!(horizontal.max.x, front.max.x);
        for rect in [vertical, horizontal] {
            assert_eq!(frame.hovered(Some(rect.center().as_vec2() / scale)), None);
        }
    }
}

#[test]
fn separators_are_absent_in_single_overview_capture_and_minimized_views() {
    let mut views = Views::default();
    for view in View::ALL {
        views.layout = Layout::Single(view);
        let frame = views.frame(UVec2::new(1440, 900), 1., true);
        assert!(frame.separator(Separator::Vertical).is_none());
        assert!(frame.separator(Separator::Horizontal).is_none());
    }
    views.show_all();
    for frame in [
        views.frame(UVec2::new(1440, 900), 1., false),
        views.frame(UVec2::ZERO, 1., true),
    ] {
        assert!(frame.separator(Separator::Vertical).is_none());
        assert!(frame.separator(Separator::Horizontal).is_none());
    }
}
