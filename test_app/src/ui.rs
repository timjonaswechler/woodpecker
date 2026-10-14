mod fonts;
pub use fonts::apply as apply_font;

use crate::{
    camera::ViewState,
    coins::CoinRun,
    expedition::Expedition,
    items::ItemKind,
    level::{GOLD, TEAL},
    player::Player,
    settings::Settings,
    shooting::{self, Practice},
};
use bevy::{
    input::mouse::MouseScrollUnit,
    input_focus::{
        InputFocus,
        tab_navigation::{TabGroup, TabIndex},
    },
    picking::hover::Hovered,
    prelude::*,
    text::{EditableText, TextCursorStyle, TextEdit},
    ui::Pressed,
    ui_widgets::{Button, TextInput},
};

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct UiState {
    pub menu: bool,
    pub status: String,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            menu: false,
            status: "Lade die Signalstation …".into(),
        }
    }
}

const PANEL: Color = Color::srgba(0.045, 0.075, 0.09, 0.96);
const MUTED: Color = Color::srgb(0.57, 0.66, 0.69);
const INK: Color = Color::srgb(0.89, 0.94, 0.93);
const BORDER: Color = Color::srgb(0.19, 0.29, 0.31);
const BUTTON: Color = Color::srgb(0.095, 0.15, 0.17);

#[derive(Component)]
pub struct UiBlock;
#[derive(Component)]
pub struct NameInput;
#[derive(Component)]
pub struct NotesInput;
#[derive(Component)]
pub struct SettingsPanel;
#[derive(Component)]
pub struct Crosshair;
#[derive(Component)]
pub struct HudPanel;
#[derive(Component)]
pub struct InteractivePanel;
#[derive(Component)]
pub struct InteractionPrompt;

#[derive(Component, Clone, Copy, Debug)]
pub enum Action {
    View,
    Map,
    Settings,
    Resume,
    Restart,
    Range,
    Interact,
    Objective,
    Checkpoint,
    Sensitivity(f32),
    Speed(f32),
    Fov(f32),
    Path,
    Shadows,
    Invert,
    Defaults,
    Quit,
}
#[derive(Resource, Default)]
pub struct Actions(pub Vec<Action>);
#[derive(Component)]
pub enum Label {
    Progress,
    Status,
    Mode,
    Controls,
    Sensitivity,
    Speed,
    Fov,
    Path,
    Shadows,
    Invert,
    Clock,
    Title,
    Resource(ItemKind),
    Range,
    Crosshair,
    Objective,
    Interaction,
}

fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    )
}

