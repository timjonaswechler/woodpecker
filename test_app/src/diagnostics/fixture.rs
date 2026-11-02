//! Explicit, opt-in reset for sequential acceptance cases; never ordinary gameplay.
use crate::{
    app::{flow, settings::Settings},
    input::{
        PointerCapture,
        intents::{Action, Actions, FrameInput},
    },
};
use bevy::{input_focus::InputFocus, prelude::*, text::EditableText};

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct TestFixture {
    pub generation: u64,
    pub pid: u32,
    #[reflect(ignore)]
    seed: Settings,
}

pub(super) fn install(app: &mut App) {
    app.insert_resource(TestFixture {
        generation: 0,
        pid: std::process::id(),
        seed: app.world().resource::<Settings>().clone(),
    })
    .register_type::<TestFixture>()
    .add_systems(Last, reset);
}

fn reset(world: &mut World) {
    if !world
        .resource::<ButtonInput<KeyCode>>()
        .just_pressed(KeyCode::F9)
    {
        return;
    }
    world.resource_mut::<InputFocus>().clear();
    world.resource_mut::<Actions>().0.clear();
    world.resource_mut::<Actions>().0.push(Action::Restart);
    world
        .run_system_cached(flow::handle_actions)
        .expect("fixture gameplay reset");
    let seed = world.resource::<TestFixture>().seed.clone();
    for (mut text, name, notes) in world
        .query::<(
            &mut EditableText,
            Has<crate::ui::NameInput>,
            Has<crate::ui::NotesInput>,
        )>()
        .iter_mut(world)
    {
        if name || notes {
            text.pending_edits.clear();
            text.pending_paste = None;
            text.viewport = default();
            text.editor.set_text(if name {
                seed.player_name()
            } else {
                seed.notes()
            });
        }
    }
    world.insert_resource(seed);
    world.insert_resource(PointerCapture::default());
    world.insert_resource(FrameInput::default());
    woodpecker::session::Plugin::reset_inputs_after_tick(world);
    let mut fixture = world.resource_mut::<TestFixture>();
    fixture.generation = fixture
        .generation
        .checked_add(1)
        .expect("fixture generation exhausted");
}
