use gpui_kit::{Hsla, rgb};

use crate::theme::mix;

use super::super::{
    consts::{ACCENT, DARKEN, EDGE_ALPHA, INK, LIGHTEN, PAPER},
    enums::MarkLook,
    structs::Tile,
};

fn hex(value: u32) -> Hsla {
    rgb(value).into()
}

impl Tile {
    /// The colours of `look` with `accent`, or terracotta when there is none.
    pub fn of(look: MarkLook, accent: Option<Hsla>) -> Self {
        let accent = accent.unwrap_or_else(|| hex(ACCENT));
        let (ink, paper) = (hex(INK), hex(PAPER));
        match look {
            MarkLook::Halo => Self { top: ink, bottom: ink, ink: paper, edge: Some(paper.opacity(EDGE_ALPHA)), glow: Some(accent) },
            MarkLook::Terracotta => Self {
                top: mix(accent, hex(0xFFFFFF), LIGHTEN),
                bottom: mix(accent, hex(0x000000), DARKEN),
                ink,
                edge: None,
                glow: None,
            },
        }
    }
}
