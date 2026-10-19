//! Exercises persisted multiline notes through the real text-input event handlers.
use super::*;
use crate::ui::NotesInput;

const PREFIX: &str = "Erste Zeile: Grüße ÄÖÜ";
const MIXED: &str = "Grüße aus der Signalstation · 世界 · 🙂\n\
中文：欢迎来到信号站，这是用于测试自动换行的笔记，我们继续探索世界。\n\
日本語：こんにちは世界 · 한국어: 안녕하세요\n\
Emoji: 🚀 👩‍🚀 👍🏽 · Ende: 世界🙂";

fn long_input() -> String {
    let line = "Grüße aus der Signalstation · 世界 · 🙂\n";
    let mut value = line.repeat(16 * 1024 / line.len());
    value.push_str(&"x".repeat(16 * 1024 - value.len()));
    value
}

fn keyboard(world: &mut World, key_code: KeyCode, logical_key: Key, text: Option<String>) {
    let window = world
        .query_filtered::<Entity, With<Window>>()
        .single(world)
        .unwrap();
    world.write_message(KeyboardInput {
        key_code,
        logical_key: logical_key.clone(),
        state: ButtonState::Pressed,
        text: text.map(Into::into),
        repeat: false,
        window,
    });
    world.write_message(KeyboardInput {
        key_code,
        logical_key,
        state: ButtonState::Released,
        text: None,
        repeat: false,
        window,
    });
}

pub(super) fn step(world: &mut World, frame: u32, position: Vec2) -> bool {
    let entity = world
        .query_filtered::<Entity, With<NotesInput>>()
        .single(world)
        .unwrap();
    if frame <= 28 {
        unicode_editing(world, entity, frame);
        return false;
    }
    let frame = frame - 28;
    match frame {
        1 => {
            world
                .resource_mut::<InputFocus>()
                .set(entity, FocusCause::Pressed);
            keyboard(
                world,
                KeyCode::KeyA,
                Key::Character(PREFIX.into()),
                Some(PREFIX.into()),
            );
        }
        5 => {
            assert_eq!(
                Settings::load().notes(),
                PREFIX,
                "notes autosave Unicode input"
            );
            keyboard(world, KeyCode::Enter, Key::Enter, None);
        }
        9 => {
            assert_eq!(
                world.resource::<InputFocus>().get(),
                Some(entity),
                "Enter keeps multiline focus"
            );
            assert_eq!(
                world.resource::<Settings>().notes(),
                format!("{PREFIX}\n"),
                "Enter inserts a newline"
            );
            let payload = long_input();
            assert_eq!(payload.len(), 16 * 1024);
            keyboard(
                world,
                KeyCode::KeyA,
                Key::Character(payload.clone().into()),
                Some(payload),
            );
            for key in [
                KeyCode::KeyW,
                KeyCode::Space,
                KeyCode::KeyR,
                KeyCode::KeyE,
                KeyCode::KeyV,
            ] {
                press(world, key);
            }
        }
        13 => {
            let expected = format!("{PREFIX}\n{}", long_input());
            assert_eq!(
                world.resource::<Settings>().notes(),
                expected,
                "16 KiB input appends without truncation"
            );
            let saved = Settings::load();
            assert_eq!(
                saved.notes(),
                expected,
                "long multiline notes survive disk round-trip"
            );
            assert_eq!(
                saved.player_name(),
                "Jäger ÄÖÜ",
                "notes do not replace the name"
            );
            assert_eq!(
                world.resource::<Player>().position(),
                position,
                "editing notes blocks movement"
            );
            assert_eq!(
                world.resource::<Player>().height(),
                0.0,
                "editing notes blocks jumping"
            );
            assert!(
                !world.resource::<ViewState>().first_person(),
                "editing notes blocks view shortcuts"
            );
            assert!(
                world.resource::<crate::app::flow::GameFlow>().menu(),
                "editing notes blocks restarting"
            );
            let input = world.get::<EditableText>(entity).unwrap();
            assert!(
                input.viewport.offset.y > 0.0,
                "long notes scroll to reveal the caret"
            );
            assert!(
                input.viewport.size.y < 180.0,
                "long notes keep a bounded visible area"
            );
            release(world);
            capture(world, "settings-notes");
            world.resource_mut::<Actions>().0.push(Action::Defaults);
        }
        17 => {
            let saved = Settings::load();
            assert_eq!(saved.sensitivity(), Settings::default().sensitivity());
            assert_eq!(
                saved.notes(),
                format!("{PREFIX}\n{}", long_input()),
                "resetting options preserves personal notes"
            );
            world
                .resource_mut::<InputFocus>()
                .set(entity, FocusCause::Pressed);
            keyboard(world, KeyCode::Escape, Key::Escape, None);
        }
        21 => {
            assert!(
                world.resource::<InputFocus>().get().is_none(),
                "Escape leaves the notes field"
            );
            world.resource_mut::<Actions>().0.push(Action::Settings);
        }
        25 => {
            assert!(world.resource::<crate::app::flow::GameFlow>().menu());
            assert_eq!(
                world
                    .get::<EditableText>(entity)
                    .unwrap()
                    .value()
                    .to_string(),
                Settings::load().notes(),
                "reopening settings keeps persisted notes"
            );
            world.resource_mut::<Player>().face_yaw(0.0);
            world
                .resource_mut::<Actions>()
                .0
                .extend([Action::Resume, Action::View]);
            return true;
        }
        _ => {}
    }
    false
}

