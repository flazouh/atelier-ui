use super::{FoldComposer, MarkBold, MarkCode, MarkItalic, MarkLink, SendComment};

use gpui_kit::{
    App,
    AppContext,
    Context,
    Entity,
    EventEmitter,
    FocusHandle,
    Focusable,
    FontWeight,
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
    agent_text::AgentText,
    button::{Button, ButtonSize, ButtonVariant},
    focus::PressStop,
    icon::{Icon, IconName},
    markdown_edit::{Format, apply},
    theme::{ActiveTheme, radius},
    tooltip::Tooltip,
    typography::TextSize,
};
use super::types::{CommentComposerEvent, MOD, SEND};

pub struct CommentComposer {
    pub(super) title: SharedString,
    pub(super) author: SharedString,
    pub(super) text: Entity<TextareaState>,
    pub(super) open: bool,
    pub(super) preview: bool,
    pub(super) send_label: SharedString,
    pub(super) _subscription: Subscription,
}

impl EventEmitter<CommentComposerEvent> for CommentComposer {}

impl Focusable for CommentComposer {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.text.focus_handle(cx)
    }
}

impl CommentComposer {
    /// `title` heads the box ("On this pull request"); `author` signs it, as the comment will be.
    pub fn new(title: impl Into<SharedString>, author: impl Into<SharedString>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let text = cx.new(|cx| TextareaState::new(window, cx).auto_grow(3, 12).placeholder("Say something about the whole pull request"));
        let subscription = cx.subscribe_in(&text, window, |_, _, event: &InputEvent, _, cx| {
            if let InputEvent::Change = event {
                cx.notify()
            }
        });
        Self { title: title.into(), author: author.into(), text, open: false, preview: false, send_label: "Comment".into(), _subscription: subscription }
    }

    /// The words on the send button. The default is "Comment".
    pub fn send_label(mut self, label: impl Into<SharedString>) -> Self {
        self.send_label = label.into();
        self
    }
    /// Opens the box, as a press on its folded line does.
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open = true;
        self.preview = false;
        self.text.update(cx, |t, cx| t.focus(window, cx));
        cx.notify();
    }

    /// Fills the box, for a story or a draft kept from before.
    pub fn set_text(&mut self, text: impl Into<SharedString>, window: &mut Window, cx: &mut Context<Self>) {
        let text = text.into();
        self.text.update(cx, |t, cx| t.set_value(text, window, cx));
    }

    pub(super) fn words(&self, cx: &App) -> SharedString {
        self.text.read(cx).value()
    }

    pub(super) fn send(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text: SharedString = self.words(cx).trim().to_string().into();
        if text.is_empty() {
            return;
        }
        self.text.update(cx, |t, cx| t.set_value("", window, cx));
        self.open = false;
        cx.emit(CommentComposerEvent::Submit(text));
        cx.notify();
    }

    pub(super) fn fold(&mut self, cx: &mut Context<Self>) {
        self.open = false;
        cx.notify();
    }

    pub(super) fn mark(&mut self, format: Format, window: &mut Window, cx: &mut Context<Self>) {
        if self.preview {
            return;
        }
        let (value, chosen) = {
            let t = self.text.read(cx);
            (t.value().to_string(), t.selected_ranges().first().cloned().unwrap_or(t.cursor()..t.cursor()))
        };
        let (text, then) = apply(&value, chosen, format);
        self.text.update(cx, |t, cx| {
            t.set_value(text, window, cx);
            t.set_selected_range(then, cx);
            t.focus(window, cx);
        });
    }
}

