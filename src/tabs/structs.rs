use std::rc::Rc;

use gpui_kit::{
    AnyElement, App, Bounds, ElementId, Entity, FocusHandle, FontWeight, InteractiveElement,
    IntoElement, ParentElement, Pixels, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, Window, div, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    focus::row_ring,
    motion::{Animated, FrameClock},
    placement::measure,
    theme::{ActiveTheme, mix, radius},
    tooltip::Tooltip,
    typography::FONT_FAMILY,
};
use super::types::{EDITOR_HEIGHT, GLIDE, Select, TabsVariant, UNDERLINE_HEIGHT};
use super::helpers::{covered, list_fill};

pub struct Tab {
    label: SharedString,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    tooltip: Option<SharedString>,
    group: Option<SharedString>,
    pub(super) pending: bool,
    selector: Option<String>,
}

impl Tab {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self { label: label.into(), leading: None, trailing: None, tooltip: None, group: None, pending: false, selector: None }
    }

    pub fn leading(mut self, element: impl IntoElement) -> Self {
        self.leading = Some(element.into_any_element());
        self
    }

    pub fn trailing(mut self, element: impl IntoElement) -> Self {
        self.trailing = Some(element.into_any_element());
        self
    }

    pub fn tooltip(mut self, words: impl Into<SharedString>) -> Self {
        self.tooltip = Some(words.into());
        self
    }

    /// A group name for the owner's `group_hover` on what it puts in the tab.
    pub fn group(mut self, name: impl Into<SharedString>) -> Self {
        self.group = Some(name.into());
        self
    }

    /// Not chosen and not lit: something still being read.
    pub fn pending(mut self, pending: bool) -> Self {
        self.pending = pending;
        self
    }

    pub fn debug_name(mut self, name: impl Into<String>) -> Self {
        self.selector = Some(name.into());
        self
    }
}

struct State {
    rects: Vec<Option<Bounds<Pixels>>>,
    pub(super) list: Option<Bounds<Pixels>>,
    pub(super) left: Animated,
    pub(super) width: Animated,
    seeded: bool,
    clock: FrameClock,
    focus: Vec<FocusHandle>,
}

#[derive(IntoElement)]
pub struct Tabs {
    id: ElementId,
    pub(super) variant: TabsVariant,
    pub(super) tabs: Vec<Tab>,
    pub(super) selected: Option<usize>,
    pub(super) on_select: Option<Select>,
}

impl Tabs {
    pub fn new(id: impl Into<ElementId>, variant: TabsVariant, tabs: impl IntoIterator<Item = Tab>, selected: Option<usize>) -> Self {
        Self { id: id.into(), variant, tabs: tabs.into_iter().collect(), selected, on_select: None }
    }

