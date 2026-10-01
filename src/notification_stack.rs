//! beui's NotificationStack (`components/motion/notification-stack.tsx`): notifications as a small stack of cards that
//! opens into a list. Collapsed, the newest card is whole and the ones behind it peek out below it, each 8px lower
//! and 12px narrower on each side; a count badge and "Notifications" sit under them. Opened, by a pointer over it, focus,
//! or a tap, the cards spread into a list, one to a row 4px apart, and the badge's words roll to "View all ↗". The stack
//! grows upward from where it sat, so it covers what is above it and moves nothing.
//!
//! Motion: the cards move on 320ms of the out curve, the background's edge on 260ms of it, and the label rolls on
//! `SPRING_SWAP` (the old words leave up in 140ms). Under Reduce Motion it jumps. A card behind the first has no words
//! until the stack is open. What gpui cannot draw is left out: the blur on the label.
//!
//! The geometry is worked out here, from the cards' heights measured in a hidden column, so what is drawn is a plain
//! function of how far open the stack is ([`Geometry`]).
use std::time::Duration;

use gpui_kit::{
    Context, ElementId, EventEmitter, FocusHandle, Focusable, FontWeight, Hsla, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Task, Window, div, prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::{
    icon::{Icon, IconName},
    motion::{Channel, Curve, Spring, ease},
    placement::measure,
    theme::{ActiveTheme, Theme},
    typography::TextSize,
};

/// The stack is at most this wide (`max-w-[22rem]`).
const MAX_WIDTH: f32 = 352.;
/// `p-3` round the cards and the footer.
const PAD: f32 = 12.;
/// A card behind the first shows this much below it, and is this much narrower on each side, for each place back.
const PEEK: f32 = 8.;
const INSET: f32 = 12.;
/// The cards in the open list are this far apart (`gap-1`).
const GAP: f32 = 4.;
/// Under the cards, when collapsed (`pb-2`), and between the cards and the footer (`mt-2`).
const REST_PAD: f32 = 8.;
const FOOTER_GAP: f32 = 8.;
/// The footer's height (`min-h-9`), its badge (`size-7`), and its room to the sides (`px-1`).
const FOOTER: f32 = 36.;
const BADGE: f32 = 28.;
/// A card: `rounded-2xl border px-4`, holding `py-4` of words.
const CARD_RADIUS: f32 = 16.;
const CARD_PAD_X: f32 = 16.;
const CARD_PAD_Y: f32 = 16.;
/// The stack's own corner (`rounded-3xl`).
const STACK_RADIUS: f32 = 24.;
/// The time the cards move in, the background's edge, and the old label's exit.
const CARDS: f32 = 0.32;
const BACKGROUND: f32 = 0.26;
const LABEL_EXIT: f32 = 0.14;
/// The labels' line, and how far a rolling label travels.
const LABEL_LINE: f32 = 20.;
const ROLL: f32 = 0.9;
/// The pointer leaving one part of the stack for another is not leaving the stack: it collapses after this.
const LEAVE_GRACE: Duration = Duration::from_millis(40);

/// The tone of the small text at a card's right edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrailingTone {
    Muted,
    Warning,
    Danger,
    Success,
}

/// The small text at a card's right edge: an optional icon and words, such as a retry count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trailing {
    pub icon: Option<IconName>,
    pub text: SharedString,
    pub tone: TrailingTone,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotificationItem {
    pub id: SharedString,
    pub title: SharedString,
    pub description: Option<SharedString>,
    pub trailing: Option<Trailing>,
}

impl NotificationItem {
    pub fn new(id: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        Self { id: id.into(), title: title.into(), description: None, trailing: None }
    }

    pub fn description(mut self, words: impl Into<SharedString>) -> Self {
        self.description = Some(words.into());
        self
    }

    pub fn trailing(mut self, icon: Option<IconName>, text: impl Into<SharedString>, tone: TrailingTone) -> Self {
        self.trailing = Some(Trailing { icon, text: text.into(), tone });
        self
    }
}

pub enum NotificationEvent {
    /// The stack opened or shut.
    Expanded(bool),
    /// A press on the open stack, when it was given [`NotificationStack::view_all`].
    ViewAll,
}

/// Where everything is for a stack `progress` open, in pixels from the bottom edge of the stack (upward is positive).
/// Every number is worked out from the cards' heights alone.
#[derive(Clone, Debug, PartialEq)]
pub struct Geometry {
    heights: Vec<f32>,
}

