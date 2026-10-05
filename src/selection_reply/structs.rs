use super::{AddReply, DropReply};

use gpui_kit::{
    Anchor, App, AppContext, Context, DispatchPhase, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement, IntoElement,
    MouseButton, MouseUpEvent, ParentElement, Render, SharedString, Styled, Subscription, Window, anchored, base::TextSelection, canvas,
    component::input::{InputEvent, Position, Textarea, TextareaState}, deferred, div, point,
};

use crate::scale::px;
use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    focus::Field,
    icon::IconName,
    kbd::Kbd,
    popover::PRIORITY,
    theme::{ActiveTheme, radius},
    typography::TextSize,
};
use super::helpers::shown;
use super::types::{CONTEXT, Phase, QUOTE_SHOWN, SelectionReplyEvent};

/// The reply to a selection: a button where the selection ended, then a small box for the note.
pub struct SelectionReply {
    phase: Phase,
    /// A press was released here and the selection is yet to be read (see [`Self::released`]); `bool`: inside the parent.
    released: Option<(gpui_kit::Point<gpui_kit::Pixels>, bool)>,
    note: Entity<TextareaState>,
    add_label: SharedString,
    _note: Subscription,
}

impl EventEmitter<SelectionReplyEvent> for SelectionReply {}

impl Focusable for SelectionReply {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.note.focus_handle(cx)
    }
}

impl SelectionReply {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let note = cx.new(|cx| TextareaState::new(window, cx).auto_grow(1, 5).placeholder("Add a note"));
        let subscription = cx.subscribe_in(&note, window, |_, _, event: &InputEvent, _, cx| {
            if let InputEvent::Change = event {
                cx.notify()
            }
        });
        Self { phase: Phase::Idle, released: None, note, add_label: "Add".into(), _note: subscription }
    }

    /// The word on the button that adds the reply.
    pub fn add_label(mut self, label: impl Into<SharedString>) -> Self {
        self.add_label = label.into();
        self
    }

    /// Whether a button or the box is showing.
    pub fn showing(&self) -> bool {
        !matches!(self.phase, Phase::Idle)
    }

    /// A press was released at `at`. What is selected is read in the next frame's paint (see [`Self::settle`]), not now: a run of
    /// plain text works out its selected words as it paints, and until then the window's selection holds the whole run. With the
    /// box open the reader is writing, so a release elsewhere changes nothing.
    fn released(&mut self, at: gpui_kit::Point<gpui_kit::Pixels>, inside: bool, cx: &mut Context<Self>) {
        if matches!(self.phase, Phase::Writing { .. }) {
            return;
        }
        self.released = Some((at, inside));
        cx.notify();
    }

    /// The frame has painted the selection: `quote` is what it holds. If words are selected the button comes up where the press
    /// was released, and if not it goes. A selection that ends beyond the parent belongs to someone else's reply.
    fn settle(&mut self, quote: &str, cx: &mut Context<Self>) {
        let Some((at, inside)) = self.released.take() else { return };
        if matches!(self.phase, Phase::Writing { .. }) {
            return;
        }
        let quote = quote.trim();
        self.phase = if quote.is_empty() || !inside { Phase::Idle } else { Phase::Offer { at, quote: quote.to_string().into() } };
        cx.notify();
    }

    /// Opens the box at `at` with `quote` and `note` already in it, to change a reply made before. Adding sends the same event, with `key` in it.
    pub fn edit(
        &mut self,
        at: gpui_kit::Point<gpui_kit::Pixels>,
        quote: impl Into<SharedString>,
        note: &str,
        key: impl Into<SharedString>,
        window: &mut Window, cx: &mut Context<Self>) {
        self.phase = Phase::Writing { at, quote: quote.into(), key: Some(key.into()) };
        let line = note.matches('\n').count() as u32;
        let character = note.rsplit('\n').next().map_or(0, |last| last.encode_utf16().count()) as u32;
        // The caret goes to the end, where a reader goes on writing.
        self.note.update(cx, |t, cx| {
            t.set_value(note.to_string(), window, cx);
            t.set_cursor_position(Position::new(line, character), window, cx);
        });
        window.focus(&self.note.focus_handle(cx), cx);
        cx.notify();
    }

    fn write(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Phase::Offer { at, quote } = std::mem::replace(&mut self.phase, Phase::Idle) {
            self.phase = Phase::Writing { at, quote, key: None };
            window.focus(&self.note.focus_handle(cx), cx);
            cx.notify();
        }
    }

    fn add(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Phase::Writing { quote, key, .. } = std::mem::replace(&mut self.phase, Phase::Idle) else { return };
        let note: SharedString = self.note.read(cx).value().trim().to_string().into();
        self.note.update(cx, |t, cx| t.set_value("", window, cx));
        TextSelection::clear(window, cx);
        cx.emit(SelectionReplyEvent::Reply { quote, note, key });
        cx.notify();
    }

    /// Drops the button or the box, and the selection it was for, so the release that pressed Cancel does not offer it again.
    fn drop_reply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.phase = Phase::Idle;
        self.note.update(cx, |t, cx| t.set_value("", window, cx));
        TextSelection::clear(window, cx);
        cx.notify();
    }
}

