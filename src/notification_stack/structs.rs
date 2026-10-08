use gpui_kit::{
    Context, ElementId, EventEmitter, FocusHandle, Focusable, FontWeight, InteractiveElement,
    IntoElement, KeyDownEvent, MouseButton, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Task, Window, div, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    icon::{Icon, IconName},
    motion::{Channel, Curve, Spring, ease},
    placement::measure,
    theme::ActiveTheme,
    typography::TextSize,
};
use super::types::{
    BACKGROUND, BADGE, CARDS, CARD_PAD_X, CARD_RADIUS, FOOTER, FOOTER_GAP, GAP, INSET,
    LABEL_EXIT, LABEL_LINE, LEAVE_GRACE, MAX_WIDTH, NotificationEvent, PAD, PEEK, REST_PAD,
    ROLL, STACK_RADIUS, TrailingTone,
};
use super::helpers::{card_words, lerp, stack_height_from};

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

/// Where everything is for a stack `progress` open, in pixels from the bottom edge of the stack (upward is positive).
/// Every number is worked out from the cards' heights alone.
#[derive(Clone, Debug, PartialEq)]
pub struct Geometry {
    pub(super) heights: Vec<f32>,
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

pub struct NotificationStack {
    id: ElementId,
    pub(super) items: Vec<NotificationItem>,
    max_visible: usize,
    collapsed_label: SharedString,
    expanded_label: SharedString,
    empty_label: SharedString,
    pub(super) view_all: bool,
    pub(super) expanded: bool,
    /// How far the cards have opened, and the background, on their own curves.
    pub(super) cards: Channel,
    pub(super) background: Channel,
    /// The label's roll: a new label arrives from 0 to 1 while the old one leaves.
    pub(super) roll: Channel,
    pub(super) leaving: Channel,
    pub(super) old_label: Option<SharedString>,
    pub(super) heights: Vec<f32>,
    width: f32,
    pub(super) focus: FocusHandle,
    pub(super) has_focus: bool,
    /// Whether the pointer is over the stack's footprint, and over the part of it that has opened above.
    pub(super) hovered: [bool; 2],
    pub(super) leave: Option<Task<()>>,
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

    pub(super) fn label(&self, expanded: bool) -> SharedString {
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

    pub(super) fn pointer(&mut self, part: usize, over: bool, cx: &mut Context<Self>) {
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
    pub(super) fn press(&mut self, cx: &mut Context<Self>) {
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
