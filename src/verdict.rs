//! What the reader thinks of the whole pull request, after GitQuiet's `Verdict.tsx` and
//! `docs/spec/verdict.md`.
//!
//! Folded, with Approve standing beside the fold: an approval needs no words and is the common answer.
//! Opening it shows the box and the three verbs in the merge tones: Approve in `success`, Request
//! changes in `danger`, Comment plain. It names the commit it is about, "About f4a97b1, the last commit
//! on this branch", and the verdict goes with that commit. Request changes and Comment need words;
//! Approve does not. There is no Approve on one's own pull request. When the owner reports a refusal,
//! the words stay in the box.

use gpui_kit::{
    Focusable,
    AppContext, Context, Entity, EventEmitter, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Subscription, Window,
    component::input::{InputEvent, Textarea, TextareaState},
    div, prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::{
    focus::PressStop,
    focus::Field,
    button::{Button, ButtonSize, ButtonVariant},
    icon::IconName,
    rail_section::{RailSection, SectionTone},
    theme::{ActiveTheme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};

/// The three verbs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    Approve,
    RequestChanges,
    Comment,
}

impl Verb {
    pub const ALL: [Verb; 3] = [Self::Approve, Self::RequestChanges, Self::Comment];

    pub fn word(self) -> &'static str {
        match self {
            Self::Approve => "Approve",
            Self::RequestChanges => "Request changes",
            Self::Comment => "Comment",
        }
    }

    /// What it says while it is on its way.
    pub fn working(self) -> &'static str {
        match self {
            Self::Approve => "Approving…",
            Self::RequestChanges => "Sending…",
            Self::Comment => "Posting…",
        }
    }

    fn wordless(self) -> bool {
        self == Self::Approve
    }
}

/// A verdict already given.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Approved,
    ChangesRequested,
    Commented,
    Dismissed,
}

/// The verbs offered: no Approve on one's own pull request.
pub fn offered(mine: bool) -> Vec<Verb> {
    Verb::ALL.into_iter().filter(|v| !(mine && *v == Verb::Approve)).collect()
}

/// Whether a verb can be pressed now: nothing while one is on its way, and words for the two that need
/// them.
pub fn enabled(verb: Verb, text: &str, sending: bool) -> bool {
    !sending && (verb.wordless() || !text.trim().is_empty())
}

/// The header's words: what the reader said, if anything, and whether it was about the last commit.
pub fn summary(stated: Option<(Decision, bool)>) -> SharedString {
    match stated {
        None => "not read yet by you".into(),
        Some((decision, current)) => {
            let said = match decision {
                Decision::Approved => "You approved this",
                Decision::ChangesRequested => "You asked for changes",
                Decision::Commented => "You commented on this",
                Decision::Dismissed => "Your review was dismissed",
            };
            if current { said.into() } else { format!("{said}, at an older commit").into() }
        }
    }
}

/// The commit, as the box names it.
pub fn about(head_sha: &str) -> SharedString {
    head_sha.chars().take(7).collect::<String>().into()
}

/// What is said when a verb that needs words is pressed on an empty box.
pub fn needs_words(verb: Verb) -> Option<&'static str> {
    match verb {
        Verb::Approve => None,
        Verb::RequestChanges => Some("Requesting changes needs words."),
        Verb::Comment => Some("A comment needs words."),
    }
}

/// Said once under the verbs while the box is empty.
pub fn hint(mine: bool) -> &'static str {
    if mine { "A comment needs words." } else { "An approval needs no words. The other two do." }
}

pub fn placeholder(mine: bool) -> &'static str {
    if mine { "Answer the review" } else { "Say what you found" }
}

/// What the box reports: a verdict to send, about `head_sha`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerdictEvent {
    Send { verb: Verb, note: SharedString, head_sha: SharedString },
}

