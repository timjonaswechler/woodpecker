//! Native/controlled devices and clicks into the game scene.
use super::*;
use crate::{
    level::{LevelLoad, navigation::Navigation},
    player::{Player, camera::GameCamera},
    ui::{UiBlock, UiState, pointer_over_ui},
};
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Native,
    Controlled(PointerId),
}

impl InputMode {
    pub fn accepts_input(self, window: &Window) -> bool {
        matches!(self, Self::Controlled(_)) || window.focused
    }
}

/// Reads the active device without changing the native window cursor.
#[derive(SystemParam)]
pub struct Cursor<'w, 's> {
    mode: Res<'w, InputMode>,
    pointers: Query<'w, 's, (&'static PointerId, &'static PointerLocation)>,
}

impl Cursor<'_, '_> {
    pub fn position(&self, window_entity: Entity, window: &Window) -> Option<Vec2> {
        match *self.mode {
            InputMode::Native => window.cursor_position(),
            InputMode::Controlled(id) => self.pointers.iter().find_map(|(pointer, location)| {
                if *pointer != id {
                    return None;
                }
                let location = location.location.as_ref()?;
                let position = location.position;
                (matches!(location.target, NormalizedRenderTarget::Window(target)
                    if target.entity() == window_entity)
                    && position.is_finite()
                    && position.cmpge(Vec2::ZERO).all()
                    && position
                        .cmplt(Vec2::new(window.width(), window.height()))
                        .all())
                .then_some(position)
            }),
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Pointer input resolves device, UI, camera and navigation state in one Bevy system."
)]
pub fn click_to_move(
    mode: Res<crate::input::InputMode>,
    cursor: crate::input::Cursor,
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<(Entity, &Window), With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<GameCamera>>,
    blocks: Query<(&ComputedNode, &UiGlobalTransform), With<UiBlock>>,
    nav: Res<Navigation>,
    mut player: ResMut<Player>,
    view: ResMut<ViewState>,
    mut capture: ResMut<crate::input::PointerCapture>,
    load: Res<LevelLoad>,
    mut ui: ResMut<UiState>,
    flow: Res<crate::app::flow::GameFlow>,
    mut focus: ResMut<InputFocus>,
) {
    let (window_entity, window) = window.into_inner();
    if !buttons.just_pressed(MouseButton::Left)
        || !super::policy::pointer_allowed(*mode, &flow, &load, window)
    {
        return;
    }
    let Some(cursor) = cursor.position(window_entity, window) else {
        return;
    };
    if pointer_over_ui(cursor, window.scale_factor(), &blocks) {
        return;
    }
    focus.clear();
    if view.first_person() {
        capture.set(true);
        return;
    }
    let Ok(ray) = camera.0.viewport_to_world(camera.1, cursor) else {
        return;
    };
    let outcome = player.follow_ray(&nav, ray);
    ui.show(crate::ui::messages::route(outcome));
}
