use super::*;
use bevy::window::PrimaryWindow;

#[cfg(feature = "ui")]
fn text_app(value: &str, max_characters: Option<usize>) -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
        bevy::text::TextPlugin,
    ));
    // Selection needs a shaped layout, even when no renderer is involved.
    let font = bevy::text::Font::from_bytes(
        include_bytes!("../../../../test_app/asset/fonts/Barlow-Regular.ttf").to_vec(),
    );
    let mut fonts = app.world_mut().resource_mut::<bevy::text::FontCx>();
    fonts.collection.register_fonts(font.data, None);
    fonts.set_sans_serif_family("Barlow").unwrap();
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    let field = app
        .world_mut()
        .spawn(EditableText {
            max_characters,
            ..EditableText::new(value)
        })
        .id();
    app.world_mut()
        .insert_resource(InputFocus::from_entity(field));
    (app, field)
}

#[cfg(feature = "ui")]
#[test]
fn replacing_selected_text_respects_the_resulting_character_count() {
    let (mut app, field) = text_app("Suite", Some(24));
    app.world_mut()
        .get_mut::<EditableText>(field)
        .unwrap()
        .queue_edit(TextEdit::SelectAll);
    app.update();
    assert_eq!(
        app.world()
            .get::<EditableText>(field)
            .unwrap()
            .editor()
            .selected_text(),
        Some("Suite"),
        "the fixture must have a real selection before exercising replacement"
    );
    let mut state = State::default();
    let boundary = "Ä界🙂".repeat(8);
    state.input(app.world(), Input::new(&boundary)).unwrap();
    state.flush(app.world_mut());
    assert_eq!(
        app.world()
            .get::<EditableText>(field)
            .unwrap()
            .value()
            .to_string(),
        "Suite",
        "accepting and flushing input must not apply an edit before the app update"
    );
    app.update();
    assert_eq!(
        app.world()
            .get::<EditableText>(field)
            .unwrap()
            .value()
            .to_string(),
        boundary,
        "the selected text must not count against the replacement's character limit"
    );
    state.input(app.world(), Input::new("X")).unwrap();
    state.flush(app.world_mut());
    app.update();
    assert_eq!(
        app.world()
            .get::<EditableText>(field)
            .unwrap()
            .value()
            .to_string(),
        boundary,
        "an insertion beyond the field limit remains rejected by Bevy"
    );
}

#[cfg(feature = "ui")]
#[test]
fn text_commit_clears_composition_and_keeps_bevy_filtering() {
    use bevy::text::EditableTextFilter;
    let (mut app, field) = text_app("Grüße ", None);
    app.world_mut()
        .get_mut::<EditableText>(field)
        .unwrap()
        .queue_edit(TextEdit::ImeSetCompose {
            value: "Entwurf".into(),
            cursor: None,
        });
    app.update();
    assert!(
        app.world()
            .get::<EditableText>(field)
            .unwrap()
            .is_composing()
    );
    let mut state = State::default();
    state.input(app.world(), Input::new("世界🙂")).unwrap();
    state.flush(app.world_mut());
    app.update();
    let input = app.world().get::<EditableText>(field).unwrap();
    assert_eq!(input.value().to_string(), "Grüße 世界🙂");
    assert!(!input.is_composing());
    app.world_mut()
        .entity_mut(field)
        .insert(EditableTextFilter::new(|character| character != '!'));
    state.input(app.world(), Input::new("verboten!")).unwrap();
    state.flush(app.world_mut());
    app.update();
    assert_eq!(
        app.world()
            .get::<EditableText>(field)
            .unwrap()
            .value()
            .to_string(),
        "Grüße 世界🙂",
        "Bevy must reject the whole insertion when the field filter rejects a character"
    );
}

#[test]
fn rejects_oversized_utf8_and_unavailable_windows_before_focus() {
    let mut state = State::default();
    let mut world = World::new();
    for text in ["x".repeat(MAX_BYTES + 1), "🦜".repeat(MAX_BYTES / 4 + 1)] {
        assert_eq!(
            state.input(&world, Input::new(text)).unwrap_err().code,
            "text_too_large"
        );
    }
    assert_eq!(
        state.input(&world, Input::new("")).unwrap_err().code,
        "text_window_unavailable"
    );
    world.spawn(PrimaryWindow); // A marker without Window is not a usable window.
    assert_eq!(
        state.input(&world, Input::new("a")).unwrap_err().code,
        "text_window_unavailable"
    );
    world.spawn((Window::default(), PrimaryWindow));
    let duplicate = world.spawn((Window::default(), PrimaryWindow)).id();
    assert_eq!(
        state.input(&world, Input::new("a")).unwrap_err().code,
        "text_window_unavailable"
    );
    world.despawn(duplicate);
    assert_eq!(
        state.input(&world, Input::new("a")).unwrap_err().code,
        "text_focus_unavailable"
    );
}

