use crate::{
    camera::ViewState,
    coins::CoinRun,
    level::{GOLD, TEAL},
    player::Player,
    settings::Settings,
    shooting::{self, Practice},
};
use bevy::{
    input_focus::{
        InputFocus,
        tab_navigation::{TabGroup, TabIndex},
    },
    prelude::*,
    text::{EditableText, TextCursorStyle},
};

#[derive(Resource)]
pub struct UiState {
    pub menu: bool,
    pub status: String,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            menu: false,
            status: "Lade den Parcours …".into(),
        }
    }
}

const PANEL: Color = Color::srgba(0.045, 0.075, 0.09, 0.96);
const MUTED: Color = Color::srgb(0.57, 0.66, 0.69);
const INK: Color = Color::srgb(0.89, 0.94, 0.93);
const BORDER: Color = Color::srgb(0.19, 0.29, 0.31);
const BUTTON: Color = Color::srgb(0.095, 0.15, 0.17);

pub fn apply_font(assets: Res<AssetServer>, mut fonts: Query<&mut TextFont, Added<TextFont>>) {
    for mut font in &mut fonts {
        font.font = assets.load("fonts/Barlow-Regular.ttf").into();
    }
}

#[derive(Component)]
pub struct UiBlock;
#[derive(Component)]
pub struct NameInput;
#[derive(Component)]
pub struct SettingsPanel;
#[derive(Component)]
pub struct Crosshair;
#[derive(Component)]
pub struct HudPanel;
#[derive(Component)]
pub struct InteractivePanel;

#[derive(Component, Clone, Copy, Debug)]
pub enum Action {
    View,
    Settings,
    Resume,
    Restart,
    Range,
    Practice,
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
    Subtitle,
    Exercise,
    Crosshair,
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

pub fn spawn_ui(mut commands: Commands, settings: Res<Settings>) {
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
                top: px(24),
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
            p.spawn(text("KAYKIT  /  SPIELDEMO", 12.0, TEAL));
            p.spawn((text("Münzparcours", 30.0, INK), Label::Title));
            p.spawn((text("", 14.0, MUTED), Label::Subtitle));
            p.spawn((
                text("", 21.0, GOLD),
                Label::Progress,
                Node {
                    margin: px(10).top(),
                    ..default()
                },
            ));
            p.spawn((text("", 13.0, MUTED), Label::Clock));
            p.spawn((text("", 14.0, TEAL), Label::Exercise));
        });
    commands
        .spawn((panel(), TabGroup::new(0), HudPanel, InteractivePanel))
        .insert(Node {
            position_type: PositionType::Absolute,
            right: px(24),
            top: px(24),
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
            button(p, "Einstellungen  [M]", Action::Settings, 2);
            button(p, "Neue Runde  [R]", Action::Restart, 3);
            button(p, "Zum Schießstand", Action::Range, 4);
            button(p, "Zielübung starten  [E]", Action::Practice, 5);
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
            p.spawn(panel()).insert(Node { width: px(490), padding: px(28).all(), flex_direction: FlexDirection::Column,
                row_gap: px(15), border: px(1).all(), border_radius: BorderRadius::all(px(14)), ..default() })
                .with_children(|p| {
                    p.spawn(text("PAUSE", 12.0, TEAL));
                    p.spawn(text("Einstellungen", 30.0, INK));
                    p.spawn(text("Änderungen werden automatisch gespeichert.", 13.0, MUTED));
                    setting_row(p, Label::Sensitivity, Action::Sensitivity(-0.2), Action::Sensitivity(0.2), 10);
                    setting_row(p, Label::Fov, Action::Fov(-5.0), Action::Fov(5.0), 12);
                    setting_row(p, Label::Speed, Action::Speed(-0.5), Action::Speed(0.5), 14);
                    for (label, action, index) in [(Label::Path, Action::Path, 16), (Label::Shadows, Action::Shadows, 17), (Label::Invert, Action::Invert, 18)] {
                        p.spawn((Button, action, TabIndex(index), Node { width: percent(100), min_height: px(39),
                            padding: UiRect::axes(px(14), px(9)), border: px(1).all(), border_radius: BorderRadius::all(px(6)), ..default() },
                            BackgroundColor(BUTTON), BorderColor::all(BORDER)))
                            .with_children(|p| { p.spawn((text("", 14.0, INK), label)); });
                    }
                    p.spawn(text("FPV: WASD + Maus · Umschalt: sprinten\nEsc gibt die Maus frei. Klick ins Level bindet sie wieder.", 12.0, MUTED));
                    p.spawn(Node { column_gap: px(10), ..default() }).with_children(|row| {
                        button(row, "Zurück zum Spiel", Action::Resume, 19);
                        button(row, "Standardwerte", Action::Defaults, 20);
                    });
                    button(p, "Spiel beenden", Action::Quit, 21);
                });
        });
}