impl Geometry {
    pub fn new(heights: Vec<f32>) -> Self {
        Self { heights }
    }

    /// The row every card shares when collapsed: the tallest.
    pub fn cell(&self) -> f32 {
        self.heights.iter().copied().fold(0., f32::max)
    }

    /// The cards' area when open: the cards and the gaps.
    pub fn list(&self) -> f32 {
        self.heights.iter().sum::<f32>() + GAP * self.heights.len().saturating_sub(1) as f32
    }

    /// The whole stack's height, collapsed (`0.`) and open (`1.`).
    pub fn height(&self, open: bool) -> f32 {
        let grid = if open { self.list() } else { self.cell() + REST_PAD };
        PAD + grid + FOOTER_GAP + FOOTER + PAD
    }

    /// The height the stack takes in the page, which the hidden first card decides: compact, with no peek.
    pub fn footprint(&self) -> f32 {
        PAD + self.heights.first().copied().unwrap_or(0.) + FOOTER_GAP + FOOTER + PAD
    }

    /// The top of card `i` above the bottom edge, in the collapsed state and in the open one.
    fn top_from_bottom(&self, i: usize, open: bool) -> f32 {
        let stack = self.height(open);
        let within = if open { self.heights[..i].iter().sum::<f32>() + GAP * i as f32 } else { PEEK * i as f32 };
        stack - PAD - within
    }

    /// The top edge of card `i` above the bottom edge, `progress` of the way from collapsed to open, and its height.
    pub fn card(&self, i: usize, progress: f32) -> (f32, f32) {
        let top = lerp(self.top_from_bottom(i, false), self.top_from_bottom(i, true), progress);
        let height = lerp(self.cell(), self.heights[i], progress);
        (top, height)
    }

    /// How far a card is cut in from each side, `progress` of the way open.
    pub fn inset(i: usize, progress: f32) -> f32 {
        lerp(INSET * i as f32, 0., progress)
    }

