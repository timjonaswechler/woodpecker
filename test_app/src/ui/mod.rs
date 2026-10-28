pub(crate) mod messages;
use crate::level;
mod fonts;
mod hud;
mod settings_menu;
mod theme;
mod widgets;
pub use fonts::apply as apply_font;
pub(crate) use hud::update_ui;
pub(crate) use settings_menu::sync_text_fields;
use theme::*;
pub(crate) use widgets::buttons;
use widgets::*;

use crate::{
    app::settings::Settings,
    input::intents::{Action, Actions},
    level::mission::Mission,
    level::mission::inventory::CoinRun,
    level::mission::inventory::ItemKind,
    level::objects::shooting_range::ShootingRange,
    level::{GOLD, TEAL},
    player::Player,
    player::camera::ViewState,
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
    status: String,
}

impl UiState {
    pub fn status(&self) -> &str {
        &self.status
    }
    pub fn show(&mut self, message: String) {
        self.status = message;
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            status: "Lade die Signalstation …".into(),
        }
    }
}

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
    hud::spawn(&mut commands, &settings, &assets, &mut tracked);
    settings_menu::spawn(&mut commands, &settings);
}

pub(crate) fn install(app: &mut App) {
    use crate::app::schedule::Phase;
    app.init_resource::<UiState>()
        .add_systems(Startup, spawn_ui)
        .add_systems(
            Update,
            (apply_font, buttons, sync_text_fields)
                .chain()
                .in_set(Phase::Input),
        )
        .add_systems(Update, update_ui.in_set(Phase::Present));
}
