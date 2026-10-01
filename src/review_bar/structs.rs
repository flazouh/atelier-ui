use std::sync::Arc;

use gpui_kit::{
    App, ElementId, FocusHandle, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Window, div, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    button::{Button, ButtonVariant},
    icon::IconName,
    keys::{self, Command},
    motion::{Channel, Curve, Spring},
    number::Digits,
    placement::measure,
    review::{ReviewHandlers, ReviewProgress, caps},
    segmented::{Segment, Segmented},
    theme::ActiveTheme,
    typography::{MONO_FONT_FAMILY, SEGMENT_GAP, TextSize},
};
use super::types::{GAP, LINE_WIDTH, PADDING, STEPS};
use super::helpers::{choose, menu, need_at, worded};

/// What the bar shows at one step of narrowing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    pub line: bool,
    pub totals: bool,
    pub nav_words: bool,
    /// The scope switch's short words ("Turn", "Session") instead of its long ones.
    pub short_scopes: bool,
    /// The count alone, as "1/254 seen": no "254 changed" before it.
    pub short_summary: bool,
    /// The key cap on the scope that is not in force. It goes last, when the bar has no room left for it.
    pub scope_cap: bool,
    /// The count without its word, as "1/254". The last thing to go before the count itself would clip.
    pub count_only: bool,
}

#[derive(IntoElement)]
pub struct ReviewBar {
    pub(super) id: ElementId,
    pub(super) progress: ReviewProgress,
    pub(super) handlers: ReviewHandlers,
    pub(super) mark_label: &'static str,
    pub(super) review_mode: bool,
    pub(super) reviewed_word: &'static str,
    pub(super) next_primary: bool,
    pub(super) scopes: Option<([(SharedString, SharedString); 2], usize)>,
}

impl ReviewBar {
    pub fn new(id: impl Into<ElementId>, progress: ReviewProgress, handlers: ReviewHandlers) -> Self {
        Self { id: id.into(), progress, handlers, mark_label: "Mark file", review_mode: false, reviewed_word: "reviewed", next_primary: false, scopes: None }
    }

    /// The word on the mark button (`x`), such as "Seen" in a pull request. It shows when the handlers
    /// have `on_mark`.
    pub fn mark_label(mut self, label: &'static str) -> Self {
        self.mark_label = label;
        self
    }

    /// The word the count uses, such as "seen": "3 of 7 seen".
    pub fn reviewed_word(mut self, word: &'static str) -> Self {
        self.reviewed_word = word;
        self
    }

    /// Next as the primary button, where nothing else on the screen is: a pull request, whose files are
    /// read rather than decided.
    pub fn next_primary(mut self, primary: bool) -> Self {
        self.next_primary = primary;
        self
    }

    /// The review's two scopes, each as its words and its short words for a narrow bar, and the one in
    /// force, as a switch; pressing the other one, or its cap, runs `on_switch_scope`.
    pub fn scopes(mut self, labels: [(SharedString, SharedString); 2], selected: usize) -> Self {
        self.scopes = Some((labels, selected.min(1)));
        self
    }

    /// Whether review mode is on, which turns its button's icon from open to close.
    pub fn review_mode(mut self, on: bool) -> Self {
        self.review_mode = on;
        self
    }
}

pub(super) struct BarState {
    pub(super) focus: FocusHandle,
    pub(super) fill: Channel,
    pub(super) menu_open: bool,
    /// The ⋯ button's bounds in its last layout, where the menu hangs from.
    pub(super) menu_anchor: Option<gpui_kit::Bounds<gpui_kit::Pixels>>,
    pub(super) width: f32,
    /// What each step was measured to need, for the content it was measured with.
    pub(super) needed: [Option<f32>; STEPS.len()],
    pub(super) measured_for: String,
    pub(super) step: usize,
    /// The summary's own width, before it shrinks, in the last frame.
    pub(super) summary: f32,
    /// The width of the box that holds it, which clips it.
    pub(super) summary_box: f32,
    /// The scope switch's width, in the last frame; nothing without one.
    pub(super) scopes: f32,
}

