use gpui_kit::{
    AnyElement, App, ElementId, FontWeight, Hsla, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder,
};
use std::rc::Rc;

use super::{
    super::{
        consts::DOT,
        enums::Selection,
        helpers::{groups, is_selected},
        structs::{UsageSource, UsageSources},
    },
    gauge::gauge,
    key::key,
};
use crate::{
    focus::PressStop,
    scale::px,
    theme::{ActiveTheme, Theme, radius},
};

impl UsageSources {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            sources: Vec::new(),
            summary: None,
            selection: Selection::default(),
            on_select: None,
        }
    }
    pub fn sources(mut self, sources: Vec<UsageSource>) -> Self {
        self.sources = sources;
        self
    }
    /// The value and note of the "All accounts" row: `5.84 M`, `today`.
    pub fn summary(
        mut self,
        value: impl Into<SharedString>,
        note: impl Into<SharedString>,
    ) -> Self {
        self.summary = Some((value.into(), note.into()));
        self
    }
    pub fn selection(mut self, selection: Selection) -> Self {
        self.selection = selection;
        self
    }
    pub fn on_select(mut self, f: impl Fn(Selection, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

/// What a row says, apart from how it looks.
struct Row {
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

impl RenderOnce for UsageSources {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let (value, note) = self.summary.clone().unwrap_or_default();
        let all = Row {
            id: "all".into(),
            dot: theme.foreground,
            name: "All accounts".into(),
            caption: format!("{} sources", self.sources.len()).into(),
            value,
            note,
            used: None,
            selected: self.selection == Selection::All,
            pick: Selection::All,
        };
        let mut column = div()
            .id(self.id.clone())
            .debug_selector(|| "usage-sources".into())
            .flex()
            .flex_col()
            .gap(px(1.))
            .w_full()
            .child(row(&self, all, &theme, window, cx));
        for group in groups(&self.sources) {
            column = column.child(heading(&self, &group.caption, &theme, window, cx));
            for source in &self.sources[group.first..group.first + group.len] {
                let item = Row {
                    id: format!("source-{}", source.id),
                    dot: theme.series(source.series.hue, source.series.shade),
                    name: source.name.clone(),
                    caption: source.caption.clone(),
                    value: source.value.clone(),
                    note: source.note.clone(),
                    used: source.limit,
                    selected: is_selected(&self.selection, source),
                    pick: Selection::Source(source.id.clone()),
                };
                column = column.child(row(&self, item, &theme, window, cx));
            }
        }
        column
    }
}

/// A provider's caption over its rows; a press picks the whole group.
fn heading(
    d: &UsageSources,
    text: &SharedString,
    theme: &Theme,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let handler = d.on_select.clone();
    let pick = Selection::Group(text.clone());
    let selector = format!("usage-group-{text}");
    div()
        .id(key(&d.id, format!("group-{text}")))
        .debug_selector(move || selector.clone())
        .mt(px(10.))
        .px(px(10.))
        .py(px(4.))
        .rounded(radius::md())
        .text_size(px(12.))
        .font_weight(FontWeight::MEDIUM)
        .text_color(if d.selection == pick {
            theme.foreground
        } else {
            theme.muted_foreground
        })
        .truncate()
        .cursor_pointer()
        .press_stop(
            key(&d.id, format!("group-press-{text}")),
            radius::md(),
            window,
            cx,
        )
        .on_click(move |_, window, cx| {
            if let Some(handler) = &handler {
                handler(pick.clone(), window, cx);
            }
        })
        .child(text.clone())
        .into_any_element()
}

/// One row: a dot, the name and its caption with the meter under them, and the value with its note at the right.
fn row(
    d: &UsageSources,
    item: Row,
    theme: &Theme,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let handler = d.on_select.clone();
    let pick = item.pick.clone();
    let selector = format!("usage-row-{}", item.id);
    div()
        .id(key(&d.id, format!("row-{}", item.id)))
        .debug_selector(move || selector.clone())
        .flex()
        .items_center()
        .gap(px(10.))
        .px(px(10.))
        .py(px(9.))
        .rounded(radius::lg())
        .when(item.selected, |r| r.bg(theme.card_strong))
        .cursor_pointer()
        .press_stop(
            key(&d.id, format!("row-press-{}", item.id)),
            radius::lg(),
            window,
            cx,
        )
        .on_click(move |_, window, cx| {
            if let Some(handler) = &handler {
                handler(pick.clone(), window, cx);
            }
        })
        .child(div().flex_none().size(px(DOT)).rounded_full().bg(item.dot))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .flex()
                        .items_baseline()
                        .gap(px(5.))
                        .text_size(px(14.))
                        .child(
                            div()
                                .font_weight(FontWeight::MEDIUM)
                                .truncate()
                                .child(item.name),
                        )
                        .child(
                            div()
                                .text_color(theme.muted_foreground)
                                .truncate()
                                .child(item.caption),
                        ),
                )
                .when_some(item.used, |c, used| {
                    c.child(div().mt(px(7.)).child(gauge(used, theme)))
                }),
        )
        .child(
            div()
                .flex_none()
                .flex()
                .flex_col()
                .items_end()
                .child(
                    div()
                        .text_size(px(13.))
                        .font_weight(FontWeight::MEDIUM)
                        .child(item.value),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(theme.muted_foreground)
                        .child(item.note),
                ),
        )
        .into_any_element()
}
