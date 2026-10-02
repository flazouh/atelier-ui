use super::{CancelComment, SubmitComment};

use std::rc::Rc;

use gpui_kit::{
    App,
    AppContext,
    ClickEvent,
    Context,
    ElementId,
    Entity,
    EventEmitter,
    FocusHandle,
    Focusable,
    FontWeight,
    InteractiveElement,
    IntoElement,
    ParentElement,
    Render,
    RenderOnce,
    SharedString,
    Styled,
    Subscription,
    Window,
    component::input::{InputEvent, Textarea, TextareaState},
    div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    ClickHandler,
    agent_text::{AgentText, AgentTextStatus},
    button::{Button, ButtonSize, ButtonVariant},
    focus::Field,
    icon::IconName,
    kbd::Kbd,
    theme::{ActiveTheme, radius},
    typography::TextSize,
};
use super::types::{COMPOSER, LineComposerEvent, SEND_KEYS};
use super::helpers::{card, gap_frame};

/// One comment in a thread.
#[derive(Clone, Debug, PartialEq)]
pub struct Comment {
    pub author: SharedString,
    /// When, as the owner words it: "2m ago", "Yesterday".
    pub time: SharedString,
    /// Markdown.
    pub body: SharedString,
}

impl Comment {
    pub fn new(author: impl Into<SharedString>, time: impl Into<SharedString>, body: impl Into<SharedString>) -> Self {
        Self { author: author.into(), time: time.into(), body: body.into() }
    }

    /// The author's first letter, for the avatar.
    pub fn initial(&self) -> SharedString {
        self.author.chars().next().map(|c| c.to_uppercase().collect::<String>()).unwrap_or_default().into()
    }
}

#[derive(IntoElement)]
pub struct LineComment {
    id: ElementId,
    comments: Vec<Comment>,
    on_reply: Option<ClickHandler>,
    on_resolve: Option<ClickHandler>,
    resolved: bool,
}

impl LineComment {
    pub fn new(id: impl Into<ElementId>, comments: Vec<Comment>) -> Self {
        Self { id: id.into(), comments, on_reply: None, on_resolve: None, resolved: false }
    }

    /// The thread is answered: it says "Resolved" in place of its buttons, and its text steps back.
    pub fn resolved(mut self, resolved: bool) -> Self {
        self.resolved = resolved;
        self
    }

    pub fn on_reply(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_reply = Some(Rc::new(handler));
        self
    }

    pub fn on_resolve(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_resolve = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for LineComment {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let id = self.id.clone();
        let entry = |(i, comment): (usize, Comment)| {
            div()
                .flex()
                .gap(px(10.))
                .child(
                    div()
                        .flex()
                        .flex_none()
                        .size(px(22.))
                        .items_center()
                        .justify_center()
                        .rounded_full()
                        .bg(theme.card)
                        .text_size(px(11.))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(muted)
                        .child(comment.initial()),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w_0()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .h(px(22.))
                                .text_size(TextSize::Xs.font_size())
                                .child(div().font_weight(FontWeight::MEDIUM).text_color(theme.foreground.opacity(0.9)).child(comment.author))
                                .child(div().text_color(muted).child(comment.time)),
                        )
                        .child(AgentText::new(ElementId::NamedInteger(format!("{id}-body").into(), i as u64), comment.body).status(AgentTextStatus::Streaming)),
                )
        };
        let button = |name: &'static str, label: &'static str, icon: IconName, handler: Option<ClickHandler>| {
            let b = Button::new(ElementId::NamedChild(std::sync::Arc::new(id.clone()), name.into()))
                .icon(icon)
                .label(label)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm);
            match handler {
                Some(h) => b.on_click(move |e, w, cx| h(e, w, cx)),
                None => b.disabled(true),
            }
        };
        let actions = if self.resolved {
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .ml(px(32.))
                .text_size(TextSize::Xs.font_size())
                .text_color(muted)
                .child(crate::icon::Icon::new(IconName::Check).size(px(14.)).color(theme.success))
                .child("Resolved")
        } else {
            div()
                .flex()
                .gap(px(4.))
                .ml(px(24.))
                .child(button("reply", "Reply", IconName::Return, self.on_reply))
                .child(button("resolve", "Resolve", IconName::Check, self.on_resolve))
        };
        gap_frame().child(
            card(cx)
                .when(self.resolved, |d| d.opacity(0.7))
                .children(self.comments.into_iter().enumerate().map(entry))
                .child(actions),
        )
    }
}

