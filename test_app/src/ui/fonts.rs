//! Bundled fallbacks keep editable text independent of installed system fonts.
use crate::game::level::LevelAssets;
use bevy::{prelude::*, text::FontSource};

#[derive(Resource)]
pub struct UiFonts(FontSource);

pub fn load(assets: &AssetServer, tracked: &mut LevelAssets) -> UiFonts {
    // Unmodified SIL OFL fonts; the corresponding licenses live beside the files.
    // CJK: notofonts/noto-cjk@f8d157532fbfaeda587e826d4cd5b21a49186f7c,
    // Sans/OTF/SimplifiedChinese/NotoSansCJKsc-Regular.otf (also includes kana/Hangul).
    // Emoji: google/fonts@66928797f271fed5ee428544e7229685e2d90127,
    // ofl/notoemoji/NotoEmoji[wght].ttf, renamed locally without modifying its data.
    // The outline emoji font works with the same monochrome UI text renderer.
    let fonts = [
        "fonts/Barlow-Regular.ttf",
        "fonts/NotoSansCJKsc-Regular.otf",
        "fonts/NotoEmoji-Variable.ttf",
    ]
    .map(|path| assets.load::<Font>(path));
    tracked.0.extend(fonts.iter().cloned().map(Handle::untyped));
    UiFonts(FontSource::list(fonts))
}

pub fn apply(fonts: Res<UiFonts>, mut added: Query<&mut TextFont, Added<TextFont>>) {
    for mut font in &mut added {
        font.font = fonts.0.clone();
    }
}