impl Render for SelectionReply {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let this = cx.entity().downgrade();
        // Whatever the reader does, a release is where a selection ends. It is read after the frame's own handlers have run, so
        // the selection is settled by then.
        let watch = canvas(
            |_, _, _| {},
            move |bounds, _, window, cx| {
                // This paints after the words it replies to, so their selection is settled by now.
                if this.read_with(cx, |reply, _| reply.released.is_some()).unwrap_or(false) {
                    let quote = TextSelection::selected_text(window, cx);
                    this.update(cx, |reply, cx| reply.settle(&quote, cx)).ok();
                }
                let this = this.clone();
                window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                    if phase != DispatchPhase::Capture || event.button != MouseButton::Left {
                        return;
                    }
                    let (this, at, inside) = (this.clone(), event.position, bounds.contains(&event.position));
                    // After the frame's own handlers have run, the selection is settled.
                    window.defer(cx, move |_, cx| {
                        this.update(cx, |reply, cx| reply.released(at, inside, cx)).ok();
                    });
                });
            },
        )
        .absolute()
        .inset_0();
        let theme = cx.theme().clone();
        let overlay = match &self.phase {
            Phase::Idle => None,
            Phase::Offer { at, .. } => {
                let button = Button::new("selection-reply-offer")
                    .icon(IconName::ChatBubble)
                    .label("Reply")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .on_click(cx.listener(|this, _, window, cx| this.write(window, cx)));
                let card = div().debug_selector(|| "selection-reply-offer".into()).rounded(radius::lg()).bg(theme.card_strong).shadow_md().child(button);
                Some(floating(*at, window, card.into_any_element()))
            }
            Phase::Writing { at, quote, .. } => {
                let card = div()
                    .debug_selector(|| "selection-reply-box".into())
                    .key_context(CONTEXT)
                    .on_action(cx.listener(|this, _: &AddReply, window, cx| this.add(window, cx)))
                    .on_action(cx.listener(|this, _: &DropReply, window, cx| this.drop_reply(window, cx)))
                    .w(px(320.))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .p(px(10.))
                    .rounded(radius::xl())
                    .bg(theme.card_strong)
                    .shadow_md()
                    .child(
                        div()
                            .debug_selector(|| "selection-reply-quote".into())
                            .pl(px(8.))
                            .border_l_2()
                            .border_color(theme.muted_foreground.opacity(0.5))
                            .text_size(TextSize::Xs.font_size())
                            .text_color(theme.muted_foreground)
                            .line_clamp(3)
                            .child(shown(quote, QUOTE_SHOWN)),
                    )
                    .child(
                        Field::new(
                            self.note.focus_handle(cx),
                            Textarea::new(&self.note).appearance(false).px(px(8.)).py(px(6.)).text_size(TextSize::Sm.font_size()).line_height(px(20.)),
                        )
                        .radius(radius::lg())
                        .surface(theme.background),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_end()
                            .gap(px(6.))
                            .child(
                                Button::new("selection-reply-cancel").debug_name("selection-reply-cancel")
                                    .label("Cancel")
                                    .variant(ButtonVariant::Ghost)
                                    .size(ButtonSize::Sm)
                                    .on_click(cx.listener(|this, _, window, cx| this.drop_reply(window, cx))),
                            )
                            .child(
                                Button::new("selection-reply-add")
                                    .label(self.add_label.clone())
                                    .variant(ButtonVariant::Primary)
                                    .size(ButtonSize::Sm)
                                    .on_click(cx.listener(|this, _, window, cx| this.add(window, cx))),
                            )
                            .child(Kbd::new("↵")),
                    );
                Some(floating(*at, window, card.into_any_element()))
            }
        };
        div().absolute().inset_0().child(watch).children(overlay)
    }
}

/// `content` over everything, next to `at`: below and right of it, or above it when it is in the lower half of the window, and
/// kept inside the window.
fn floating(at: gpui_kit::Point<gpui_kit::Pixels>, window: &Window, content: gpui_kit::AnyElement) -> gpui_kit::Deferred {
    let low = at.y > window.viewport_size().height / 2.;
    let (anchor, at) = if low { (Anchor::BottomLeft, at + point(px(4.), px(-12.))) } else { (Anchor::TopLeft, at + point(px(4.), px(12.))) };
    deferred(anchored().position(at).anchor(anchor).snap_to_window_with_margin(px(8.)).child(div().occlude().child(content)))
        .with_priority(PRIORITY + 1)
}