#[cfg(feature = "ui")]
#[test]
fn validates_focus_and_queues_exact_utf8_limit_without_application_mutation() {
    let mut state = State::default();
    let mut world = World::new();
    world.spawn((
        Window {
            focused: false,
            ..default()
        },
        PrimaryWindow,
    ));
    world.init_resource::<InputFocus>();
    assert_eq!(
        state.input(&world, Input::new("a")).unwrap_err().code,
        "text_focus_unavailable"
    );
    let ordinary = world.spawn_empty().id();
    world.insert_resource(InputFocus::from_entity(ordinary));
    assert_eq!(
        state.input(&world, Input::new("a")).unwrap_err().code,
        "text_focus_unavailable"
    );
    world.despawn(ordinary);
    assert_eq!(
        state.input(&world, Input::new("a")).unwrap_err().code,
        "text_focus_unavailable"
    );
    let field = world.spawn(EditableText::default()).id();
    world.insert_resource(InputFocus::from_entity(field));
    let changed = world
        .entity(field)
        .get_change_ticks::<EditableText>()
        .unwrap()
        .changed;
    let text = "🦜".repeat(MAX_BYTES / 4);
    state.input(&world, Input::new(&text)).unwrap();
    state.input(&world, Input::new("")).unwrap();
    assert!(state.input(&world, Input::new(format!("{text}x"))).is_err());
    assert_eq!(state.pending.len(), 2);
    assert_eq!(
        world
            .get::<EditableText>(field)
            .unwrap()
            .value()
            .to_string(),
        ""
    );
    assert!(
        world
            .get::<EditableText>(field)
            .unwrap()
            .pending_edits
            .is_empty()
    );
    assert_eq!(
        world
            .entity(field)
            .get_change_ticks::<EditableText>()
            .unwrap()
            .changed,
        changed
    );
    state.flush(&mut world);
    assert_eq!(
        world.get::<EditableText>(field).unwrap().pending_edits,
        vec![
            TextEdit::ImeSetCompose {
                value: "".into(),
                cursor: None
            },
            TextEdit::Insert(text.into()),
            TextEdit::ImeSetCompose {
                value: "".into(),
                cursor: None
            },
            TextEdit::Insert("".into())
        ]
    );
    assert_eq!(
        world
            .get::<EditableText>(field)
            .unwrap()
            .value()
            .to_string(),
        ""
    );
    state.flush(&mut world);
    assert_eq!(
        world
            .get::<EditableText>(field)
            .unwrap()
            .pending_edits
            .len(),
        4
    );
}

#[cfg(feature = "ui")]
#[test]
fn focus_changes_and_despawns_do_not_redirect_accepted_text() {
    let mut state = State::default();
    let mut world = World::new();
    world.spawn((Window::default(), PrimaryWindow));
    let first = world.spawn(EditableText::default()).id();
    let second = world.spawn(EditableText::default()).id();
    world.insert_resource(InputFocus::from_entity(first));
    state.input(&world, Input::new("first")).unwrap();
    world.insert_resource(InputFocus::from_entity(second));
    state.input(&world, Input::new("second")).unwrap();
    state.flush(&mut world);
    assert_eq!(
        world.get::<EditableText>(first).unwrap().pending_edits,
        vec![
            TextEdit::ImeSetCompose {
                value: "".into(),
                cursor: None
            },
            TextEdit::Insert("first".into())
        ]
    );
    assert_eq!(
        world.get::<EditableText>(second).unwrap().pending_edits,
        vec![
            TextEdit::ImeSetCompose {
                value: "".into(),
                cursor: None
            },
            TextEdit::Insert("second".into())
        ]
    );
    state.input(&world, Input::new("discarded")).unwrap();
    world.despawn(second);
    let replacement = world.spawn(EditableText::default()).id();
    world.insert_resource(InputFocus::from_entity(replacement));
    state.flush(&mut world);
    assert!(
        world
            .get::<EditableText>(replacement)
            .unwrap()
            .pending_edits
            .is_empty()
    );
}
