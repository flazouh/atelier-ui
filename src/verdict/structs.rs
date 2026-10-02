use gpui_kit::{
    AppContext,
    Context,
    Entity,
    EventEmitter,
    Focusable,
    InteractiveElement,
    IntoElement,
    ParentElement,
    Render,
    SharedString,
    StatefulInteractiveElement,
    Styled,
    Subscription,
    Window,
    component::input::{InputEvent, Textarea, TextareaState},
    div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    focus::Field,
    focus::PressStop,
    icon::IconName,
    rail_section::{RailSection, SectionTone},
    theme::{ActiveTheme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};
use super::types::{Decision, Verb, VerdictEvent};
use super::helpers::{about, enabled, needs_words, offered, placeholder, summary};

/// The Verdict box, as a rail section.
pub struct VerdictBox {
    pub(super) head_sha: SharedString,
    pub(super) mine: bool,
    pub(super) stated: Option<(Decision, bool)>,
    pub(super) text: Entity<TextareaState>,
    pub(super) writing: bool,
    pub(super) sending: Option<Verb>,
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

    pub(super) fn send(&mut self, verb: Verb, cx: &mut Context<Self>) {
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
