//! Settings widgets and text drafts enter the settings interface.
use super::*;
pub(crate) fn notes_input(parent: &mut ChildSpawnerCommands, value: &str) {
    parent.spawn(text("NOTIZEN", 11.0, MUTED));
    parent
        .spawn((
            NotesInput,
            Name::new("Notizen"),
            TextInput,
            EditableText {
                allow_newlines: true,
                visible_lines: Some(5.0),
                ..EditableText::new(value)
            },
            TextLayout {
                linebreak: LineBreak::WordOrCharacter,
                ..default()
            },
            TextCursorStyle::default(),
            TabIndex(19),
            TextFont {
                font_size: FontSize::Px(16.0),
                ..default()
            },
            TextColor(INK),
            Node {
                width: percent(100),
                flex_shrink: 0.0,
                padding: px(10).all(),
                border: px(1).all(),
                border_radius: BorderRadius::all(px(5)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.03, 0.055, 0.065)),
            BorderColor::all(BORDER),
        ))
        .observe(
            |mut event: On<PointerScroll>,
             mut inputs: Query<&mut EditableText, With<NotesInput>>| {
                let Ok(mut input) = inputs.get_mut(event.entity) else {
                    return;
                };
                input.queue_edit(match event.unit {
                    MouseScrollUnit::Line => TextEdit::ScrollByLines(-event.y),
                    MouseScrollUnit::Pixel => TextEdit::ScrollBy(Vec2::new(0.0, -event.y)),
                });
                event.propagate(false);
            },
        );
    parent.spawn(text(
        "Enter: neue Zeile · Mausrad: scrollen · Esc: verlassen",
        11.0,
        MUTED,
    ));
}

#[expect(
    clippy::type_complexity,
    reason = "One query synchronizes the two editable settings fields."
)]
pub(crate) fn sync_text_fields(
    mut inputs: Query<
        (Entity, Ref<EditableText>, Has<NameInput>, &mut BorderColor),
        Or<(With<NameInput>, With<NotesInput>)>,
    >,
    focus: Res<InputFocus>,
    mut settings: ResMut<Settings>,
) {
    for (entity, editable, name, mut border) in &mut inputs {
        border.set_all(if focus.get() == Some(entity) {
            TEAL
        } else {
            BORDER
        });
        if !editable.is_changed() {
            continue;
        }
        let value = editable.value().to_string();
        if name {
            settings.set_name(&value);
        } else {
            settings.set_notes(&value);
        }
    }
}

pub(super) fn spawn(commands: &mut Commands, settings: &Settings) {
    commands.spawn((SettingsPanel, UiBlock, TabGroup::new(1), GlobalZIndex(10), Node {
        position_type: PositionType::Absolute, width: percent(100), height: percent(100),
        display: Display::None, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default()
    }, BackgroundColor(Color::srgba(0.015, 0.025, 0.035, 0.83))))
        .with_children(|p| {
            p.spawn(panel()).insert(Node { width: px(540), padding: px(24).all(), flex_direction: FlexDirection::Column,
                row_gap: px(12), border: px(1).all(), border_radius: BorderRadius::all(px(14)), ..default() })
                .with_children(|p| {
                    p.spawn(text("PAUSE", 12.0, TEAL));
                    p.spawn(text("Einstellungen", 30.0, INK));
                    p.spawn(text("Änderungen werden automatisch gespeichert.", 13.0, MUTED));
                    setting_row(p, Label::Sensitivity, Action::Sensitivity(-0.2), Action::Sensitivity(0.2), 10);
                    setting_row(p, Label::Fov, Action::Fov(-5.0), Action::Fov(5.0), 12);
                    setting_row(p, Label::Speed, Action::Speed(-0.5), Action::Speed(0.5), 14);
                    for (label, action, index) in [(Label::Path, Action::Path, 16), (Label::Shadows, Action::Shadows, 17), (Label::Invert, Action::Invert, 18)] {
                        p.spawn((Button, Hovered::default(), action, TabIndex(index), Node { width: percent(100), min_height: px(39),
                            padding: UiRect::axes(px(14), px(9)), border: px(1).all(), border_radius: BorderRadius::all(px(6)), ..default() },
                            BackgroundColor(BUTTON), BorderColor::all(BORDER)))
                            .with_children(|p| { p.spawn((text("", 14.0, INK), label)); });
                    }
                    notes_input(p, settings.notes());
                    p.spawn(text("FPV: WASD + Maus · Umschalt: sprinten\nEsc gibt die Maus frei. Klick ins Level bindet sie wieder.", 12.0, MUTED));
                    p.spawn(Node { column_gap: px(10), ..default() }).with_children(|row| {
                        button(row, "Zurück zum Spiel", Action::Resume, 20);
                        button(row, "Standardwerte", Action::Defaults, 21);
                    });
                    button(p, "Spiel beenden", Action::Quit, 22);
                });
        });
}
