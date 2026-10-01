//! The bar on top of a review, for the agent's turn and the pull request view alike.
//!
//! - Left: "7 changed", `+62 −12` in the diff colours, a thin progress line, and "3 of 7 reviewed". The line
//!   fills on [`Spring::LAYOUT`] when the count changes; once every file is reviewed it turns `success`,
//!   and the words read "All 7 reviewed".
//! - Right: the mark ("Mark file", "Seen"), Review mode, Previous and Next, each with its words and its key
//!   cap inside it (the cap read from the key table), and a menu (…) that holds Accept all and Reject all
//!   with their caps, and Put all back, each where the owner handles it. Next reads
//!   Done once every file is reviewed. It holds reading and moving only, as GitQuiet's top bar does:
//!   Accept file and Reject file act on one file, so they sit on its card
//!   ([`crate::review_file_header::ReviewFileHeader`]).
//!
//! A narrow pane takes things away one [`Step`] at a time, stopping at the first that fits: the progress
//! line, then the totals, then the navigation words. A button never shows its cap alone: without words
//! it shows its icon beside the cap, and its words move to its tooltip. The bar measures what each step needs off its own layout, so the fit rests
//! on real widths rather than guesses.
//!
//! A review with two scopes (one turn or the whole session) shows them as a switch before the summary:
//! the scope in force on a tone, the other with the switch's cap.
//!
//! The owner handles every action through [`ReviewHandlers`], from these buttons and from the keys alike.
//! The bar takes focus when pressed.

use std::sync::Arc;

use gpui_kit::{
    App, ElementId, FocusHandle, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    icon::IconName,
    keys::{self, Command},
    number::Digits,
    menu::{self, Entry, Menu, MenuItem, Origin},
    segmented::{Segment, Segmented},
    motion::{Channel, Curve, Spring},
    placement::measure,
    popover::{Align, Popover},
    review::{ReviewHandler, ReviewHandlers, ReviewProgress, caps},
    theme::ActiveTheme,
    tooltip::Tooltip,
    typography::{MONO_FONT_FAMILY, SEGMENT_GAP, TextSize},
};

/// The progress line's length.
const LINE_WIDTH: f32 = 56.;
/// Space between the bar's parts, and on each side of it.
const GAP: f32 = 8.;
const PADDING: f32 = 12.;
/// Layout rounds each box to whole pixels, so the parts can add up to one or two pixels less than the whole needs.
const FIT_SLACK: f32 = 2.;

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

/// The steps, widest first.
pub const STEPS: [Step; 8] = [
    Step { line: true, totals: true, nav_words: true, short_scopes: false, short_summary: false, scope_cap: true, count_only: false },
    Step { line: false, totals: true, nav_words: true, short_scopes: false, short_summary: false, scope_cap: true, count_only: false },
    Step { line: false, totals: false, nav_words: true, short_scopes: false, short_summary: false, scope_cap: true, count_only: false },
    Step { line: false, totals: false, nav_words: false, short_scopes: false, short_summary: false, scope_cap: true, count_only: false },
    Step { line: false, totals: false, nav_words: false, short_scopes: false, short_summary: true, scope_cap: true, count_only: false },
    Step { line: false, totals: false, nav_words: false, short_scopes: true, short_summary: true, scope_cap: true, count_only: false },
    Step { line: false, totals: false, nav_words: false, short_scopes: true, short_summary: true, scope_cap: false, count_only: false },
    Step { line: false, totals: false, nav_words: false, short_scopes: true, short_summary: true, scope_cap: false, count_only: true },
];

/// What a step needs, given what its parts measured (`sum`), the bar's `width`, the count's own width and the width
/// of the box that holds it. The sum is a guess that layout can beat by a pixel or two, or by more. If the count
/// does not fit its box, the step does not fit at this width whatever the sum says: it needs more than `width`.
pub fn need_at(sum: f32, width: f32, summary: f32, summary_box: f32) -> f32 {
    let need = sum + FIT_SLACK;
    if summary > summary_box + 0.5 { need.max(width + 1.) } else { need }
}

/// The step to show at `width`, given what each step was measured to need so far: the first whose need
/// fits, or the first not measured yet, which is tried next. The narrowest when nothing fits.
pub fn choose(width: f32, needed: &[Option<f32>]) -> usize {
    needed.iter().position(|need| need.is_none_or(|n| n <= width)).unwrap_or(needed.len().saturating_sub(1))
}

