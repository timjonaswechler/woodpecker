//! Focused rendered check for object interaction and persistent visual feedback.
use super::{aim_at, capture, press, release};
use crate::{
    camera::ViewState,
    course::Pickup,
    game::items::blueprint as map,
    player::Player,
    player::actions::expedition::{CellState, Expedition, Interaction},
    player::actions::navigation::Navigation,
    stations::{CompletionMark, UnreadPage},
    ui::{Action, Actions, InteractionPrompt, Label},
};
use bevy::prelude::*;

fn expect_prompt(world: &mut World, expected: Interaction) {
    assert_eq!(
        world.resource::<Expedition>().interaction(
            world.resource::<Player>(),
            world.resource::<Navigation>(),
            world.resource::<ViewState>()
        ),
        expected
    );
    let text = world
        .query::<(&Label, &Text)>()
        .iter(world)
        .find_map(|(label, text)| matches!(label, Label::Interaction).then_some(text.0.clone()))
        .unwrap();
    assert_eq!(text, expected.prompt());
    let node = world
        .query_filtered::<&Node, With<InteractionPrompt>>()
        .single(world)
        .unwrap();
    assert_eq!(
        node.display,
        if expected == Interaction::None {
            Display::None
        } else {
            Display::Flex
        }
    );
}

pub fn step(world: &mut World, stage: usize, frames: u32) -> bool {
    if frames < 12 {
        return false;
    }
    if frames == 12 {
        match stage {
            100 => capture(world, "station-scanner-ready"),
            103 => capture(world, "station-assembly"),
            105 => capture(world, "station-cell-ready"),
            109 => capture(world, "station-valve-ready"),
            _ => (),
        }
    }
    if frames < 20 {
        return false;
    }
    release(world);
    match stage {
        100 => {
            expect_prompt(world, Interaction::Scan(0));
            press(world, KeyCode::KeyE);
        }
        101 => {
            assert!(world.resource::<Expedition>().scans[0]);
            expect_prompt(world, Interaction::None);
            for (page, visible) in world.query::<(&UnreadPage, &Visibility)>().iter(world) {
                if page.0 == 0 {
                    assert_eq!(*visible, Visibility::Hidden);
                }
            }
            for (mark, visible) in world.query::<(&CompletionMark, &Visibility)>().iter(world) {
                if mark.0 == Interaction::Scan(0) {
                    assert_ne!(*visible, Visibility::Hidden);
                }
            }
            capture(world, "station-scanner-used");
        }
        102 => {
            for action in [
                Interaction::Part(0),
                Interaction::Rack,
                Interaction::Part(1),
            ] {
                world.resource_mut::<Expedition>().apply(action);
            }
            world.resource_mut::<Player>().respawn(map::BENCH.xz(), 0.);
            aim_at(world, map::point(6.6, 33., 1.1));
        }
        103 => {
            expect_prompt(world, Interaction::Craft);
            press(world, KeyCode::KeyE);
        }
        104 => {
            assert_eq!(
                world.resource::<Expedition>().cells[0],
                CellState::Available
            );
            expect_prompt(world, Interaction::None);
            let (transform, visible) = world
                .query::<(&Pickup, &Transform, &Visibility)>()
                .iter(world)
                .find_map(|(pickup, transform, visible)| {
                    matches!(pickup, Pickup::Cell(0)).then_some((*transform, *visible))
                })
                .unwrap();
            assert_eq!(transform.translation, map::CELL_ITEMS[0]);
            assert_ne!(visible, Visibility::Hidden);
            world
                .resource_mut::<Player>()
                .respawn(map::REWARDS[0].xz(), 0.);
            aim_at(world, map::CELL_ITEMS[0]);
        }
        105 => {
            expect_prompt(world, Interaction::Cell(0));
            press(world, KeyCode::KeyE);
        }
        106 => {
            assert_eq!(world.resource::<Expedition>().carried_cells(), 1);
            expect_prompt(world, Interaction::None);
            for (pickup, visible) in world.query::<(&Pickup, &Visibility)>().iter(world) {
                if matches!(pickup, Pickup::Cell(0)) {
                    assert_eq!(*visible, Visibility::Hidden);
                }
            }
            capture(world, "station-cell-taken");
            press(world, KeyCode::KeyE);
        }
        107 => {
            assert_eq!(world.resource::<Expedition>().carried_cells(), 1);
            world.resource_mut::<Actions>().0.push(Action::Restart);
        }
        108 => {
            assert_eq!(world.resource::<Expedition>().cells, [CellState::Locked; 3]);
            for (_, visible) in world.query::<(&CompletionMark, &Visibility)>().iter(world) {
                assert_eq!(*visible, Visibility::Hidden);
            }
            for (_, visible) in world.query::<(&UnreadPage, &Visibility)>().iter(world) {
                assert_ne!(*visible, Visibility::Hidden);
            }
            world
                .resource_mut::<Player>()
                .respawn(map::VALVES[0].xz(), 0.);
            *world.resource_mut::<ViewState>() = ViewState {
                first_person: true,
                captured: true,
                ..default()
            };
            aim_at(world, map::point(64., 43., 1.12));
        }
        109 => {
            expect_prompt(world, Interaction::Valve(0));
            press(world, KeyCode::KeyE);
        }
        110 => {
            assert!(world.resource::<Expedition>().valves[0]);
            expect_prompt(world, Interaction::None);
            capture(world, "station-valve-used");
        }
        111 => {
            info!(
                "STATION SMOKE TEST PASSED: rear approach, focus prompt, distinct assembly and pickup, item disappears, completed scanner/valve and reset"
            );
            world.write_message(AppExit::Success);
            return false;
        }
        _ => panic!("unexpected station smoke stage {stage}"),
    }
    true
}