/// The Verdict box, as a rail section.
pub struct VerdictBox {
    head_sha: SharedString,
    mine: bool,
    stated: Option<(Decision, bool)>,
    text: Entity<TextareaState>,
    writing: bool,
    sending: Option<Verb>,
    refused: Option<SharedString>,
    /// A verb was pressed on an empty box: say what it needs, until the reader types or puts the box away.
    wants_words: Option<Verb>,
    _subscription: Subscription,
}

impl EventEmitter<VerdictEvent> for VerdictBox {}

impl VerdictBox {
    /// A verdict about `head_sha`. `mine` when the reader wrote the pull request.
    pub fn new(head_sha: impl Into<SharedString>, mine: bool, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let text = cx.new(|cx| TextareaState::new(window, cx).auto_grow(2, 10).placeholder(placeholder(mine)));
        let subscription = cx.subscribe_in(&text, window, |_, _, event: &InputEvent, _, cx| {
            if let InputEvent::Change = event {
                cx.notify()
            }
        });
        Self { head_sha: head_sha.into(), mine, stated: None, text, writing: false, sending: None, refused: None, wants_words: None, _subscription: subscription }
    }

    /// The branch moved: the verdict now is about this commit.
    pub fn set_head(&mut self, head_sha: impl Into<SharedString>, cx: &mut Context<Self>) {
        let head_sha = head_sha.into();
        if head_sha != self.head_sha {
            self.head_sha = head_sha;
            cx.notify();
        }
    }

    /// Whether the pull request is the reader's own, which takes Approve away.
    pub fn set_mine(&mut self, mine: bool, cx: &mut Context<Self>) {
        if mine != self.mine {
            self.mine = mine;
            cx.notify();
        }
    }

    /// What the reader said before, and whether it was about the last commit.
    pub fn set_stated(&mut self, stated: Option<(Decision, bool)>, cx: &mut Context<Self>) {
        self.stated = stated;
        cx.notify();
    }

    /// The owner took the verdict: the words go and the box folds.
    pub fn sent(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let decided = match self.sending.take() {
            Some(Verb::Approve) => Decision::Approved,
            Some(Verb::RequestChanges) => Decision::ChangesRequested,
            _ => Decision::Commented,
        };
        self.stated = Some((decided, true));
        self.writing = false;
        self.refused = None;
        self.text.update(cx, |t, cx| t.set_value("", window, cx));
        cx.notify();
    }

    /// The owner could not send it: the words stay, and the reason shows under the box.
    pub fn refused(&mut self, reason: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.sending = None;
        self.refused = Some(reason.into());
        cx.notify();
    }

    fn send(&mut self, verb: Verb, cx: &mut Context<Self>) {
        let note: SharedString = self.text.read(cx).value().trim().to_string().into();
        if self.sending.is_none() && !enabled(verb, &note, false) {
            self.wants_words = Some(verb);
            cx.notify();
            return;
        }
        if !enabled(verb, &note, self.sending.is_some()) {
            return;
        }
        self.wants_words = None;
        self.sending = Some(verb);
        self.refused = None;
        cx.emit(VerdictEvent::Send { verb, note, head_sha: self.head_sha.clone() });
        cx.notify();
    }

    fn verb_button(&self, verb: Verb, _text: &str, cx: &mut Context<Self>) -> Button {
        let theme = cx.theme();
        let busy = self.sending == Some(verb);
        let words = if busy { verb.working() } else { verb.word() };
        // No colour on a button: the verbs carry the merge tones in their words and marks instead.
        let button = match verb {
            Verb::Approve => Button::new("verdict-approve").debug_name("verdict-approve").variant(ButtonVariant::Primary).icon(IconName::Check).icon_ink(theme.success),
            Verb::RequestChanges => Button::new("verdict-changes").debug_name("verdict-changes").variant(ButtonVariant::Secondary).ink(theme.danger),
            Verb::Comment => Button::new("verdict-comment").debug_name("verdict-comment").variant(ButtonVariant::Secondary),
        };
        button
            .label(words)
            .size(ButtonSize::Sm)
            .disabled(self.sending.is_some())
            .on_click(cx.listener(move |this, _, _, cx| this.send(verb, cx)))
    }
}