/// The composer for a new comment on `row`.
pub struct LineComposer {
    pub(super) row: usize,
    /// `None`: a comment is sent. `Some(in_review)`: it may also be held for a review.
    pass: Option<bool>,
    pub(super) text: Entity<TextareaState>,
    _subscription: Subscription,
}

impl EventEmitter<LineComposerEvent> for LineComposer {}

impl Focusable for LineComposer {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.text.focus_handle(cx)
    }
}

impl LineComposer {
    pub fn new(row: usize, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let text = cx.new(|cx| TextareaState::new(window, cx).auto_grow(2, 8).placeholder("Leave a comment"));
        // Sending needs text, so Comment turns on and off with it.
        let subscription = cx.subscribe_in(&text, window, |_, _, event: &InputEvent, _, cx| {
            if let InputEvent::Change = event {
                cx.notify()
            }
        });
        Self { row, pass: None, text, _subscription: subscription }
    }

    /// A comment here can be held for a review, sent with a verdict later. `in_review` is whether a review is
    /// open: then every comment joins it ("Add to review"), and otherwise Comment sends at once and
    /// "Start a review" holds this one and opens it.
    pub fn review_pass(mut self, in_review: bool) -> Self {
        self.pass = Some(in_review);
        self
    }

    pub fn row(&self) -> usize {
        self.row
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // In a review every comment is held; that is what the primary key sends.
        self.send(self.pass == Some(true), window, cx);
    }

    fn send(&mut self, hold: bool, window: &mut Window, cx: &mut Context<Self>) {
        let text: SharedString = self.text.read(cx).value().trim().to_string().into();
        if text.is_empty() {
            return;
        }
        self.text.update(cx, |t, cx| t.set_value("", window, cx));
        cx.emit(if hold { LineComposerEvent::Hold { row: self.row, text } } else { LineComposerEvent::Submit { row: self.row, text } });
    }

    fn cancel(&mut self, cx: &mut Context<Self>) {
        cx.emit(LineComposerEvent::Cancel { row: self.row });
    }
}

impl Render for LineComposer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let empty = self.text.read(cx).value().trim().is_empty();
        let theme = cx.theme().clone();
        gap_frame().child(
            card(cx)
                .key_context(COMPOSER)
                .on_action(cx.listener(|this, _: &SubmitComment, window, cx| this.submit(window, cx)))
                .on_action(cx.listener(|this, _: &CancelComment, _, cx| this.cancel(cx)))
                .child(
                    Field::new(self.text.focus_handle(cx), 
                        Textarea::new(&self.text)
                            .appearance(false)
                            .px(px(8.))
                            .py(px(6.))
                            .text_size(TextSize::Sm.font_size())
                            .line_height(px(20.)),
                    ).radius(radius::lg()).surface(theme.background),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_end()
                        .gap(px(6.))
                        .child(
                            Button::new("comment-cancel")
                                .label("Cancel")
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Sm)
                                .on_click(cx.listener(|this, _, _, cx| this.cancel(cx))),
                        )
                        .when(self.pass == Some(false), |d| {
                            d.child(
                                Button::new("comment-hold")
                                    .label("Start a review")
                                    .variant(ButtonVariant::Secondary)
                                    .size(ButtonSize::Sm)
                                    .disabled(empty)
                                    .on_click(cx.listener(|this, _, window, cx| this.send(true, window, cx))),
                            )
                        })
                        .child(
                            Button::new("comment-send")
                                .label(if self.pass == Some(true) { "Add to review" } else { "Comment" })
                                .variant(ButtonVariant::Primary)
                                .size(ButtonSize::Sm)
                                .disabled(empty)
                                .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx))),
                        )
                        .child(Kbd::new(SEND_KEYS)),
                ),
        )
    }
}
