use super::*;
use bevy::{
    input_focus::{FocusCause, InputFocus},
    text::{TextReadWriteMode, apply_text_edits},
    ui::InteractionDisabled,
    window::PrimaryWindow,
};

#[test]
fn accepted_text_stays_bound_across_focus_despawn_and_editability_changes() {
    // Isolated software edit schedules; no UI renderer, OS clipboard or game.
    for scenario in [
        "focus",
        "despawn",
        "not-editable",
        "readonly",
        "static",
        "disabled",
    ] {
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
        // Non-editable targets must be rejected before any edit is queued.
        match scenario {
            "readonly" => {
                app.world_mut()
                    .entity_mut(a)
                    .insert(TextReadWriteMode::ReadOnly);
            }
            "static" => {
                app.world_mut()
                    .entity_mut(a)
                    .insert(TextReadWriteMode::Static);
            }
            "disabled" => {
                app.world_mut().entity_mut(a).insert(InteractionDisabled);
            }
            _ => {}
        }
        if matches!(scenario, "readonly" | "static" | "disabled") {
            let error = text
                .input(
                    app.world(),
                    Input {
                        text: "blocked".into(),
                    },
                )
                .unwrap_err();
            assert_eq!(error.code, "text_focus_unavailable", "{scenario}");
            app.world_mut()
                .entity_mut(a)
                .remove::<InteractionDisabled>();
            app.world_mut()
                .entity_mut(a)
                .insert(TextReadWriteMode::Editable);
        }
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
            "readonly" => {
                app.world_mut()
                    .entity_mut(a)
                    .insert(TextReadWriteMode::ReadOnly);
                None
            }
            "static" => {
                app.world_mut()
                    .entity_mut(a)
                    .insert(TextReadWriteMode::Static);
                None
            }
            "disabled" => {
                app.world_mut().entity_mut(a).insert(InteractionDisabled);
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
        } else if scenario == "not-editable" {
            assert!(app.world().get::<EditableText>(a).is_none());
            app.world_mut()
                .entity_mut(a)
                .insert(EditableText::new("restored:"));
        } else {
            assert_eq!(
                app.world()
                    .get::<EditableText>(a)
                    .unwrap()
                    .value()
                    .to_string(),
                "alpha:",
                "{scenario}: edits accepted before the target became non-editable must be discarded"
            );
            app.world_mut()
                .entity_mut(a)
                .remove::<InteractionDisabled>();
            app.world_mut()
                .entity_mut(a)
                .insert(TextReadWriteMode::Editable);
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
        } else if scenario != "despawn" {
            assert_eq!(
                app.world()
                    .get::<EditableText>(a)
                    .unwrap()
                    .value()
                    .to_string(),
                if scenario == "not-editable" {
                    "restored:"
                } else {
                    "alpha:"
                },
                "{scenario}: discarded text must not revive when editability returns"
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