impl Render for VerdictBox {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let text = self.text.read(cx).value().to_string();
        let tone = match self.stated {
            Some((Decision::Approved, true)) => SectionTone::Done,
            Some((Decision::ChangesRequested, true)) => SectionTone::Bad,
            _ => SectionTone::Plain,
        };
        let about_line = div()
            .flex()
            .gap(px(4.))
            .px(px(12.))
            .pb(px(8.))
            .text_size(TextSize::Xs.font_size())
            .text_color(muted)
            .child("About")
            .child(
                div()
                    .flex()
                    .child(div().font_family(MONO_FONT_FAMILY).text_color(theme.foreground.opacity(0.9)).child(about(&self.head_sha)))
                    .child(", the last commit on this branch."),
            );

        let body = if self.writing {
            let verbs: Vec<Button> = offered(self.mine).into_iter().map(|v| self.verb_button(v, &text, cx)).collect();
            div()
                .flex()
                .flex_col()
                .gap(px(8.))
                .px(px(12.))
                .pb(px(12.))
                // Escape puts the box away, as Cancel does, unless the words are being sent.
                .on_key_down(cx.listener(|this, event: &gpui_kit::KeyDownEvent, _, cx| {
                    if event.keystroke.key == "escape" && this.sending.is_none() {
                        cx.stop_propagation();
                        this.writing = false;
                        this.wants_words = None;
                        cx.notify();
                    }
                }))
                .child(Field::new(self.text.focus_handle(cx), 
                    Textarea::new(&self.text).appearance(false).px(px(10.)).py(px(6.)).text_size(TextSize::Sm.font_size()).line_height(px(20.)),
                ).radius(radius::md()).surface(theme.background))
                .child(
                    div().flex().flex_wrap().items_center().gap(px(6.)).children(verbs).child(div().flex_1()).child(
                        Button::new("verdict-cancel")
                            .label("Cancel")
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .disabled(self.sending.is_some())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.writing = false;
                                this.wants_words = None;
                                cx.notify();
                            })),
                    ),
                )
                .when(text.trim().is_empty(), |d| {
                    d.children(self.wants_words.and_then(needs_words).map(|words| {
                        div().debug_selector(|| "verdict-words-error".into()).text_size(TextSize::Xs.font_size()).text_color(theme.danger).child(words)
                    }))
                })
                .into_any_element()
        } else {
            let label = if !text.trim().is_empty() { "Carry on with what you were writing" } else { placeholder(self.mine) };
            let text_state = self.text.clone();
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .px(px(12.))
                .pb(px(12.))
                .when(!self.mine, |d| d.child(self.verb_button(Verb::Approve, &text, cx)))
                .child(
                    div()
                        .id("verdict-open")
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .px(px(10.))
                        .py(px(6.))
                        .rounded(radius::md())
                        .bg(theme.card_strong)
                        .cursor_text()
                        .text_size(TextSize::Xs.font_size())
                        .text_color(muted)
                        .hover(|s| s.bg(theme.muted_hover()))
                        .press_stop("verdict-open-focus", crate::theme::radius::md(), window, cx)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.writing = true;
                            text_state.update(cx, |t, cx| t.focus(window, cx));
                            cx.notify();
                        }))
                        .child(label),
                )
                .into_any_element()
        };
        RailSection::new("Verdict")
            .icon(IconName::Visibility)
            .summary(summary(self.stated))
            .tone(tone)
            .child(about_line)
            .child(body)
            .when_some(self.refused.clone(), |d, reason| {
                d.child(div().px(px(12.)).pb(px(10.)).text_size(TextSize::Xs.font_size()).text_color(theme.danger).child(format!("That was not taken: {reason}")))
            })
    }
}

#[cfg(test)]
mod tests;
