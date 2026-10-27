use super::*;
use bevy::{
    input_focus::{FocusCause, InputFocus},
    text::apply_text_edits,
    window::PrimaryWindow,
};

#[test]
fn accepted_text_stays_bound_across_focus_despawn_and_editability_changes() {
    // Three isolated software edit schedules; no UI renderer, OS clipboard or game.
    for scenario in ["focus", "despawn", "not-editable"] {
        let mut app = App::new();
        app.add_plugins((bevy::asset::AssetPlugin::default(), bevy::text::TextPlugin));
        app.init_resource::<InputFocus>();
        app.world_mut().spawn((Window::default(), PrimaryWindow));
        let a = app.world_mut().spawn(EditableText::new("alpha:")).id();
        let b = app.world_mut().spawn(EditableText::new("beta:")).id();
        app.world_mut()
            .resource_mut::<InputFocus>()
            .set(a, FocusCause::Navigated);
        let mut text = State::default();
        text.input(
            app.world(),
            Input {
                text: "Grüße 🦉".into(),
            },
        )
        .unwrap();
        assert_eq!(
            app.world()
                .get::<EditableText>(a)
                .unwrap()
                .value()
                .to_string(),
            "alpha:"
        );
        app.world_mut()
            .resource_mut::<InputFocus>()
            .set(b, FocusCause::Navigated);
        let replacement = match scenario {
            "despawn" => {
                app.world_mut().despawn(a);
                Some(
                    app.world_mut()
                        .spawn(EditableText::new("replacement:"))
                        .id(),
                )
            }
            "not-editable" => {
                app.world_mut().entity_mut(a).remove::<EditableText>();
                None
            }
            "focus" => None,
            _ => unreachable!(),
        };
        text.flush(app.world_mut());
        app.world_mut().run_system_cached(apply_text_edits).unwrap(); // Explicit edit pass, no application tick.
        assert_eq!(
            app.world()
                .get::<EditableText>(b)
                .unwrap()
                .value()
                .to_string(),
            "beta:",
            "{scenario}"
        );
        if scenario == "focus" {
            assert_eq!(
                app.world()
                    .get::<EditableText>(a)
                    .unwrap()
                    .value()
                    .to_string(),
                "alpha:Grüße 🦉"
            );
        } else if let Some(entity) = replacement {
            assert_eq!(
                app.world()
                    .get::<EditableText>(entity)
                    .unwrap()
                    .value()
                    .to_string(),
                "replacement:"
            );
            assert!(app.world().get_entity(a).is_err());
        } else {
            assert!(app.world().get::<EditableText>(a).is_none());
            app.world_mut()
                .entity_mut(a)
                .insert(EditableText::new("restored:"));
        }
        // A later fresh target/edit pass must not revive discarded text.
        text.flush(app.world_mut());
        app.world_mut().run_system_cached(apply_text_edits).unwrap();
        if scenario == "focus" {
            assert_eq!(
                app.world()
                    .get::<EditableText>(a)
                    .unwrap()
                    .value()
                    .to_string(),
                "alpha:Grüße 🦉",
                "The second flush must not deliver accepted text twice"
            );
        } else if scenario == "not-editable" {
            assert_eq!(
                app.world()
                    .get::<EditableText>(a)
                    .unwrap()
                    .value()
                    .to_string(),
                "restored:"
            );
        }
        assert_eq!(
            app.world()
                .get::<EditableText>(b)
                .unwrap()
                .value()
                .to_string(),
            "beta:"
        );
    }
}
