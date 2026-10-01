//! Tabs: beui.dev's Tabs (`components/motion/tabs.tsx`). A row of tabs with one indicator that glides from
//! the chosen tab to the next on `{ stiffness: 245, damping: 36, mass: 1.2 }`, which settles without
//! overshoot. Three looks:
//!
//! - Pill: `rounded-full bg-card p-1`, tabs with `px-3.5 py-1.5`, and a primary pill under the chosen one.
//! - Segment: `rounded-lg bg-card p-0.5`, no gap, and a `rounded-md` primary pill.
//! - Underline: tabs of at least 44px (`px-3 pb-2.5 pt-1`) over a 1px border, and a 1px primary line
//!   under the chosen one.
//!
//! The words are muted and go to the foreground on hover. On a pill they are the primary's text colour
//! by how much of the tab the pill covers, so the change follows the glide. Under Reduce Motion the
//! indicator jumps. A tab may carry something before its words and something after (a file's icon, a
//! close button); a pending tab (a file still being read) has no indicator and cannot be chosen.
//!
//! What gpui cannot draw is left out: the scroll arrows and edge fades of a list too long for its room
//! (the list scrolls), and the content's 4px rise on a change.
use std::rc::Rc;

use gpui_kit::{
    AnyElement, App, Bounds, ElementId, Entity, FocusHandle, FontWeight, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::{
    focus::row_ring,
    motion::{Animated, FrameClock, Spring},
    placement::measure,
    theme::{ActiveTheme, Theme, mix, radius},
    tooltip::Tooltip,
    typography::FONT_FAMILY,
};

/// The indicator's spring: `{ stiffness: 245, damping: 36, mass: 1.2 }`.
pub const GLIDE: Spring = Spring { stiffness: 245., damping: 36., mass: 1.2 };
/// An underline tab's least height (`min-h-[44px]`).
pub const UNDERLINE_HEIGHT: f32 = 44.;
/// An editor tab's height: atelier's control size.
pub const EDITOR_HEIGHT: f32 = 28.;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabsVariant {
    #[default]
    Pill,
    Underline,
    Segment,
    /// The editor strip's designs, in atelier's look: a 28px tab and no rule. The open tab's marker glides.
    /// A: a `card_strong` chip behind the open tab.
    Chip,
    /// B: the chip and a 2px line at its foot.
    ChipLine,
    /// C: a text tab with a 4px dot under it.
    Dot,
    /// D: a 2px accent tick at the tab's left.
    Tick,
}

impl TabsVariant {
    /// One of the editor strip's designs.
    pub fn is_editor(self) -> bool {
        matches!(self, TabsVariant::Chip | TabsVariant::ChipLine | TabsVariant::Dot | TabsVariant::Tick)
    }
    /// The list's padding.
    pub fn pad(self) -> f32 {
        match self {
            TabsVariant::Pill => 4.,
            TabsVariant::Segment => 2.,
            TabsVariant::Underline | TabsVariant::Chip | TabsVariant::ChipLine | TabsVariant::Dot | TabsVariant::Tick => 0.,
        }
    }

    /// The gap between tabs.
    pub fn gap(self) -> f32 {
        match self {
            TabsVariant::Segment => 0.,
            _ => 4.,
        }
    }

    /// A tab's padding across and down.
    pub fn tab_pad(self) -> (f32, f32) {
        match self {
            TabsVariant::Underline => (12., 0.),
            TabsVariant::Chip | TabsVariant::ChipLine | TabsVariant::Dot | TabsVariant::Tick => (10., 0.),
            _ => (14., 6.),
        }
    }
}

/// How much of a tab (`tab_left`, `tab_width`) the indicator (`left`, `width`) covers, 0 to 1.
pub fn covered(tab_left: f32, tab_width: f32, left: f32, width: f32) -> f32 {
    if tab_width <= 0. {
        return 0.;
    }
    let overlap = (tab_left + tab_width).min(left + width) - tab_left.max(left);
    (overlap / tab_width).clamp(0., 1.)
}

type Select = Rc<dyn Fn(usize, &mut Window, &mut App)>;

pub struct Tab {
    label: SharedString,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    tooltip: Option<SharedString>,
    group: Option<SharedString>,
    pending: bool,
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
    list: Option<Bounds<Pixels>>,
    left: Animated,
    width: Animated,
    seeded: bool,
    clock: FrameClock,
    focus: Vec<FocusHandle>,
}

#[derive(IntoElement)]
pub struct Tabs {
    id: ElementId,
    variant: TabsVariant,
    tabs: Vec<Tab>,
    selected: Option<usize>,
    on_select: Option<Select>,
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

fn list_fill(theme: &Theme, variant: TabsVariant) -> Option<Hsla> {
    (!matches!(variant, TabsVariant::Underline) && !variant.is_editor()).then_some(theme.card)
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

#[cfg(test)]
mod tests;
