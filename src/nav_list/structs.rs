use std::{rc::Rc, sync::Arc};

use gpui_kit::{
    Animation, AnimationExt, App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};

use crate::{
    icon::{Icon, IconName},
    motion::{cubic_bezier, duration, ease},
    scale::px,
    theme::{ActiveTheme, radius},
    typography::TextSize,
};

/// A row's height, a head's height, the space at a row's sides, and the gap between two rows.
const ROW_HEIGHT: f32 = 30.;
const HEAD_HEIGHT: f32 = 28.;
const SIDE: f32 = 10.;
const GAP: f32 = 1.;
/// The space over a group that is not the first.
const GROUP_GAP: f32 = 6.;
/// The fold arrow's size and the gap after it: a row under a head starts where the head's name does.
const ARROW: f32 = 12.;
const ARROW_GAP: f32 = 6.;

type Pick = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// One place: the key the owner knows it by, its words, and a quiet note at its right end.
#[derive(Clone)]
pub struct NavRow {
    key: SharedString,
    label: SharedString,
    note: Option<SharedString>,
}

impl NavRow {
    pub fn new(key: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self { key: key.into(), label: label.into(), note: None }
    }
    /// A few quiet words at the row's right end, such as the group a search result comes from.
    pub fn note(mut self, note: impl Into<SharedString>) -> Self {
        self.note = Some(note.into());
        self
    }
}

/// A group of rows under a head that folds. With an empty name it has no head and its rows always show.
#[derive(Clone)]
pub struct NavGroup {
    name: SharedString,
    rows: Vec<NavRow>,
    open: bool,
}

impl NavGroup {
    pub fn new(name: impl Into<SharedString>, rows: impl IntoIterator<Item = NavRow>) -> Self {
        Self { name: name.into(), rows: rows.into_iter().collect(), open: false }
    }
    /// Rows with no head over them.
    pub fn flat(rows: impl IntoIterator<Item = NavRow>) -> Self {
        Self::new("", rows)
    }
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    /// Whether it has no row, as a search that found nothing.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    fn shows_rows(&self) -> bool {
        self.open || self.name.is_empty()
    }
}

#[derive(IntoElement)]
pub struct NavList {
    id: ElementId,
    groups: Vec<NavGroup>,
    selected: Option<SharedString>,
    on_pick: Option<Pick>,
    on_toggle: Option<Pick>,
}

impl NavList {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), groups: Vec::new(), selected: None, on_pick: None, on_toggle: None }
    }
    pub fn groups(mut self, groups: impl IntoIterator<Item = NavGroup>) -> Self {
        self.groups.extend(groups);
        self
    }
    /// The key of the chosen row.
    pub fn selected(mut self, key: impl Into<SharedString>) -> Self {
        self.selected = Some(key.into());
        self
    }
    /// Hears the key of a pressed row.
    pub fn on_pick(mut self, f: impl Fn(&SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_pick = Some(Rc::new(f));
        self
    }
    /// Hears the name of a group whose head was pressed. The owner folds or opens it.
    pub fn on_toggle(mut self, f: impl Fn(&SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for NavList {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let (muted, faint, hover) = (theme.muted_foreground, theme.faint(), theme.muted_hover());
        let child = |kind: &'static str, name: &SharedString| ElementId::NamedChild(Arc::new(self.id.clone()), format!("{kind}-{name}").into());
        let selected = self.selected;
        let groups = self.groups.into_iter().enumerate().map(|(at, group)| {
            let shows = group.shows_rows();
            let head = (!group.name.is_empty()).then(|| {
                let (name, toggle) = (group.name.clone(), self.on_toggle.clone());
                let selector = format!("nav-group-{}", group.name);
                div()
                    .id(child("group", &group.name))
                    .debug_selector(move || selector.clone())
                    .group("nav-head")
                    .flex()
                    .items_center()
                    .gap(px(ARROW_GAP))
                    .h(px(HEAD_HEIGHT))
                    .px(px(SIDE))
                    .rounded(radius::md())
                    .cursor_pointer()
                    .text_size(TextSize::Xs.font_size())
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(muted)
                    .hover(|s| s.text_color(theme.foreground))
                    .on_click(move |_, window, cx| {
                        if let Some(f) = &toggle {
                            f(&name, window, cx);
                        }
                    })
                    .child(
                        div()
                            .flex_none()
                            .text_color(faint)
                            .group_hover("nav-head", |s| s.text_color(theme.foreground))
                            .child(Icon::new(if group.open { IconName::ChevronDown } else { IconName::ChevronRight }).size(px(ARROW))),
                    )
                    .child(div().flex_1().min_w_0().truncate().child(group.name.clone()))
                    .child(div().flex_none().font_weight(FontWeight::NORMAL).text_color(faint).child(group.rows.len().to_string()))
            });
            let indent = if group.name.is_empty() { 0. } else { ARROW + ARROW_GAP };
            let rows = shows.then(|| {
                let rows = group.rows.into_iter().map(|row| {
                    let chosen = selected.as_ref() == Some(&row.key);
                    let (key, pick) = (row.key.clone(), self.on_pick.clone());
                    let selector = format!("nav-row-{}", row.key);
                    div()
                        .id(child("row", &row.key))
                        .debug_selector(move || selector.clone())
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .h(px(ROW_HEIGHT))
                        .pl(px(SIDE + indent))
                        .pr(px(SIDE))
                        .rounded(radius::md())
                        .cursor_pointer()
                        .text_size(TextSize::Sm.font_size())
                        .when(chosen, |d| d.bg(theme.card_strong).text_color(theme.foreground).font_weight(FontWeight::MEDIUM))
                        .when(!chosen, |d| d.text_color(muted).hover(move |s| s.bg(hover)))
                        .when(chosen, |d| d.child(div().absolute().debug_selector(|| "nav-row-chosen".into())))
                        .on_click(move |_, window, cx| {
                            if let Some(f) = &pick {
                                f(&key, window, cx);
                            }
                        })
                        .child(div().flex_1().min_w_0().truncate().child(row.label))
                        .children(row.note.map(|note| div().flex_none().text_size(TextSize::Xs.font_size()).font_weight(FontWeight::NORMAL).text_color(faint).child(note)))
                });
                div().flex().flex_col().gap(px(GAP)).children(rows)
            });
            let body_id = child("rows", &group.name);
            div()
                .flex()
                .flex_col()
                .gap(px(GAP))
                .when(at > 0 && head.is_some(), |d| d.mt(px(GROUP_GAP)))
                .children(head)
                .when_some(rows, |d, rows| {
                    // Only a group the reader can fold fades its rows in; a flat list changes with each letter typed.
                    if reduce || group.name.is_empty() {
                        d.child(rows)
                    } else {
                        d.child(rows.with_animation(body_id, Animation::new(duration::REVEAL).with_easing(|t| cubic_bezier(ease::OUT, t)), |rows, t| rows.opacity(t)))
                    }
                })
        });
        div().id(self.id.clone()).flex().flex_col().w_full().children(groups)
    }
}