    pub fn on_select(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Tabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let variant = self.variant;
        let count = self.tabs.len();
        let state: Entity<State> = window.use_keyed_state(self.id.clone(), cx, |_, _| State {
            rects: Vec::new(),
            list: None,
            left: Animated::new(GLIDE, 0.),
            width: Animated::new(GLIDE, 0.),
            seeded: false,
            clock: FrameClock::default(),
            focus: Vec::new(),
        });
        let focus = state.update(cx, |s, cx| {
            s.rects.resize(count, None);
            while s.focus.len() < count {
                s.focus.push(cx.focus_handle());
            }
            s.focus[..count].to_vec()
        });
        let (glide, moving) = state.update(cx, |s, _| {
            let target = self.selected.and_then(|i| s.rects.get(i).copied().flatten()).zip(s.list).map(|(r, l)| {
                (f32::from(r.origin.x - l.origin.x), f32::from(r.size.width))
            });
            let mut moving = false;
            match target {
                Some((x, w)) => {
                    if !s.seeded {
                        s.left = Animated::new(GLIDE, x);
                        s.width = Animated::new(GLIDE, w);
                        s.seeded = true;
                    } else {
                        s.left.set_target(x);
                        s.width.set_target(w);
                    }
                }
                None => {
                    if self.selected.is_none() {
                        s.seeded = false;
                    } else {
                        // The tab is not measured yet: draw once more when it is.
                        moving = true;
                    }
                }
            }
            let dt = s.clock.tick();
            moving |= s.left.step(dt, reduce) | s.width.step(dt, reduce);
            if !moving {
                s.clock.rest();
            }
            (target.map(|_| (s.left.value(), s.width.value())), moving)
        });
        if moving {
            window.request_animation_frame();
        }

        let selected_pending = self.selected.is_some_and(|i| self.tabs.get(i).is_none_or(|t| t.pending));
        let (pad_x, pad_y) = variant.tab_pad();
        let keyboard = window.last_input_was_keyboard();
        let list_left = state.read(cx).list.map(|l| f32::from(l.origin.x)).unwrap_or(0.);
        let select = self.on_select.clone();
        let tabs: Vec<AnyElement> = self
            .tabs
            .into_iter()
            .enumerate()
            .map(|(i, tab)| {
                let chosen = self.selected == Some(i) && !tab.pending;
                let covering = match (glide, state.read(cx).rects.get(i).copied().flatten()) {
                    (Some((l, w)), Some(r)) => covered(f32::from(r.origin.x) - list_left, f32::from(r.size.width), l, w),
                    _ => 0.,
                };
                let ink = match variant {
                    v if v == TabsVariant::Underline || v.is_editor() => {
                        if chosen { theme.foreground } else { theme.muted_foreground }
                    }
                    _ => mix(theme.muted_foreground, theme.primary_foreground, covering),
                };
                let handle = focus[i].clone();
                let report = {
                    let state = state.clone();
                    move |b: Bounds<Pixels>, cx: &mut App| state.update(cx, |s, _| s.rects[i] = Some(b))
                };
                let pick = select.clone().filter(|_| !tab.pending);
                let focused = handle.is_focused(window) && keyboard;
                let hover_ink = if variant == TabsVariant::Underline || variant.is_editor() || covering < 0.5 { theme.foreground } else { ink };
                div()
                    .id(ElementId::NamedInteger("tab".into(), i as u64))
                    .relative()
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(px(6.))
                    .px(px(pad_x))
                    .py(px(pad_y))
                    .when(variant == TabsVariant::Underline, |d| d.min_h(px(UNDERLINE_HEIGHT)).pb(px(10.)).pt(px(4.)))
                    .when(variant.is_editor(), |d| d.h(px(EDITOR_HEIGHT)).text_size(px(14.)))
                    .when(variant == TabsVariant::Pill, |d| d.rounded_full())
                    .when(variant == TabsVariant::Segment, |d| d.rounded(radius::md()))
                    .when_some(tab.group, |d, name| d.group(name))
                    .font_family(FONT_FAMILY)
                    .font_weight(FontWeight::MEDIUM)
                    .text_size(px(14.))
                    .line_height(px(20.))
                    .whitespace_nowrap()
                    .text_color(ink)
                    .when(!tab.pending, |d| d.cursor_pointer().hover(move |s| s.text_color(hover_ink)).track_focus(&handle.tab_stop(true)))
                    .when_some(tab.tooltip, |d, words| d.tooltip(Tooltip::text(words)))
                    .when_some(tab.selector, |d, name| d.debug_selector(move || name.clone()))
                    .child(measure(report))
                    .children(tab.leading)
                    .child(div().relative().child(tab.label))
                    .children(tab.trailing)
                    .when(focused, |d| d.child(row_ring(&theme, theme.card, radius::md())))
                    .when_some(pick, |d, pick| {
                        let key_pick = pick.clone();
                        d.on_key_down(move |event, window, cx| {
                            if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                cx.stop_propagation();
                                key_pick(i, window, cx);
                            }
                        })
                        .on_click(move |_, window, cx| pick(i, window, cx))
                    })
                    .into_any_element()
            })
            .collect();

        let indicator = glide.filter(|_| self.selected.is_some() && !selected_pending).map(|(left, width)| match variant {
            TabsVariant::Underline => div().absolute().left(px(left)).bottom_0().w(px(width)).h(px(1.)).bg(theme.primary).debug_selector(|| "tabs-indicator".into()),
            TabsVariant::Pill => div().absolute().left(px(left)).top(px(variant.pad())).bottom(px(variant.pad())).w(px(width)).rounded_full().bg(theme.primary).debug_selector(|| "tabs-indicator".into()),
            TabsVariant::Chip => div().absolute().left(px(left)).top_0().h(px(EDITOR_HEIGHT)).w(px(width)).rounded(radius::md()).bg(theme.card_strong),
            TabsVariant::ChipLine => div()
                .absolute()
                .left(px(left))
                .top_0()
                .h(px(EDITOR_HEIGHT))
                .w(px(width))
                .rounded(radius::md())
                .bg(theme.card_strong)
                .child(div().absolute().left(px(10.)).right(px(10.)).bottom_0().h(px(2.)).rounded_full().bg(theme.foreground)),
            TabsVariant::Dot => div().absolute().top(px(EDITOR_HEIGHT + 3.)).left(px(left + width / 2. - 2.)).size(px(4.)).rounded_full().bg(theme.foreground),
            TabsVariant::Tick => div().absolute().top(px(7.)).left(px(left + 2.)).w(px(2.)).h(px(14.)).rounded_full().bg(theme.primary),
            TabsVariant::Segment => {
                div().absolute().left(px(left)).top(px(variant.pad())).bottom(px(variant.pad())).w(px(width)).rounded(radius::md()).bg(theme.primary).debug_selector(|| "tabs-indicator".into())
            }
        });
        let list_state = state.clone();
        div()
            .id(ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), "list".into()))
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(variant.gap()))
            .p(px(variant.pad()))
            .when(variant == TabsVariant::Dot, |d| d.pb(px(8.)))
            .overflow_x_scroll()
            .when(variant == TabsVariant::Pill, |d| d.rounded_full())
            .when(variant == TabsVariant::Segment, |d| d.rounded(radius::lg()))
            .when_some(list_fill(&theme, variant), |d, fill| d.bg(fill))
            .when(variant == TabsVariant::Underline, |d| {
                d.child(div().absolute().left_0().right_0().bottom_0().h(px(1.)).bg(crate::text_input::edge(&theme, theme.background)))
            })
            .child(measure(move |b, cx| list_state.update(cx, |s, _| s.list = Some(b))))
            .children(indicator)
            .children(tabs)
    }
}
