use gpui_kit::{
    AnyElement, App, FontWeight, Hsla, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};

use super::{
    super::{
        consts::{DOT, TILE_HEIGHT},
        enums::Selection,
        helpers::{groups, is_selected},
        structs::UsageDashboard,
    },
    gauge::gauge,
    key::key,
};
use crate::{
    focus::PressStop,
    scale::px,
    theme::{Theme, radius},
};

/// What a tile says, apart from how it looks.
struct Tile {
    id: String,
    dot: Hsla,
    name: SharedString,
    caption: SharedString,
    value: SharedString,
    note: SharedString,
    used: Option<f32>,
    selected: bool,
    pick: Selection,
}

/// The row of tiles: all accounts first, then each provider's under its caption.
pub(super) fn tiles(d: &UsageDashboard, theme: &Theme, window: &mut Window, cx: &mut App) -> AnyElement {
    let (value, note) = d.summary.clone().unwrap_or_default();
    let all = Tile {
        id: "all".into(),
        dot: theme.foreground,
        name: "All accounts".into(),
        caption: format!("{} sources", d.sources.len()).into(),
        value,
        note,
        used: None,
        selected: d.selection == Selection::All,
        pick: Selection::All,
    };
    let mut row = div().flex().items_end().gap(px(10.)).child(column(None, 1, vec![tile(d, all, theme, window, cx)]));
    for group in groups(&d.sources) {
        let members: Vec<AnyElement> = d.sources[group.first..group.first + group.len]
            .iter()
            .map(|source| {
                let item = Tile {
                    id: format!("source-{}", source.id),
                    dot: theme.series(source.series.hue, source.series.shade),
                    name: source.name.clone(),
                    caption: source.caption.clone(),
                    value: source.value.clone(),
                    note: source.note.clone(),
                    used: source.limit,
                    selected: is_selected(&d.selection, source),
                    pick: Selection::Source(source.id.clone()),
                };
                tile(d, item, theme, window, cx)
            })
            .collect();
        let caption = caption(d, &group.caption, theme, window, cx);
        row = row.child(column(Some(caption), group.len, members));
    }
    row.into_any_element()
}

/// A group of tiles under its caption, as wide as it has tiles.
fn column(caption: Option<AnyElement>, tiles: usize, members: Vec<AnyElement>) -> AnyElement {
    let mut col = div().flex().flex_col().gap(px(6.)).min_w_0();
    col.style().flex_grow = Some(tiles as f32);
    col.style().flex_basis = Some(gpui_kit::px(0.).into());
    col.child(caption.unwrap_or_else(|| div().h(px(16.)).into_any_element()))
        .child(div().flex().gap(px(10.)).children(members))
        .into_any_element()
}

/// The provider's caption over its tiles; a press picks the whole group.
fn caption(d: &UsageDashboard, text: &SharedString, theme: &Theme, window: &mut Window, cx: &mut App) -> AnyElement {
    let handler = d.on_select.clone();
    let pick = Selection::Group(text.clone());
    let selector = format!("usage-group-{text}");
    div()
        .id(key(&d.id, format!("group-{text}")))
        .debug_selector(move || selector.clone())
        .h(px(16.))
        .px(px(2.))
        .rounded(radius::md())
        .text_size(px(12.))
        .text_color(if d.selection == pick { theme.foreground } else { theme.muted_foreground })
        .truncate()
        .cursor_pointer()
        .press_stop(key(&d.id, format!("group-press-{text}")), radius::md(), window, cx)
        .on_click(move |_, window, cx| {
            if let Some(handler) = &handler {
                handler(pick.clone(), window, cx);
            }
        })
        .child(text.clone())
        .into_any_element()
}

/// One tile: a dot and the name, its caption, the big value with its note, and the gauge.
fn tile(d: &UsageDashboard, tile: Tile, theme: &Theme, window: &mut Window, cx: &mut App) -> AnyElement {
    let handler = d.on_select.clone();
    let pick = tile.pick.clone();
    let selector = format!("usage-tile-{}", tile.id);
    div()
        .id(key(&d.id, format!("tile-{}", tile.id)))
        .debug_selector(move || selector.clone())
        .flex_1()
        .min_w_0()
        .h(px(TILE_HEIGHT))
        .p(px(12.))
        .flex()
        .flex_col()
        .rounded(radius::xl())
        .bg(if tile.selected { theme.card_strong } else { theme.card })
        .cursor_pointer()
        .press_stop(key(&d.id, format!("tile-press-{}", tile.id)), radius::xl(), window, cx)
        .on_click(move |_, window, cx| {
            if let Some(handler) = &handler {
                handler(pick.clone(), window, cx);
            }
        })
        .child(
            div()
                .flex_none()
                .flex()
                .items_center()
                .gap(px(7.))
                .text_size(px(13.))
                .font_weight(FontWeight::MEDIUM)
                .child(div().flex_none().size(px(DOT)).rounded_full().bg(tile.dot))
                .child(div().min_w_0().truncate().child(tile.name)),
        )
        .child(
            div()
                .flex_none()
                .h(px(16.))
                .mt(px(2.))
                .text_size(px(12.))
                .text_color(theme.muted_foreground)
                .truncate()
                .child(tile.caption),
        )
        .child(
            div()
                .flex_none()
                .flex()
                .items_baseline()
                .gap(px(4.))
                .mt(px(10.))
                .child(div().text_size(px(19.)).font_weight(FontWeight::MEDIUM).child(tile.value))
                .child(div().text_size(px(12.)).text_color(theme.muted_foreground).truncate().child(tile.note)),
        )
        .when_some(tile.used, |t, used| t.child(div().flex_none().mt(px(8.)).child(gauge(used, theme))))
        .into_any_element()
}