#[derive(IntoElement)]
pub struct ReviewBar {
    id: ElementId,
    progress: ReviewProgress,
    handlers: ReviewHandlers,
    mark_label: &'static str,
    review_mode: bool,
    reviewed_word: &'static str,
    next_primary: bool,
    scopes: Option<([(SharedString, SharedString); 2], usize)>,
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

struct BarState {
    focus: FocusHandle,
    fill: Channel,
    menu_open: bool,
    /// The ⋯ button's bounds in its last layout, where the menu hangs from.
    menu_anchor: Option<gpui_kit::Bounds<gpui_kit::Pixels>>,
    width: f32,
    /// What each step was measured to need, for the content it was measured with.
    needed: [Option<f32>; STEPS.len()],
    measured_for: String,
    step: usize,
    /// The summary's own width, before it shrinks, in the last frame.
    summary: f32,
    /// The width of the box that holds it, which clips it.
    summary_box: f32,
    /// The scope switch's width, in the last frame; nothing without one.
    scopes: f32,
}

/// A small button with its words and cap, or with its icon and cap when the bar is too narrow for the
/// words, which then move to its tooltip. It runs `handler` when pressed and sits disabled without one.
pub(crate) fn worded(
    button: Button,
    words: &'static str,
    icon: IconName,
    with_words: bool,
    handler: &Option<ReviewHandler>,
) -> impl IntoElement {
    let button = button.size(ButtonSize::Sm);
    let button = if with_words { button.label(words) } else { button.icon(icon) };
    let button = match handler.clone() {
        Some(f) => button.on_click(move |_, window, cx| f(window, cx)),
        None => button.disabled(true),
    };
    div()
        .id(ElementId::Name(format!("{words}-button").into()))
        .flex_none()
        .when(!with_words, |d| d.tooltip(Tooltip::text(words)))
        .child(button)
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

/// What the ⋯ menu holds, in order: each entry the owner handles, with its cap.
pub fn menu_entries(h: &ReviewHandlers) -> Vec<(&'static str, Option<&'static str>, ReviewHandler)> {
    [
        ("Accept all", Some(caps::ACCEPT_ALL), &h.on_accept_all),
        ("Reject all", Some(caps::REJECT_ALL), &h.on_reject_all),
        ("Put all back", None, &h.on_put_back),
    ]
    .into_iter()
    .filter_map(|(words, cap, f)| f.clone().map(|f| (words, cap, f)))
    .collect()
}

/// The ⋯ button and its menu.
fn menu(id: &ElementId, state: &gpui_kit::Entity<BarState>, h: &ReviewHandlers, open: bool, cx: &mut App) -> impl IntoElement {
    let child = |name: &'static str| ElementId::NamedChild(Arc::new(id.clone()), name.into());
    let (toggle, close) = (state.clone(), state.clone());
    let entries = menu_entries(h);
    div()
        .relative()
        .flex_none()
        .child(
            Button::new(child("more")).debug_name("review-bar-more").icon(IconName::MoreHoriz).variant(ButtonVariant::Ghost).size(ButtonSize::Icon).disabled(entries.is_empty()).on_click(
                move |_, _, cx| {
                    toggle.update(cx, |s, cx| {
                        s.menu_open = !s.menu_open;
                        cx.notify();
                    })
                },
            ),
        )
        .child(measure({
            let state = state.clone();
            move |bounds, cx| state.update(cx, |s, _| s.menu_anchor = Some(bounds))
        }))
        .when(!entries.is_empty(), |d| {
            let (anchor, focus) = { let s = state.read(cx); (s.menu_anchor, s.focus.clone()) };
            let rows = entries.len();
            let panel = Menu::new(child("menu"), entries.into_iter().map(|(words, cap, f)| {
                let item = MenuItem::new(words).on_select(move |window, cx| f(window, cx));
                Entry::from(match cap {
                    Some(cap) => item.cap(cap),
                    None => item,
                })
            }))
            .origin(Origin::TopRight)
            .on_dismiss({
                let pick = state.clone();
                move |_, cx| {
                    pick.update(cx, |s, cx| {
                        s.menu_open = false;
                        cx.notify();
                    })
                }
            });
            d.child(
                Popover::new(child("popover"))
                    .open(open)
                    .anchor(anchor)
                    .align(Align::End)
                    .gap(4.)
                    .height(menu::height(rows))
                    .return_focus(&focus)
                    .on_close(move |_, cx| {
                        close.update(cx, |s, cx| {
                            s.menu_open = false;
                            cx.notify();
                        })
                    })
                    .child(panel),
            )
        })
}

#[cfg(test)]
mod tests;
