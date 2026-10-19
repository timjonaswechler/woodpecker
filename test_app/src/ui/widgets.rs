//! Shared widget construction and activation.
use super::*;
pub(crate) fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    )
}

pub(crate) fn panel() -> impl Bundle {
    (
        Node {
            padding: px(20).all(),
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            border: px(1).all(),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        BackgroundColor(PANEL),
        BorderColor::all(BORDER),
        UiBlock,
    )
}

pub(crate) fn button(parent: &mut ChildSpawnerCommands, title: &str, action: Action, tab: i32) {
    parent
        .spawn((
            Button,
            Hovered::default(),
            action,
            TabIndex(tab),
            Node {
                min_height: px(38),
                padding: UiRect::axes(px(14), px(9)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: px(1).all(),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            BackgroundColor(BUTTON),
            BorderColor::all(BORDER),
        ))
        .with_children(|parent| {
            parent.spawn(text(title, 14.0, INK));
        });
}

pub(crate) fn setting_row(
    parent: &mut ChildSpawnerCommands,
    label: Label,
    minus: Action,
    plus: Action,
    tab: i32,
) {
    parent
        .spawn(Node {
            width: percent(100),
            align_items: AlignItems::Center,
            column_gap: px(8),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                text("", 16.0, INK),
                label,
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
            button(row, "−", minus, tab);
            button(row, "+", plus, tab + 1);
        });
}

#[expect(
    clippy::type_complexity,
    reason = "Button appearance and activation use the same Bevy query."
)]
pub(crate) fn buttons(
    mut buttons: Query<(
        Entity,
        &Hovered,
        Option<Ref<Pressed>>,
        &Action,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
    focus: Res<InputFocus>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut actions: ResMut<Actions>,
) {
    let mut keyboard_activation = false;
    for (entity, hovered, pressed, action, mut background, mut border) in &mut buttons {
        let focused = focus.get() == Some(entity);
        let hovered = hovered.get() || pressed.is_some();
        background.0 = if pressed.is_some() {
            Color::srgb(0.12, 0.34, 0.3)
        } else if hovered {
            Color::srgb(0.15, 0.24, 0.26)
        } else {
            BUTTON
        };
        border.set_all(if focused || hovered { TEAL } else { BORDER });
        if pressed.as_ref().is_some_and(|pressed| pressed.is_added())
            || (focused && keys.any_just_pressed([KeyCode::Enter, KeyCode::Space]))
        {
            actions.0.push(*action);
            keyboard_activation |= focused && keys.just_pressed(KeyCode::Space);
        }
    }
    if keyboard_activation {
        keys.clear_just_pressed(KeyCode::Space);
    }
}
