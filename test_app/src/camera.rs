use crate::{
    game::items::blueprint,
    game::level::{LevelLoad, ground},
    input::InputMode,
    player::Player,
    settings::Settings,
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

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ViewState {
    pub first_person: bool,
    pub captured: bool,
    pub pitch: f32,
    pub map: bool,
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
    mode: Res<InputMode>,
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
        || !mode.accepts_input(&window)
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
    mode: Res<InputMode>,
    view: Res<ViewState>,
    ui: Res<UiState>,
    window: Single<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    let (window, mut cursor) = window.into_inner();
    let captured = *mode == InputMode::Native
        && view.first_person
        && view.captured
        && !ui.menu
        && window.focused;
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
    time: Res<Time>,
    mut previous_fpv: Local<bool>,
    mut camera: Single<(&mut Transform, &mut Projection), With<GameCamera>>,
) {
    let (ref mut camera_transform, ref mut projection) = *camera;
    if view.first_person {
        **projection = Projection::Perspective(PerspectiveProjection {
            fov: settings.fov.to_radians(),
            near: 0.05,
            far: 200.,
            ..default()
        });
        **camera_transform =
            Transform::from_translation(ground(player.position, player.height + 1.5))
                .with_rotation(Quat::from_euler(EulerRot::YXZ, player.yaw, view.pitch, 0.));
    } else {
        let (target, extent) = if view.map {
            (Vec3::new(-2., 0., 1.), 100.)
        } else if let Some(room) = blueprint::room(player.position) {
            let [x0, y0, x1, y1] = room.bounds;
            (
                blueprint::point((x0 + x1) / 2. + 3., (y0 + y1) / 2., 1.),
                (x1 - x0).max(y1 - y0) * 0.95 + 14.,
            )
        } else {
            (ground(player.position, 1.), 32.)
        };
        let blend = 1. - (-8. * time.delta_secs()).exp();
        let extent = if let Projection::Orthographic(current) = &**projection
            && let bevy::camera::ScalingMode::FixedVertical { viewport_height } =
                current.scaling_mode
        {
            viewport_height + (extent - viewport_height) * blend
        } else {
            extent
        };
        **projection = Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: extent,
            },
            ..OrthographicProjection::default_3d()
        });
        let desired = Transform::from_translation(target + Vec3::new(12., 40., 32.))
            .looking_at(target, Vec3::Y);
        if *previous_fpv {
            **camera_transform = desired;
        } else {
            camera_transform.translation = camera_transform
                .translation
                .lerp(desired.translation, blend);
            camera_transform.rotation = camera_transform.rotation.slerp(desired.rotation, blend);
        }
    }
    *previous_fpv = view.first_person;
}
