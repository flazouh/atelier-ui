use gpui_kit::{Hsla, rgb};

use crate::{
    atelier_mark::{MarkLook, Tile},
    theme::{Appearance, contrast},
};

fn hex(value: u32) -> Hsla {
    rgb(value).into()
}

#[test]
fn the_dark_gets_the_halo_and_the_light_the_terracotta() {
    assert_eq!(MarkLook::of(Appearance::Dark), MarkLook::Halo);
    assert_eq!(MarkLook::of(Appearance::Light), MarkLook::Terracotta);
}

#[test]
fn the_halo_is_a_paper_a_on_ink_with_an_edge_and_a_glow_of_the_accent() {
    let tile = Tile::of(MarkLook::Halo, None);
    assert_eq!((tile.top, tile.bottom, tile.ink), (hex(0x141413), hex(0x141413), hex(0xF0EEE6)));
    assert_eq!(tile.glow, Some(hex(0xE0875A)), "terracotta unless told");
    assert!(tile.edge.is_some());
}

#[test]
fn the_terracotta_is_an_ink_a_on_the_accent_lighter_at_the_top_and_darker_at_the_foot() {
    let tile = Tile::of(MarkLook::Terracotta, None);
    let accent = hex(0xE0875A);
    assert_eq!(tile.ink, hex(0x141413));
    assert!(tile.glow.is_none() && tile.edge.is_none());
    assert!(contrast(tile.top, hex(0xFFFFFF)) < contrast(accent, hex(0xFFFFFF)), "lighter at the top");
    assert!(contrast(tile.bottom, hex(0xFFFFFF)) > contrast(accent, hex(0xFFFFFF)), "darker at the foot");
}

#[test]
fn an_accent_the_owner_gives_colours_both_looks() {
    let blue = hex(0x4F8EF7);
    assert_eq!(Tile::of(MarkLook::Halo, Some(blue)).glow, Some(blue));
    let tile = Tile::of(MarkLook::Terracotta, Some(blue));
    assert_ne!(tile.top, Tile::of(MarkLook::Terracotta, None).top);
}

/// The "A" reads on its tile in both looks, at the contrast of large text at least.
#[test]
fn the_a_reads_on_the_tile_in_both_looks() {
    for look in [MarkLook::Halo, MarkLook::Terracotta] {
        let tile = Tile::of(look, None);
        assert!(contrast(tile.ink, tile.top) >= 3. && contrast(tile.ink, tile.bottom) >= 3., "{look:?}");
    }
}
