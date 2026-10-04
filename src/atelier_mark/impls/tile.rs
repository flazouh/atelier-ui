use gpui_kit::{Hsla, rgb};

use crate::theme::mix;

use super::super::{
    consts::{ACCENT, DARKEN, LETTER, LIGHTEN},
    structs::Tile,
};

fn hex(value: u32) -> Hsla {
    rgb(value).into()
}

impl Tile {
    /// The colours with `accent`, or terracotta when there is none.
    pub fn of(accent: Option<Hsla>) -> Self {
        let accent = accent.unwrap_or_else(|| hex(ACCENT));
        Self { top: mix(accent, hex(0xFFFFFF), LIGHTEN), bottom: mix(accent, hex(0x000000), DARKEN), letter: hex(LETTER) }
    }
}
