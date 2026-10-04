use gpui_kit::{Hsla, rgb};

use crate::{atelier_mark::Tile, theme::contrast};

fn hex(value: u32) -> Hsla {
    rgb(value).into()
}

#[test]
fn the_tile_is_terracotta_lighter_at_the_top_and_darker_at_the_foot_with_a_white_a() {
    let (tile, accent, white) = (Tile::of(None), hex(0xE0875A), hex(0xFFFFFF));
    assert_eq!(tile.letter, white);
    assert!(contrast(tile.top, white) < contrast(accent, white), "lighter at the top");
    assert!(contrast(tile.bottom, white) > contrast(accent, white), "darker at the foot");
}

#[test]
fn an_accent_the_owner_gives_colours_the_tile() {
    let tile = Tile::of(Some(hex(0x4F8EF7)));
    assert_ne!(tile.top, Tile::of(None).top);
    assert_eq!(tile.letter, hex(0xFFFFFF));
}