fn panel() -> impl Bundle {
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

fn button(parent: &mut ChildSpawnerCommands, title: &str, action: Action, tab: i32) {
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

fn setting_row(
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

fn notes_input(parent: &mut ChildSpawnerCommands, value: &str) {
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

pub fn spawn_ui(
    mut commands: Commands,
    settings: Res<Settings>,
    assets: Res<AssetServer>,
    mut tracked: ResMut<crate::level::LevelAssets>,
) {
    commands.insert_resource(fonts::load(&assets, &mut tracked));
    // Draw the HUD after both the level and the FPV weapon pass.
    commands.spawn((
        Name::new("HUD-Kamera"),
        Camera2d,
        IsDefaultUiCamera,
        Camera {
            order: 2,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        bevy::camera::visibility::RenderLayers::layer(2),
        Msaa::Sample4,
    ));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(24),
                right: px(24),
                top: px(20),
                height: px(70),
                padding: UiRect::axes(px(22), px(10)),
                column_gap: px(30),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceEvenly,
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            BackgroundColor(PANEL),
            UiBlock,
            HudPanel,
        ))
        .with_children(|bar| {
            for kind in ItemKind::ALL {
                let icon: Handle<Image> = assets.load(kind.icon());
                tracked.0.push(icon.clone().untyped());
                bar.spawn(Node {
                    align_items: AlignItems::Center,
                    column_gap: px(12),
                    ..default()
                })
                .with_children(|slot| {
                    slot.spawn((
                        ImageNode::new(icon),
                        Node {
                            width: px(42),
                            height: px(42),
                            ..default()
                        },
                    ));
                    slot.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(1),
                        ..default()
                    })
                    .with_children(|p| {
                        p.spawn(text(kind.label(), 12., kind.color()));
                        p.spawn((text("0", 25., INK), Label::Resource(kind)));
                    });
                });
            }
        });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(24),
                top: px(110),
                width: px(295),
                padding: px(20).all(),
                flex_direction: FlexDirection::Column,
                row_gap: px(7),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            BackgroundColor(PANEL),
            UiBlock,
            HudPanel,
        ))
        .with_children(|p| {
            p.spawn(text("SIGNALSTATION  /  ERKUNDUNG", 12.0, TEAL));
            p.spawn((text("Signalstation", 30.0, INK), Label::Title));
            p.spawn((text("", 16.0, INK), Label::Objective));
            p.spawn((
                text("", 21.0, GOLD),
                Label::Progress,
                Node {
                    margin: px(10).top(),
                    ..default()
                },
            ));
            p.spawn((text("", 13.0, MUTED), Label::Clock));
            p.spawn((text("", 14.0, TEAL), Label::Range));
        });
    commands
        .spawn((panel(), TabGroup::new(0), HudPanel, InteractivePanel))
        .insert(Node {
            position_type: PositionType::Absolute,
            right: px(24),
            top: px(110),
            width: px(245),
            padding: px(18).all(),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            border: px(1).all(),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        })
        .with_children(|p| {
            p.spawn(text("DEIN NAME", 11.0, MUTED));
            p.spawn((
                NameInput,
                Name::new("Spielername"),
                TextInput,
                EditableText {
                    max_characters: Some(24),
                    visible_width: Some(14.0),
                    ..EditableText::new(&settings.player_name)
                },
                TextLayout::no_wrap(),
                TextCursorStyle::default(),
                TabIndex(0),
                TextFont {
                    font_size: FontSize::Px(17.0),
                    ..default()
                },
                TextColor(INK),
                Node {
                    width: percent(100),
                    min_height: px(42),
                    padding: px(10).all(),
                    border: px(1).all(),
                    border_radius: BorderRadius::all(px(5)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.03, 0.055, 0.065)),
                BorderColor::all(BORDER),
            ));
            p.spawn(text("Anklicken, schreiben, Enter bestätigen", 11.0, MUTED));
            button(p, "Ansicht wechseln  [V]", Action::View, 1);
            button(p, "Gesamtkarte  [G]", Action::Map, 8);
            button(p, "Einstellungen  [M]", Action::Settings, 2);
            button(p, "Neue Runde  [R]", Action::Restart, 3);
            button(p, "Zum nächsten Auftrag", Action::Objective, 4);
            button(p, "Interagieren  [E]", Action::Interact, 5);
            button(p, "Zum Schießstand", Action::Range, 6);
        });
    commands
        .spawn((panel(), HudPanel))
        .insert(Node {
            position_type: PositionType::Absolute,
            left: px(24),
            bottom: px(106),
            width: px(325),
            padding: px(16).all(),
            flex_direction: FlexDirection::Column,
            row_gap: px(9),
            border: px(1).all(),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        })
        .with_children(|p| {
            p.spawn((text("", 13.0, TEAL), Label::Mode));
            p.spawn((text("", 14.0, INK), Label::Controls));
        });
    commands
        .spawn((
            HudPanel,
            InteractionPrompt,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: percent(50),
                margin: px(32).top(),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn((
                Node {
                    padding: UiRect::axes(px(16), px(10)),
                    border_radius: BorderRadius::all(px(8)),
                    ..default()
                },
                BackgroundColor(PANEL),
            ))
            .with_children(|p| {
                p.spawn((text("", 18.0, INK), Label::Interaction));
            });
        });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(24),
                left: px(24),
                right: px(24),
                min_height: px(60),
                padding: px(18).all(),
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(px(10)),
                ..default()
            },
            BackgroundColor(PANEL),
            UiBlock,
        ))
        .with_children(|p| {
            p.spawn((text("", 16.0, INK), Label::Status));
        });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: percent(50),
                margin: UiRect {
                    left: px(-7),
                    top: px(-13),
                    ..default()
                },
                ..default()
            },
            Crosshair,
        ))
        .with_children(|p| {
            p.spawn((text("+", 24.0, INK), Label::Crosshair));
        });

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
                    notes_input(p, &settings.notes);
                    p.spawn(text("FPV: WASD + Maus · Umschalt: sprinten\nEsc gibt die Maus frei. Klick ins Level bindet sie wieder.", 12.0, MUTED));
                    p.spawn(Node { column_gap: px(10), ..default() }).with_children(|row| {
                        button(row, "Zurück zum Spiel", Action::Resume, 20);
                        button(row, "Standardwerte", Action::Defaults, 21);
                    });
                    button(p, "Spiel beenden", Action::Quit, 22);
                });
        });
}

