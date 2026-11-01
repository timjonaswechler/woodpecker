//! HUD construction and state-driven display.
use super::*;
pub(super) fn spawn(
    commands: &mut Commands,
    settings: &Settings,
    assets: &AssetServer,
    tracked: &mut crate::level::LevelAssets,
) {
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
                tracked.track(icon.clone().untyped());
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
                    ..EditableText::new(settings.player_name())
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
}
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct HudState<'w, 's> {
    gate: crate::input::policy::InputGate<'w, 's>,
    player: Res<'w, Player>,
    view: Res<'w, ViewState>,
    capture: Res<'w, crate::input::PointerCapture>,
    run: Res<'w, CoinRun>,
    ui: Res<'w, UiState>,
    flow: Res<'w, crate::app::flow::GameFlow>,
    settings: Res<'w, Settings>,
    shooting_range: Res<'w, ShootingRange>,
    mission: Res<'w, Mission>,
    nav: Res<'w, crate::level::navigation::Navigation>,
}

#[expect(
    clippy::type_complexity,
    reason = "The UI query selects panel roles without separate mutable Node queries."
)]
pub fn update_ui(
    hud: HudState,
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
    let player = &hud.player;
    let view = &hud.view;
    let capture = &hud.capture;
    let run = &hud.run;
    let ui = &hud.ui;
    let flow = &hud.flow;
    let settings = &hud.settings;
    let shooting_range = &hud.shooting_range;
    let mission = &hud.mission;
    let nav = &hud.nav;

    let viewport = hud_camera
        .logical_viewport_size()
        .unwrap_or(Vec2::new(window.width(), window.height()));
    let desired_scale = (viewport.x / 1440.0)
        .min(viewport.y / 900.0)
        .clamp(0.65, 1.5);
    if scale.0 != desired_scale {
        scale.0 = desired_scale;
    }
    let interaction = if hud.gate.allows(
        crate::input::policy::GameplayAction::Interact,
        view,
        capture,
    ) {
        mission.interaction(player, nav, view)
    } else {
        crate::level::mission::Interaction::None
    };
    for (label, mut value) in &mut labels {
        let new = match label {
            Label::Title => {
                if mission.finished() {
                    "Signal aktiv".into()
                } else {
                    crate::level::definition::room(player.position()).map_or_else(
                        || "Wartungsgang".into(),
                        |room| format!("{} · {}", room.id, room.name),
                    )
                }
            }
            Label::Resource(kind) => kind.count(run, mission).to_string(),
            Label::Objective => mission.objective().text().into(),
            Label::Interaction => interaction.prompt().into(),
            Label::Crosshair => {
                if shooting_range.hit_flash() > 0.0 { "×" } else { "+" }.into()
            }
            Label::Range => {
                if !level::objects::shooting_range::in_range(player.position()) {
                    String::new()
                } else if mission.practice_reward() {
                    "Bauplan-Truhe entriegelt · hinten rechts".into()
                } else {
                    let distance = level::objects::shooting_range::TARGETS
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| !shooting_range.targets_hit()[*i])
                        .map(|(_, point)| point.xz().distance(player.position()))
                        .fold(f32::INFINITY, f32::min);
                    format!(
                        "Mindestabstand {:.0} m · nächstes Ziel {distance:.1} m\n{} gültige Treffer · V für FPV",
                        level::objects::shooting_range::MIN_TARGET_DISTANCE,
                        shooting_range.hits(),
                    )
                }
            }
            Label::Progress => {
                if level::objects::shooting_range::in_range(player.position())
                    && player.height() < 0.2
                {
                    format!("{} Punkte", shooting_range.score())
                } else {
                    let floor = if player.height() > 2.8 { "oben" } else { "unten" };
                    format!(
                        "Ziel {:.0} m · Ebene {floor}",
                        player.feet().distance(mission.destination()),
                    )
                }
            }
            Label::Clock => format!(
                "{:02}:{:02}  ·  {}",
                mission.elapsed() as u32 / 60,
                mission.elapsed() as u32 % 60,
                settings.player_name()
            ),
            Label::Status => ui.status().to_owned(),
            Label::Mode => if view.first_person() {
                "02 / EGOPERSPEKTIVE"
            } else {
                "01 / ÜBERSICHT"
            }
            .into(),
            Label::Controls => if view.first_person() {
                if capture.active() {
                    "WASD + Maus · Leertaste springen\nUmschalt sprinten · E interagieren\nLinksklick schießen · Esc Maus freigeben"
                } else {
                    "Klicke ins Level, um weiterzuspielen.\nV Übersicht · M Einstellungen"
                }
            } else {
                "Pfeiltasten · Klickweg · V FPV · G Karte\nLeertaste springen · Umschalt sprinten\nE interagieren · Backspace Checkpoint"
            }
            .into(),
            Label::Sensitivity => format!("Maus-Empfindlichkeit    {:.1}", settings.sensitivity()),
            Label::Speed => format!("Lauftempo    {:.1} m/s", settings.speed()),
            Label::Fov => format!("FPV-Sichtfeld    {:.0}°", settings.fov()),
            Label::Path => format!(
                "Laufroute anzeigen: {}",
                if settings.show_path() { "An" } else { "Aus" }
            ),
            Label::Shadows => format!("Schatten: {}", if settings.shadows() { "An" } else { "Aus" }),
            Label::Invert => format!(
                "Maus Y-Achse umkehren: {}",
                if settings.invert_y() { "An" } else { "Aus" }
            ),
        };
        if value.0 != new {
            value.0 = new;
        }
    }
    for (mut node, menu, crosshair, hud, interactive, prompt) in &mut nodes {
        if menu {
            node.display = if flow.menu() {
                Display::Flex
            } else {
                Display::None
            };
        }
        if crosshair {
            node.display = if view.first_person() && capture.active() && !flow.menu() {
                Display::Flex
            } else {
                Display::None
            };
        }
        if prompt {
            node.top = if view.first_person() {
                percent(50)
            } else {
                Val::Auto
            };
            node.bottom = if view.first_person() {
                Val::Auto
            } else {
                px(110)
            };
        }
        if hud {
            node.display = if flow.menu()
                || (interactive && view.first_person() && capture.active())
                || (prompt && interaction == crate::level::mission::Interaction::None)
            {
                Display::None
            } else {
                Display::Flex
            };
        }
    }
}