impl RenderOnce for ReviewBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let reduce = cx.reduce_motion();
        let p = self.progress;
        let h = self.handlers.clone();
        let fraction = p.fraction();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| BarState {
            focus: cx.focus_handle(),
            fill: Channel::new(fraction),
            menu_open: false,
            menu_anchor: None,
            width: f32::MAX,
            needed: [None; STEPS.len()],
            measured_for: String::new(),
            step: 0,
            summary: 0.,
            summary_box: 0.,
            scopes: 0.,
        });
        // Measures belong to the words on screen; new words are measured again.
        let content = format!("{p:?}{}{}{}{}{}{:?}", self.mark_label, self.reviewed_word, h.on_mark.is_some(), h.on_review_mode.is_some(), keys::profile(cx) as u8, self.scopes);
        state.update(cx, |s, _| {
            if (s.fill.target() - fraction).abs() > 1e-4 {
                s.fill.animate(fraction, Curve::Spring(Spring::LAYOUT), 0., reduce);
            }
            if s.measured_for != content {
                s.measured_for = content;
                s.needed = [None; STEPS.len()];
            }
        });
        let (focus, fill, menu_open, step_at) = {
            let s = state.read(cx);
            if s.fill.is_running() {
                window.request_animation_frame();
            }
            (s.focus.clone(), s.fill.value().clamp(0., 1.), s.menu_open, s.step)
        };
        let step = STEPS[step_at];
        let child = |name: &'static str| ElementId::NamedChild(Arc::new(self.id.clone()), name.into());
        let done = p.is_done();

        // After layout: the bar's width, the summary's own width, then the buttons', which settle what
        // this step needs and which step to show.
        let bar_measure = {
            let state = state.clone();
            measure(move |b, cx| state.update(cx, |s, _| s.width = f32::from(b.size.width)))
        };
        let summary_measure = {
            let state = state.clone();
            measure(move |b, cx| state.update(cx, |s, _| s.summary = f32::from(b.size.width)))
        };
        let box_measure = {
            let state = state.clone();
            measure(move |b, cx| state.update(cx, |s, _| s.summary_box = f32::from(b.size.width)))
        };
        let buttons_measure = {
            let state = state.clone();
            measure(move |b, cx| {
                state.update(cx, |s, cx| {
                    let scopes = if s.scopes > 0. { s.scopes + GAP } else { 0. };
                    let need = scopes + s.summary + GAP + f32::from(b.size.width) + 2. * PADDING;
                    s.needed[s.step] = Some(need_at(need, s.width, s.summary, s.summary_box));
                    let next = choose(s.width, &s.needed);
                    if next != s.step {
                        s.step = next;
                        cx.notify();
                    }
                })
            })
        };

        let count = |text: String, added: bool| {
            div().font_family(MONO_FONT_FAMILY).text_color(theme.diff_color(added)).child(Digits::new(child(if added { "added" } else { "removed" }), text, TextSize::Xs.font_size()))
        };
        // A row, so the inner part keeps its own width to be measured, and the outer one clips it.
        let summary = div().relative().flex().flex_1().min_w_0().overflow_hidden().debug_selector(|| "review-bar-summary-box".into()).child(box_measure).child(
            div()
                .relative()
                .flex()
                .flex_none()
                .w_auto()
                .items_center()
                .gap(px(SEGMENT_GAP))
                .whitespace_nowrap()
                .text_size(TextSize::Xs.font_size())
                .debug_selector(|| "review-bar-summary".into())
                .child(summary_measure)
                .when(!step.short_summary, |d| d.child(div().font_weight(FontWeight::MEDIUM).text_color(theme.foreground.opacity(0.9)).child(Digits::new(child("changed"), p.changed_text(), TextSize::Xs.font_size()))))
                .when(step.totals, |d| {
                    d.child(div().flex().gap(px(6.)).child(count(format!("+{}", p.added), true)).child(count(format!("\u{2212}{}", p.removed), false)))
                })
                .when(step.line, |d| {
                    d.child(
                        div().flex_none().w(px(LINE_WIDTH)).h(px(3.)).rounded_full().bg(theme.card_strong).child(
                            div().h_full().w(px(LINE_WIDTH * fill)).rounded_full().bg(if done { theme.success } else { theme.foreground.opacity(0.7) }),
                        ),
                    )
                })
                .child(div().text_color(if done { theme.success } else { muted }).child(Digits::new(child("reviewed"), if step.count_only { p.count_text() } else if step.short_summary { p.short_text(self.reviewed_word) } else { p.reviewed_text_as(self.reviewed_word) }, TextSize::Xs.font_size()))),
        );

        let scopes_measure = {
            let state = state.clone();
            measure(move |b, cx| state.update(cx, |s, _| s.scopes = f32::from(b.size.width)))
        };
        let (short, cap) = (step.short_scopes, step.scope_cap);
        let scopes = self.scopes.clone().map(|(labels, selected)| {
            let switch = h.on_switch_scope.clone();
            let has_switch = switch.is_some();
            let segments = labels.into_iter().map(|label| Segment::new(if short { label.1 } else { label.0 }));
            let track = Segmented::new(format!("{}-scopes", self.id), segments, selected)
                .debug_name("review-bar-scopes")
                .on_change(move |_, window, cx| {
                    if let Some(f) = &switch {
                        f(window, cx)
                    }
                });
            let track = if cap && has_switch { track.cap(caps::SWITCH_SCOPE) } else { track };
            div().relative().flex().flex_none().child(scopes_measure).child(track)
        });
        let nav = step.nav_words;
        let buttons = div()
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(GAP))
            .child(buttons_measure)
            .when(h.on_mark.is_some(), |d| {
                d.child(worded(Button::new(child("mark")).variant(ButtonVariant::Ghost).command(Command::MarkFile), self.mark_label, IconName::Check, nav, &h.on_mark))
            })
            .when(h.on_review_mode.is_some(), |d| {
                let icon = if self.review_mode { IconName::CloseFullscreen } else { IconName::OpenInFull };
                d.child(worded(Button::new(child("mode")).variant(ButtonVariant::Ghost).command(Command::ReviewMode), keys::word(Command::ReviewMode), icon, nav, &h.on_review_mode))
            })
            .child(worded(Button::new(child("previous")).variant(ButtonVariant::Ghost).command(Command::PreviousFile), "Previous", IconName::ArrowUp, nav, &h.on_previous))
            .child(worded(
                Button::new(child("next")).variant(if self.next_primary { ButtonVariant::Primary } else { ButtonVariant::Ghost }).command(Command::NextFile),
                p.next_label(),
                IconName::ArrowDown,
                nav,
                &h.on_next,
            ))
            .child(menu(&self.id, &state, &h, menu_open, cx));

        let press_focus = focus.clone();
        div()
            .id(self.id.clone())
            .key_context("ReviewBar")
            .track_focus(&focus)
            .on_mouse_down(gpui_kit::MouseButton::Left, move |_, window, cx| press_focus.focus(window, cx))
            .relative()
            .flex()
            .items_center()
            .gap(px(GAP))
            .h(px(44.))
            .px(px(PADDING))
            .min_w_0()
            .overflow_hidden()
            .child(bar_measure)
            .children(scopes)
            .child(summary)
            .child(buttons)
    }
}