#[expect(
    clippy::type_complexity,
    reason = "Button activation and styling share the same Bevy query."
)]
pub fn buttons(
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

#[expect(
    clippy::type_complexity,
    reason = "Both persisted text fields share focus styling and saving."
)]
pub fn sync_text_fields(
    mut inputs: Query<
        (Entity, Ref<EditableText>, Has<NameInput>, &mut BorderColor),
        Or<(With<NameInput>, With<NotesInput>)>,
    >,
    focus: Res<InputFocus>,
    mut settings: ResMut<Settings>,
    mut ui: ResMut<UiState>,
) {
    let mut changed = false;
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
        let (stored, value) = if name {
            let value = if value.trim().is_empty() {
                "Spieler"
            } else {
                value.trim()
            };
            (&mut settings.player_name, value)
        } else {
            (&mut settings.notes, value.as_str())
        };
        if stored != value {
            *stored = value.to_owned();
            changed = true;
        }
    }
    if changed && let Err(error) = settings.save() {
        ui.status = format!("Texteingabe konnte nicht gespeichert werden: {error}");
    }
}

#[expect(
    clippy::type_complexity,
    clippy::too_many_arguments,
    reason = "UI markers select which visibility rule applies to each node."
)]
pub fn update_ui(
    mode: Res<crate::input::InputMode>,
    player: Res<Player>,
    view: Res<ViewState>,
    run: Res<CoinRun>,
    ui: Res<UiState>,
    settings: Res<Settings>,
    practice: Res<Practice>,
    expedition: Res<Expedition>,
    nav: Res<crate::navigation::Navigation>,
    focus: Res<InputFocus>,
    inputs: Query<(), With<EditableText>>,
    mut labels: Query<(&Label, &mut Text)>,
    mut nodes: Query<(
        &mut Node,
        Has<SettingsPanel>,
        Has<Crosshair>,
        Has<HudPanel>,
        Has<InteractivePanel>,
        Has<InteractionPrompt>,
    )>,
    window: Single<&Window>,
    mut scale: ResMut<UiScale>,
    hud_camera: Single<&Camera, With<IsDefaultUiCamera>>,
) {
    let viewport = hud_camera
        .logical_viewport_size()
        .unwrap_or(Vec2::new(window.width(), window.height()));
    let desired_scale = (viewport.x / 1440.0)
        .min(viewport.y / 900.0)
        .clamp(0.65, 1.5);
    if scale.0 != desired_scale {
        scale.0 = desired_scale;
    }
    let interaction = if ui.menu
        || !mode.accepts_input(&window)
        || focus.get().is_some_and(|entity| inputs.contains(entity))
    {
        crate::expedition::Interaction::None
    } else {
        expedition.interaction(&player, &nav, &view)
    };
    for (label, mut value) in &mut labels {
        let new = match label {
            Label::Title => if expedition.finished { "Signal aktiv".into() }
                else {crate::blueprint::room(player.position).map_or("Wartungsgang".into(),|r|format!("{} · {}",r.id,r.name))},
            Label::Resource(kind) => kind.count(&run, &expedition).to_string(),
            Label::Objective => expedition.objective().into(),
            Label::Interaction => interaction.prompt().into(),
            Label::Crosshair => if practice.hit_flash > 0.0 { "×" } else { "+" }.into(),
            Label::Range => if shooting::in_range(player.position) {
                if expedition.practice_reward { "Bauplan-Truhe entriegelt · hinten rechts".into() }
                else {
                    let distance = shooting::TARGETS.iter().enumerate().filter(|(i,_)| !practice.targets_hit[*i])
                        .map(|(_,p)| p.xz().distance(player.position)).fold(f32::INFINITY, f32::min);
                    format!("Mindestabstand {:.0} m · nächstes Ziel {distance:.1} m\n{} gültige Treffer · V für FPV", shooting::MIN_TARGET_DISTANCE, practice.hits)
                }
            } else { String::new() },
            Label::Progress => if shooting::in_range(player.position) && player.height<0.2 {format!("{} Punkte",practice.score)}
                else {format!("Ziel {:.0} m · Ebene {}",player.feet().distance(expedition.destination()),if player.height>2.8 {"oben"} else {"unten"})},
            Label::Clock => format!(
                "{:02}:{:02}  ·  {}",
                expedition.elapsed as u32 / 60,
                expedition.elapsed as u32 % 60,
                settings.player_name
            ),
            Label::Status => ui.status.clone(),
            Label::Mode => if view.first_person {
                "02 / EGOPERSPEKTIVE"
            } else {
                "01 / ÜBERSICHT"
            }
            .into(),
            Label::Controls => if view.first_person {
                if view.captured {
                    "WASD + Maus · Leertaste springen\nUmschalt sprinten · E interagieren\nLinksklick schießen · Esc Maus freigeben"
                } else {
                    "Klicke ins Level, um weiterzuspielen.\nV Übersicht · M Einstellungen"
                }
            } else {
                "Pfeiltasten · Klickweg · V FPV · G Karte\nLeertaste springen · Umschalt sprinten\nE interagieren · Backspace Checkpoint"
            }
            .into(),
            Label::Sensitivity => format!("Maus-Empfindlichkeit    {:.1}", settings.sensitivity),
            Label::Speed => format!("Lauftempo    {:.1} m/s", settings.speed),
            Label::Fov => format!("FPV-Sichtfeld    {:.0}°", settings.fov),
            Label::Path => format!(
                "Laufroute anzeigen: {}",
                if settings.show_path { "An" } else { "Aus" }
            ),
            Label::Shadows => format!("Schatten: {}", if settings.shadows { "An" } else { "Aus" }),
            Label::Invert => format!(
                "Maus Y-Achse umkehren: {}",
                if settings.invert_y { "An" } else { "Aus" }
            ),
        };
        if value.0 != new {
            value.0 = new;
        }
    }
    for (mut node, menu, crosshair, hud, interactive, prompt) in &mut nodes {
        if menu {
            node.display = if ui.menu {
                Display::Flex
            } else {
                Display::None
            };
        }
        if crosshair {
            node.display = if view.first_person && view.captured && !ui.menu {
                Display::Flex
            } else {
                Display::None
            };
        }
        if prompt {
            node.top = if view.first_person {
                percent(50)
            } else {
                Val::Auto
            };
            node.bottom = if view.first_person {
                Val::Auto
            } else {
                px(110)
            };
        }
        if hud {
            node.display = if ui.menu
                || (interactive && view.first_person && view.captured)
                || (prompt && interaction == crate::expedition::Interaction::None)
            {
                Display::None
            } else {
                Display::Flex
            };
        }
    }
}

/// Hit test the actual UI layout, including display scale and window resizing.
pub fn pointer_over_ui(
    pointer: Vec2,
    scale_factor: f32,
    blocks: &Query<(&ComputedNode, &UiGlobalTransform), With<UiBlock>>,
) -> bool {
    let pointer = pointer * scale_factor;
    blocks.iter().any(|(node, transform)| {
        node.size().min_element() > 0.0
            && transform.try_inverse().is_some_and(|inverse| {
                let local = inverse.transform_point2(pointer);
                (local.abs().cmple(node.size() * 0.5)).all()
            })
    })
}