    /// The stack's height, `progress` of the way from collapsed to open.
    pub fn stack(&self, progress: f32) -> f32 {
        lerp(self.height(false), self.height(true), progress)
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub struct NotificationStack {
    id: ElementId,
    items: Vec<NotificationItem>,
    max_visible: usize,
    collapsed_label: SharedString,
    expanded_label: SharedString,
    empty_label: SharedString,
    view_all: bool,
    expanded: bool,
    /// How far the cards have opened, and the background, on their own curves.
    cards: Channel,
    background: Channel,
    /// The label's roll: a new label arrives from 0 to 1 while the old one leaves.
    roll: Channel,
    leaving: Channel,
    old_label: Option<SharedString>,
    heights: Vec<f32>,
    width: f32,
    focus: FocusHandle,
    has_focus: bool,
    /// Whether the pointer is over the stack's footprint, and over the part of it that has opened above.
    hovered: [bool; 2],
    leave: Option<Task<()>>,
}

impl EventEmitter<NotificationEvent> for NotificationStack {}

impl Focusable for NotificationStack {
    fn focus_handle(&self, _: &gpui_kit::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl NotificationStack {
    pub fn new(id: impl Into<ElementId>, items: Vec<NotificationItem>, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        Self {
            id: id.into(),
            items,
            max_visible: 3,
            collapsed_label: "Notifications".into(),
            expanded_label: "View all".into(),
            empty_label: "All caught up".into(),
            view_all: false,
            expanded: false,
            cards: Channel::new(0.),
            background: Channel::new(0.),
            roll: Channel::new(1.),
            leaving: Channel::new(1.),
            old_label: None,
            heights: Vec::new(),
            width: MAX_WIDTH,
            focus,
            has_focus: false,
            hovered: [false; 2],
            leave: None,
        }
    }

    pub fn max_visible(mut self, max: usize) -> Self {
        self.max_visible = max.max(1);
        self
    }

    pub fn labels(mut self, collapsed: impl Into<SharedString>, expanded: impl Into<SharedString>, empty: impl Into<SharedString>) -> Self {
        self.collapsed_label = collapsed.into();
        self.expanded_label = expanded.into();
        self.empty_label = empty.into();
        self
    }

    /// A press on the open stack sends [`NotificationEvent::ViewAll`] instead of shutting it.
    pub fn view_all(mut self, on: bool) -> Self {
        self.view_all = on;
        self
    }

    pub fn set_items(&mut self, items: Vec<NotificationItem>, cx: &mut Context<Self>) {
        self.items = items;
        cx.notify();
    }

    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    fn label(&self, expanded: bool) -> SharedString {
        if expanded { self.expanded_label.clone() } else { self.collapsed_label.clone() }
    }

    pub fn set_expanded(&mut self, expanded: bool, cx: &mut Context<Self>) {
        if self.expanded == expanded {
            return;
        }

        let was = self.expanded;
        self.expanded = expanded;
        let reduce = cx.reduce_motion();
        let target = if expanded { 1. } else { 0. };
        self.cards.animate(target, Curve::Ease(CARDS, ease::OUT), 0., reduce);
        self.background.animate(target, Curve::Ease(BACKGROUND, ease::OUT), 0., reduce);
        // The label rolls: the old words up and out, the new ones up from below.
        self.old_label = Some(self.label(was));
        self.leaving = Channel::new(0.);
        self.leaving.animate(1., Curve::Ease(LABEL_EXIT, ease::OUT), 0., reduce);
        self.roll = Channel::new(0.);
        self.roll.animate(1., Curve::Spring(Spring::SWAP), 0., reduce);
        cx.emit(NotificationEvent::Expanded(expanded));
        cx.notify();
    }

    fn pointer(&mut self, part: usize, over: bool, cx: &mut Context<Self>) {
        let was_over = self.hovered.iter().any(|h| *h);
        self.hovered[part] = over;
        if self.hovered.iter().any(|h| *h) {
            self.leave = None;
            self.set_expanded(true, cx);
        } else if was_over && !self.has_focus {
            // Passing from one part of the stack to the other is not leaving: wait a moment before shutting.
            let weak = cx.entity().downgrade();
            self.leave = Some(cx.spawn(async move |_, cx| {
                cx.background_executor().timer(LEAVE_GRACE).await;
                weak.update(cx, |s, cx| {
                    if !s.hovered.iter().any(|h| *h) && !s.has_focus {
                        s.set_expanded(false, cx);
                    }
                })
                .ok();
            }));
        }
    }

    /// A press: the closed stack opens (a finger never hovers); the open one goes to View all, or shuts.
    fn press(&mut self, cx: &mut Context<Self>) {
        if !self.expanded {
            self.set_expanded(true, cx);
        } else if self.view_all {
            cx.emit(NotificationEvent::ViewAll);
        } else {
            self.set_expanded(false, cx);
        }
    }

    fn key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "escape" if self.expanded => self.set_expanded(false, cx),
            "enter" | "space" => self.press(cx),
            _ => return,
        }
        cx.stop_propagation();
    }
}

fn trailing_color(theme: &Theme, tone: TrailingTone) -> Hsla {
    match tone {
        TrailingTone::Muted => theme.muted_foreground,
        TrailingTone::Warning => theme.warning,
        TrailingTone::Danger => theme.danger,
        TrailingTone::Success => theme.success,
    }
}

/// A card's words: the title, the small text at its right, and the description under them.
fn card_words(item: &NotificationItem, theme: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .py(px(CARD_PAD_Y))
        .child(
            div()
                .flex()
                .items_start()
                .justify_between()
                .gap(px(12.))
                .child(div().min_w_0().text_size(TextSize::Sm.font_size()).line_height(px(19.25)).font_weight(FontWeight::MEDIUM).text_color(theme.foreground).child(item.title.clone()))
                .children(item.trailing.clone().map(|t| {
                    let color = trailing_color(theme, t.tone);
                    div()
                        .flex_none()
                        .flex()
                        .items_center()
                        .gap(px(4.))
                        .text_size(TextSize::Xs.font_size())
                        .line_height(px(16.))
                        .text_color(color)
                        .children(t.icon.map(|icon| Icon::new(icon).size(px(14.)).color(color)))
                        .child(t.text)
                })),
        )
        .children(item.description.clone().map(|d| {
            div().text_size(TextSize::Xs.font_size()).line_height(px(19.5)).text_color(theme.muted_foreground).child(d)
        }))
}

impl Render for NotificationStack {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        // Focus that the keyboard brought opens the stack, and focus leaving shuts it (unless the pointer holds it open); a
        // window that starts with focus here, or a press, does not open it. Read each frame, so it does not depend on
        // an event that only an active window is given.
        let focused = self.focus.is_focused(window);
        if focused && !self.has_focus {
            self.has_focus = true;
            if window.last_input_was_keyboard() {
                self.set_expanded(true, cx);
            }
        } else if !focused && self.has_focus {
            self.has_focus = false;
            if !self.hovered.iter().any(|h| *h) {
                self.set_expanded(false, cx);
            }
        }
        let this = cx.entity().downgrade();
        let shown: Vec<NotificationItem> = self.items.iter().take(self.max_visible).cloned().collect();
        let Some(first) = shown.first().cloned() else {
            return div()
                .flex()
                .items_center()
                .justify_center()
                .gap(px(8.))
                .w_full()
                .max_w(px(MAX_WIDTH))
                .rounded(px(STACK_RADIUS))
                .bg(theme.card_strong.opacity(0.7))
                .px(px(20.))
                .py(px(32.))
                .text_size(TextSize::Sm.font_size())
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.muted_foreground)
                .child(Icon::new(IconName::NotificationsOff).size(px(16.)).color(theme.muted_foreground))
                .child(self.empty_label.clone())
                .debug_selector(|| "notification-empty".into())
                .into_any_element();
        };
        let moving = self.cards.is_running() || self.background.is_running() || self.roll.is_running() || self.leaving.is_running();
        if moving {
            window.request_animation_frame();
        }
        // Measured in a hidden column of the same width: each card's height, and the stack's width.
        if self.heights.len() != shown.len() {
            self.heights = vec![0.; shown.len()];
        }
        let geometry = Geometry::new(self.heights.clone());
        let inner = (self.width - 2. * PAD).max(0.);
        let (open_cards, open_bg) = (self.cards.value().clamp(0., 1.), self.background.value().clamp(0., 1.));
        let ready = self.heights.iter().all(|h| *h > 0.);
        let footprint = if ready { geometry.footprint() } else { 0. };
        let stack_height = if ready { geometry.stack(open_bg) } else { 0. };

        let card_face = |item: &NotificationItem| {
            div()
                .w(px(inner))
                .rounded(px(CARD_RADIUS))
                .border_1()
                .border_color(theme.foreground.opacity(0.06))
                .bg(theme.background)
                .px(px(CARD_PAD_X))
                .child(card_words(item, &theme))
        };
        // The hidden column: nothing to see, and the cards' heights come from it.
        let hidden = div().absolute().left(px(PAD)).bottom_0().opacity(0.).flex().flex_col().gap(px(GAP)).children(shown.iter().enumerate().map(|(i, item)| {
            let this = this.clone();
            div().relative().child(card_face(item)).child(measure(move |b, cx| {
                this.update(cx, |s, cx| {
                    let h = f32::from(b.size.height);
                    if s.heights.get(i).is_some_and(|old| (old - h).abs() > 0.4) {
                        s.heights[i] = h;
                        cx.notify();
                    }
                })
                .ok();
            }))
        }));

        // What is drawn: the stack's background, and the cards, last one first so the first is on top.
        let cards = (0..shown.len()).rev().filter(|_| ready).map(|i| {
            let (top, height) = geometry.card(i, open_cards);
            let cut = Geometry::inset(i, open_cards);
            let words_shown = i == 0 || self.expanded;
            div()
                .debug_selector(move || format!("notification-card-{i}"))
                .absolute()
                .left(px(PAD + cut))
                .w(px((inner - 2. * cut).max(0.)))
                .bottom(px(stack_height_from(&geometry, top, height)))
                .h(px(height))
                .rounded(px(CARD_RADIUS))
                .border_1()
                .border_color(theme.foreground.opacity(0.06))
                .bg(theme.background)
                .overflow_hidden()
                .child(div().w(px(inner)).ml(px(-cut)).px(px(CARD_PAD_X)).when(!words_shown, |d| d.opacity(0.)).child(card_words(&shown[i], &theme)))
        });
        let background = ready.then(|| {
            div().absolute().left_0().right_0().bottom_0().h(px(stack_height)).rounded(px(STACK_RADIUS)).bg(theme.card_strong)
        });

        // The footer: the count of all the notifications, and the label that rolls.
        let badge_fill = theme.warning_fill;
        // The words on the badge read at 4.5:1: the ink on a light fill, the page on a dark one.
        let badge_text = if crate::theme::contrast(theme.foreground, badge_fill) >= crate::theme::contrast(theme.background, badge_fill) { theme.foreground } else { theme.background };
        let roll = self.roll.value().clamp(0., 1.);
        let leaving = self.leaving.value().clamp(0., 1.);
        let current = self.label(self.expanded);
        let label_line = |words: SharedString, arrow: bool| {
            div().flex().items_center().gap(px(4.)).child(words).when(arrow, |d| d.child(Icon::new(IconName::ArrowOutward).size(px(16.))))
        };
        let footer = div()
            .absolute()
            .left(px(PAD + 4.))
            .right(px(PAD + 4.))
            .bottom(px(PAD))
            .h(px(FOOTER))
            .flex()
            .items_center()
            .gap(px(8.))
            .child(
                div()
                    .flex_none()
                    .size(px(BADGE))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .bg(badge_fill)
                    .text_color(badge_text)
                    .text_size(TextSize::Xs.font_size())
                    .font_weight(FontWeight::MEDIUM)
                    .child(SharedString::from(self.items.len().to_string())),
            )
            .child(
                div()
                    .relative()
                    .h(px(LABEL_LINE))
                    .min_w(px(96.))
                    .overflow_hidden()
                    .text_size(TextSize::Sm.font_size())
                    .line_height(px(LABEL_LINE))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground)
                    .children(self.old_label.clone().filter(|_| leaving < 1.).map(|old| {
                        div().absolute().top(px(-ROLL * LABEL_LINE * leaving)).opacity(1. - leaving).child(label_line(old.clone(), old == self.expanded_label))
                    }))
                    .child(div().absolute().top(px(ROLL * LABEL_LINE * (1. - roll))).opacity(roll.min(1.)).child(label_line(current.clone(), self.expanded))),
            );

