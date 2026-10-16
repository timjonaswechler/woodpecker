use crate::{
    input::InputMode,
    level::layout,
    level::{LevelLoad, ground},
    player::Player,
    settings::Settings,
    ui::UiState,
};
use bevy::{
    input::mouse::AccumulatedMouseMotion,
    input_focus::InputFocus,
    prelude::*,
    text::EditableText,
    transform::helper::TransformHelper,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

#[derive(Component)]
pub struct GameCamera;

/// Fixed eye point in Dummy_Head's local, unscaled glTF coordinates.
const EYE_POSITION: Vec3 = Vec3::new(0., 0.65, 0.);

#[derive(Component)]
pub(crate) struct EyeAnchor;

pub(super) fn spawn_eye(commands: &mut Commands, head: Entity) {
    commands.spawn((
        Name::new("Augenpunkt"),
        EyeAnchor,
        ChildOf(head),
        // The model faces +Z; cameras look along -Z.
        Transform::from_translation(EYE_POSITION)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
    ));
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Spielkamera"),
        GameCamera,
        Camera3d::default(),
        bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: 32.,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_translation(layout::START + Vec3::new(10., 32., 25.))
            .looking_at(layout::START, Vec3::Y),
        Msaa::Sample4,
    ));
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ViewState {
    pub first_person: bool,
    pub captured: bool,
    pub pitch: f32,
    pub map: bool,
    /// Current world-space view, shared by rendering, shots and interaction rays.
    pub eye: Option<Transform>,
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

#[expect(
    clippy::type_complexity,
    clippy::too_many_arguments,
    reason = "The head hierarchy must be read before mutating the camera transform."
)]
pub fn update_camera(
    mut commands: Commands,
    player: Res<Player>,
    mut view: ResMut<ViewState>,
    settings: Res<Settings>,
    time: Res<Time>,
    mut previous_fpv: Local<bool>,
    eyes: Query<Entity, With<EyeAnchor>>,
    mut transforms: ParamSet<(
        TransformHelper,
        Single<(Entity, &mut Transform, &mut Projection, Option<&ChildOf>), With<GameCamera>>,
    )>,
) {
    // GlobalTransform is not propagated until PostUpdate. Resolve the current
    // model/head pose now, so rays and the rendered camera use the same frame.
    let eye = eyes.single().ok().and_then(|entity| {
        transforms
            .p0()
            .compute_global_transform(entity)
            .ok()
            .map(|global| (entity, global))
    });
    let mut camera = transforms.p1();
    let (entity, ref mut camera_transform, ref mut projection, parent) = *camera;
    if view.first_person {
        **projection = Projection::Perspective(PerspectiveProjection {
            fov: settings.fov.to_radians(),
            near: 0.05,
            far: 200.,
            ..default()
        });
        view.eye = eye.map(|(anchor, global)| {
            if parent.map(ChildOf::parent) != Some(anchor) {
                commands.entity(entity).insert(ChildOf(anchor));
            }
            // Translation stays fixed at the head's eye point. Only mouse look
            // changes the local rotation; walking belongs entirely to the model.
            **camera_transform = Transform::from_rotation(Quat::from_rotation_x(view.pitch))
                .with_scale(global.scale().recip());
            global.mul_transform(**camera_transform).compute_transform()
        });
        if view.eye.is_none() {
            // The glTF hierarchy may still be loading. Never invent an eye
            // height or run a separate camera gait while waiting for its head.
            camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, player.yaw, view.pitch, 0.);
        }
    } else {
        view.eye = None;
        if parent.is_some() {
            commands.entity(entity).remove::<ChildOf>();
        }
        let (target, extent) = if view.map {
            (Vec3::new(-2., 0., 1.), 100.)
        } else if let Some(room) = layout::room(player.position) {
            let [x0, y0, x1, y1] = room.bounds;
            (
                layout::point((x0 + x1) / 2. + 3., (y0 + y1) / 2., 1.),
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

#[cfg(test)]
mod tests;