pub fn buttons(
    mut buttons: Query<(
        Entity,
        Ref<Interaction>,
        &Action,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
    focus: Res<InputFocus>,
    keys: Res<ButtonInput<KeyCode>>,
    mut actions: ResMut<Actions>,
) {
    for (entity, interaction, action, mut background, mut border) in &mut buttons {
        let focused = focus.get() == Some(entity);
        let hovered = *interaction != Interaction::None;
        background.0 = if *interaction == Interaction::Pressed {
            Color::srgb(0.12, 0.34, 0.3)
        } else if hovered {
            Color::srgb(0.15, 0.24, 0.26)
        } else {
            BUTTON
        };
        border.set_all(if focused || hovered { TEAL } else { BORDER });
        if (interaction.is_changed() && *interaction == Interaction::Pressed)
            || (focused && keys.any_just_pressed([KeyCode::Enter, KeyCode::Space]))
        {
            actions.0.push(*action);
        }
    }
}

pub fn sync_name(
    inputs: Query<(Entity, &EditableText), With<NameInput>>,
    focus: Res<InputFocus>,
    mut borders: Query<&mut BorderColor, With<NameInput>>,
    mut settings: ResMut<Settings>,
    mut ui: ResMut<UiState>,
) {
    for (entity, editable) in &inputs {
        if let Ok(mut border) = borders.get_mut(entity) {
            border.set_all(if focus.get() == Some(entity) {
                TEAL
            } else {
                BORDER
            });
        }
        let value = editable.value().to_string();
        let name = if value.trim().is_empty() {
            "Spieler"
        } else {
            value.trim()
        };
        if settings.player_name != name {
            settings.player_name = name.to_owned();
            if let Err(error) = settings.save() {
                ui.status = format!("Name konnte nicht gespeichert werden: {error}");
            }
        }
    }
}

#[expect(
    clippy::type_complexity,
    clippy::too_many_arguments,
    reason = "UI markers select which visibility rule applies to each node."
)]
pub fn update_ui(
    player: Res<Player>,
    view: Res<ViewState>,
    run: Res<CoinRun>,
    ui: Res<UiState>,
    settings: Res<Settings>,
    practice: Res<Practice>,
    mut labels: Query<(&Label, &mut Text)>,
    mut nodes: Query<(
        &mut Node,
        Has<SettingsPanel>,
        Has<Crosshair>,
        Has<HudPanel>,
        Has<InteractivePanel>,
    )>,
    window: Single<&Window>,
    mut scale: ResMut<UiScale>,
) {
    let desired_scale = (window.width() / 1280.0)
        .min(window.height() / 800.0)
        .clamp(0.65, 1.5);
    if scale.0 != desired_scale {
        scale.0 = desired_scale;
    }
    for (label, mut value) in &mut labels {
        let new = match label {
            Label::Title => if shooting::in_range(player.position) { "Zieltraining" } else { "Münzparcours" }.into(),
            Label::Subtitle => if shooting::in_range(player.position) { "Drei Bahnen. 30 Sekunden." } else { "Sechs Münzen. Ein Weg zurück." }.into(),
            Label::Crosshair => if practice.hit_flash > 0.0 { "×" } else { "+" }.into(),
            Label::Exercise => {
                if practice.running {
                    format!("{}/{} Treffer  ·  {:.0}% Genauigkeit\nRekord: {}{}",
                        practice.hits, practice.shots, practice.accuracy(), practice.best,
                        if !view.captured || !view.first_person || ui.menu || !window.focused { " · Pausiert" } else { "" })
                } else if shooting::in_range(player.position) {
                    format!("E startet 30 Sekunden Zieltraining\nLetzte Runde: {}  ·  Rekord: {}", practice.score, practice.best)
                } else { "Schießstand im Norden\nDer Button bringt dich direkt hin.".into() }
            }
            Label::Progress => {
                if shooting::in_range(player.position) {
                    format!("{:03} Punkte", practice.score)
                } else if run.finished {
                    "Parcours geschafft!".into()
                } else {
                    format!("{:02} / 06   Münzen", run.count())
                }
            }
            Label::Clock if shooting::in_range(player.position) => format!("{:02.0}s  ·  {}", practice.remaining.ceil(), settings.player_name),
            Label::Clock => format!(
                "{:02}:{:02}  ·  {}",
                run.elapsed as u32 / 60,
                run.elapsed as u32 % 60,
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
                    "WASD bewegen · Maus umsehen\nLinksklick schießen · E Zielübung\nUmschalt sprinten · Esc Maus freigeben"
                } else {
                    "Klicke ins Level, um weiterzuspielen.\nV Übersicht · M Einstellungen"
                }
            } else {
                "Pfeiltasten bewegen · Boden anklicken\nV Egoperspektive · Umschalt sprinten"
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
    for (mut node, menu, crosshair, hud, interactive) in &mut nodes {
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
        if hud {
            node.display = if ui.menu || (interactive && view.first_person && view.captured) {
                Display::None
            } else {
                Display::Flex
            };
        }
    }
}

/// Hit test the actual UI layout, including display scale and window resizing.
pub fn pointer_over_ui(
    window: &Window,
    blocks: &Query<(&ComputedNode, &UiGlobalTransform), With<UiBlock>>,
) -> bool {
    let Some(pointer) = window.cursor_position() else {
        return false;
    };
    let pointer = pointer * window.scale_factor();
    blocks.iter().any(|(node, transform)| {
        node.size().min_element() > 0.0
            && transform.try_inverse().is_some_and(|inverse| {
                let local = inverse.transform_point2(pointer);
                (local.abs().cmple(node.size() * 0.5)).all()
            })
    })
}