impl Render for CommentComposer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let words = self.words(cx);
        let empty = words.trim().is_empty();
        let initial: String = self.author.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default();
        let head = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .h(px(32.))
            .text_size(TextSize::Xs.font_size())
            .child(
                div().flex().size(px(18.)).items_center().justify_center().rounded_full().bg(theme.card_strong).text_size(px(10.)).text_color(muted).child(initial),
            )
            .child(div().font_weight(FontWeight::SEMIBOLD).text_color(theme.foreground.opacity(0.9)).child(self.title.clone()));

        if !self.open {
            let label = if empty { "Say something" } else { "Carry on with what you were writing" };
            return div().flex().flex_col().px(px(12.)).pb(px(10.)).child(head).child(
                div()
                    .id("composer-folded")
                    .px(px(10.))
                    .py(px(6.))
                    .rounded(radius::md())
                    .bg(theme.card_strong)
                    .cursor_text()
                    .text_size(TextSize::Xs.font_size())
                    .text_color(muted)
                    .hover(|s| s.bg(theme.muted_hover()))
                    .press_stop("composer-folded-focus", radius::md(), window, cx)
                    .on_click(cx.listener(|this, _, window, cx| this.open(window, cx)))
                    .child(label),
            );
        }

        let tool = |id: &'static str, icon: IconName, tip: SharedString, on: bool| {
            div()
                .id(id)
                .flex()
                .size(px(24.))
                .items_center()
                .justify_center()
                .rounded(radius::md())
                .cursor_pointer()
                .text_color(if on { theme.foreground } else { muted })
                .when(on, |d| d.bg(theme.card_strong))
                .hover(|s| s.bg(theme.muted_hover()).text_color(theme.foreground))
                .tooltip(Tooltip::text(tip))
                .child(Icon::new(icon).size(px(14.)))
        };
        let keyed = |word: &str, key: Option<&str>| -> SharedString {
            match key {
                Some(k) => format!("{word}  {MOD}{k}").into(),
                None => word.to_string().into(),
            }
        };
        let preview = self.preview;
        let toolbar = div()
            .flex()
            .items_center()
            .gap(px(2.))
            .child(tool("write", IconName::Edit, "Write".into(), !preview).on_click(cx.listener(|this, _, _, cx| {
                this.preview = false;
                cx.notify();
            })))
            .child(tool("preview", IconName::Visibility, "Preview".into(), preview).on_click(cx.listener(|this, _, _, cx| {
                this.preview = true;
                cx.notify();
            })))
            .child(div().w(px(8.)))
            .child(tool("bold", IconName::FormatBold, keyed("Bold", Some("B")), false).on_click(cx.listener(|this, _, w, cx| this.mark(Format::Bold, w, cx))))
            .child(tool("italic", IconName::FormatItalic, keyed("Italic", Some("I")), false).on_click(cx.listener(|this, _, w, cx| this.mark(Format::Italic, w, cx))))
            .child(tool("code", IconName::Code, keyed("Code", Some("E")), false).on_click(cx.listener(|this, _, w, cx| this.mark(Format::Code, w, cx))))
            .child(tool("link", IconName::Link, keyed("Link", Some("K")), false).on_click(cx.listener(|this, _, w, cx| this.mark(Format::Link, w, cx))))
            .child(tool("quote", IconName::FormatQuote, keyed("Quote", None), false).on_click(cx.listener(|this, _, w, cx| this.mark(Format::Quote, w, cx))))
            .child(tool("list", IconName::FormatListBulleted, keyed("List", None), false).on_click(cx.listener(|this, _, w, cx| this.mark(Format::List, w, cx))));

        let body = if preview {
            div()
                .min_h(px(72.))
                .px(px(10.))
                .py(px(6.))
                .child(if empty {
                    div().text_size(TextSize::Xs.font_size()).text_color(muted).child("Nothing to preview").into_any_element()
                } else {
                    AgentText::new("composer-preview", words.clone()).into_any_element()
                })
                .into_any_element()
        } else {
            Textarea::new(&self.text).appearance(false).px(px(10.)).py(px(6.)).text_size(TextSize::Sm.font_size()).line_height(px(20.)).into_any_element()
        };

        div()
            .key_context("CommentComposer")
            .on_action(cx.listener(|this, _: &SendComment, window, cx| this.send(window, cx)))
            .on_action(cx.listener(|this, _: &FoldComposer, _, cx| this.fold(cx)))
            .on_action(cx.listener(|this, _: &MarkBold, w, cx| this.mark(Format::Bold, w, cx)))
            .on_action(cx.listener(|this, _: &MarkItalic, w, cx| this.mark(Format::Italic, w, cx)))
            .on_action(cx.listener(|this, _: &MarkCode, w, cx| this.mark(Format::Code, w, cx)))
            .on_action(cx.listener(|this, _: &MarkLink, w, cx| this.mark(Format::Link, w, cx)))
            .flex()
            .flex_col()
            .gap(px(8.))
            .px(px(12.))
            .pb(px(12.))
            .child(head)
            .child(toolbar)
            .child(div().rounded(radius::md()).bg(theme.background).child(body))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .child(
                        Button::new("composer-send")
                            .label(self.send_label.clone())
                            .cap(SEND)
                            .variant(ButtonVariant::Primary)
                            .size(ButtonSize::Sm)
                            .disabled(empty)
                            .on_click(cx.listener(|this, _, window, cx| this.send(window, cx))),
                    )
                    .child(
                        Button::new("composer-cancel")
                            .label("Cancel")
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .on_click(cx.listener(|this, _, _, cx| this.fold(cx))),
                    ),
            )
    }
}