        let (enter, leave_) = (this.clone(), this.clone());
        let (press, keys) = (this.clone(), this.clone());
        let focus = self.focus.clone();
        let width_probe = {
            let this = this.clone();
            measure(move |b, cx| {
                this.update(cx, |s, cx| {
                    let w = f32::from(b.size.width);
                    if (s.width - w).abs() > 0.4 {
                        s.width = w;
                        cx.notify();
                    }
                })
                .ok();
            })
        };
        let _ = (&leave_, &first);
        let above = this.clone();
        // The part of the open stack that rises above the footprint is a piece of the stack too: the pointer over it
        // keeps it open. It has no size when the stack is shut.
        let rise = ((stack_height - footprint).max(0.) * if self.expanded { 1. } else { 0. }).max(0.);
        let rise_zone = (rise > 0.5).then(|| {
            div()
                .absolute()
                .left_0()
                .right_0()
                .bottom(px(footprint))
                .h(px(rise))
                .id((self.id.clone(), "rise"))
                .on_hover(move |over, _, cx| {
                    above.update(cx, |s, cx| s.pointer(1, *over, cx)).ok();
                })
                .on_click(move |_, _, cx| {
                    press.update(cx, |s, cx| s.press(cx)).ok();
                })
        });
        div()
            .id(self.id.clone())
            .debug_selector(|| "notification-stack".into())
            .key_context("NotificationStack")
            .track_focus(&focus.tab_stop(true))
            .focus_visible(|s| s.shadow(crate::focus::ring_shadow(&theme, theme.background)))
            .relative()
            .w_full()
            .max_w(px(MAX_WIDTH))
            .h(px(footprint))
            .rounded(px(STACK_RADIUS))
            .cursor_pointer()
            .child(width_probe)
            .child(hidden)
            .children(background)
            .children(cards)
            .child(footer)
            .children(rise_zone)
            .on_hover(move |over, _, cx| {
                enter.update(cx, |s, cx| s.pointer(0, *over, cx)).ok();
            })
            .on_mouse_down(MouseButton::Left, |_, window, cx| {
                let _ = window;
                cx.stop_propagation();
            })
            .on_click(cx.listener(|this, _, _, cx| this.press(cx)))
            // A tap elsewhere shuts a stack a tap opened: a finger never leaves.
            .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                if this.expanded && !this.hovered.iter().any(|h| *h) && !this.has_focus {
                    this.set_expanded(false, cx);
                }
            }))
            .on_key_down(move |event, _, cx| {
                keys.update(cx, |s, cx| s.key(event, cx)).ok();
            })
            .into_any_element()
    }
}

/// A card's top edge above the stack's bottom, to the bottom edge of the card above it: how far up from the bottom to put
/// the card's own bottom.
fn stack_height_from(_: &Geometry, top: f32, height: f32) -> f32 {
    top - height
}

#[cfg(test)]
mod tests;