fn unicode_editing(world: &mut World, entity: Entity, frame: u32) {
    match frame {
        1 => {
            world
                .resource_mut::<InputFocus>()
                .set(entity, FocusCause::Pressed);
            keyboard(
                world,
                KeyCode::KeyA,
                Key::Character(MIXED.into()),
                Some(MIXED.into()),
            );
        }
        5 => {
            assert_eq!(Settings::load().notes(), MIXED);
            let mut input = world.get_mut::<EditableText>(entity).unwrap();
            let layout = input.editor().try_layout().expect("text has been shaped");
            assert!(
                layout.lines().count() > MIXED.lines().count(),
                "long Chinese text wraps within the notes field"
            );
            let mut glyph_count = 0;
            for line in layout.lines() {
                for run in line.runs() {
                    for cluster in run.clusters() {
                        let text = &MIXED[cluster.text_range()];
                        if text.chars().all(char::is_whitespace) {
                            continue;
                        }
                        for glyph in cluster.glyphs() {
                            assert_ne!(glyph.id, 0, "missing glyph for {text:?}");
                            glyph_count += 1;
                        }
                    }
                }
            }
            assert!(glyph_count > 50, "the mixed text produces visible glyphs");
            input.queue_edit(TextEdit::ScrollByLines(-1000.0));
        }
        9 => {
            capture(world, "settings-notes-unicode");
            keyboard(world, KeyCode::ArrowLeft, Key::ArrowLeft, None);
        }
        13 => {
            let input = world.get::<EditableText>(entity).unwrap();
            assert_eq!(
                input.editor().raw_selection().focus().index(),
                MIXED.len() - '🙂'.len_utf8(),
                "Left moves across one whole emoji"
            );
            keyboard(world, KeyCode::Backspace, Key::Backspace, None);
        }
        17 => {
            assert_eq!(
                Settings::load().notes(),
                MIXED.strip_suffix("界🙂").unwrap().to_owned() + "🙂",
                "Backspace removes one CJK character without damaging the emoji"
            );
            keyboard(
                world,
                KeyCode::KeyA,
                Key::Character("界".into()),
                Some("界".into()),
            );
        }
        21 => {
            assert_eq!(Settings::load().notes(), MIXED, "CJK insertion round-trips");
            keyboard(world, KeyCode::ArrowRight, Key::ArrowRight, None);
        }
        25 => {
            let mut input = world.get_mut::<EditableText>(entity).unwrap();
            assert_eq!(input.editor().raw_selection().focus().index(), MIXED.len());
            input.queue_edit(TextEdit::SelectAll);
        }
        _ => {}
    }
}
