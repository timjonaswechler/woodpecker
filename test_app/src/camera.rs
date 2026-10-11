use crate::{
    level::{LevelLoad, ground},
    player::Player,
    settings::Settings,
    shooting,
    ui::UiState,
};
use bevy::{
    input::mouse::AccumulatedMouseMotion,
    input_focus::InputFocus,
    prelude::*,
    text::EditableText,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

#[derive(Component)]
pub struct GameCamera;

#[derive(Resource, Default)]
pub struct ViewState {
    pub first_person: bool,
    pub captured: bool,
    pub pitch: f32,
}

impl ViewState {
    pub fn toggle(&mut self) {
        self.first_person = !self.first_person;
        self.captured = self.first_person;
        self.pitch = 0.0;
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Mouse look reads the input gates and changes the player's heading."
)]
pub fn mouse_look(
    mouse: Res<AccumulatedMouseMotion>,
    window: Single<&Window, With<PrimaryWindow>>,
    focus: Res<InputFocus>,
    inputs: Query<(), With<EditableText>>,
    load: Res<LevelLoad>,
    ui: Res<UiState>,
    settings: Res<Settings>,
    mut player: ResMut<Player>,
    mut view: ResMut<ViewState>,
) {
    if !load.is_ready()
        || ui.menu
        || !window.focused
        || !view.first_person
        || !view.captured
        || focus.get().is_some_and(|e| inputs.contains(e))
    {
        return;
    }
    player.yaw -= mouse.delta.x * 0.0025 * settings.sensitivity;
    view.pitch = (view.pitch
        - mouse.delta.y
            * 0.0025
            * settings.sensitivity
            * if settings.invert_y { -1.0 } else { 1.0 })
    .clamp(-1.45, 1.45);
}

pub fn sync_cursor(
    view: Res<ViewState>,
    ui: Res<UiState>,
    window: Single<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    let (window, mut cursor) = window.into_inner();
    let captured = view.first_person && view.captured && !ui.menu && window.focused;
    cursor.visible = !captured;
    cursor.grab_mode = if captured {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
}

pub fn update_camera(
    player: Res<Player>,
    view: Res<ViewState>,
    settings: Res<Settings>,
    mut camera: Single<(&mut Transform, &mut Projection), With<GameCamera>>,
) {
    let (ref mut camera_transform, ref mut projection) = *camera;
    if let Projection::Perspective(perspective) = &mut **projection {
        perspective.fov = if view.first_person {
            settings.fov
        } else {
            50.0
        }
        .to_radians();
    }
    if view.first_person {
        **camera_transform = Transform::from_translation(ground(player.position, 1.5))
            .with_rotation(Quat::from_euler(EulerRot::YXZ, player.yaw, view.pitch, 0.0));
    } else {
        // A fixed overview keeps world directions aligned with the arrow keys and the click target stable.
        **camera_transform = if shooting::in_range(player.position) {
            Transform::from_xyz(0.0, 21.0, -5.0).looking_at(Vec3::new(0.0, 0.0, -20.0), Vec3::Y)
        } else {
            Transform::from_xyz(0.0, 39.0, 24.0).looking_at(Vec3::new(0.0, 0.0, -5.0), Vec3::Y)
        };
    }
}
