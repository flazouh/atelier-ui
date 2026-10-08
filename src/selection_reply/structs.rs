use std::time::Instant;

use gpui_kit::{Animation, AnimationExt, FontWeight, StatefulInteractiveElement};

use super::{AddReply, DropReply};

use gpui_kit::{
    Anchor, App, AppContext, Context, DispatchPhase, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, MouseUpEvent, ParentElement, Render, SharedString, Styled, Subscription, Window, anchored, base::TextSelection, canvas,
    component::input::{InputEvent, Position, Textarea, TextareaState}, deferred, div, point,
};

use crate::scale::px;
use crate::{
    motion::{Channel, Curve, Spring, cubic_bezier, ease},
    prompt_input::append_transcript,
    voice_input::{Mic, VoiceInputEvent, VoiceMode, listening_row, mic_slot},
};
use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    icon::{Icon, IconName},
    message_bubble::{MessageBubble, MessageBubbleAlign, MessageBubbleVariant},
    popover::PRIORITY,
    theme::{ActiveTheme, radius},
    typography::TextSize,
};
use super::helpers::shown;
use super::types::{CONTEXT, OFFER_IN, Phase, QUOTE_SHOWN, ReplyPreset, SelectionReplyEvent};

/// The reply to a selection: a button where the selection ended, then a small box for the note.
pub struct SelectionReply {
    phase: Phase,
    /// A press was released here and the selection is yet to be read (see [`Self::released`]); `bool`: inside the parent.
    released: Option<(gpui_kit::Point<gpui_kit::Pixels>, bool)>,
    note: Entity<TextareaState>,
    add_label: SharedString,
    /// Whether the microphone shows; the owner answers its events.
    presets: Vec<ReplyPreset>,
    /// Where the left button last went down, to tell a drag from a click.
    down: Option<gpui_kit::Point<gpui_kit::Pixels>>,
    /// How many offers have come up, so each one plays its entrance.
    pub(super) offers: u64,
    dictation: bool,
    voice: VoiceMode,
    voice_level: f32,
    voice_since: Option<Instant>,
    /// 0 shows the microphone, 1 the stop square.
    mic_swap: Channel,
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
        Self {
            phase: Phase::Idle,
            released: None,
            note,
            add_label: "Add".into(),
            presets: Vec::new(),
            offers: 0,
            down: None,
            dictation: false,
            voice: VoiceMode::Idle,
            voice_level: 0.,
            voice_since: None,
            mic_swap: Channel::new(0.),
            _note: subscription,
        }
    }

    /// The word on the button that adds the reply.
    pub fn add_label(mut self, label: impl Into<SharedString>) -> Self {
        self.add_label = label.into();
        self
    }

    /// One-press replies beside Reply on the offer: pressing one adds the reply with its note, with no box.
    pub fn presets(mut self, presets: Vec<ReplyPreset>) -> Self {
        self.presets = presets;
        self
    }
    /// Shows a microphone in the box. Its presses are reported as [`SelectionReplyEvent::Dictate`](super::SelectionReplyEvent).
    pub fn dictation(mut self, on: bool) -> Self {
        self.dictation = on;
        self
    }
    pub fn voice_mode(&self) -> VoiceMode {
        self.voice
    }
    pub fn set_voice_idle(&mut self, cx: &mut Context<Self>) {
        self.go_voice(VoiceMode::Idle, cx);
    }
    pub fn set_voice_listening(&mut self, cx: &mut Context<Self>) {
        self.go_voice(VoiceMode::Listening, cx);
    }
    /// The microphone's level now, 0 to 1.
    pub fn set_voice_level(&mut self, level: f32, cx: &mut Context<Self>) {
        self.voice_level = level;
        cx.notify();
    }
    fn go_voice(&mut self, mode: VoiceMode, cx: &mut Context<Self>) {
        let listening = mode == VoiceMode::Listening;
        if listening && self.voice != VoiceMode::Listening {
            self.voice_since = Some(Instant::now());
        }
        if !listening {
            self.voice_level = 0.;
        }
        self.voice = mode;
        let reduce = cx.reduce_motion();
        self.mic_swap.animate(if listening { 1. } else { 0. }, Curve::Spring(Spring::SWAP), 0., reduce);
        cx.notify();
    }
    /// The words the microphone heard, after the note. They are dropped when the box has closed since.
    pub fn insert_transcript(&mut self, words: &str, window: &mut Window, cx: &mut Context<Self>) {
        if self.voice == VoiceMode::Listening {
            self.go_voice(VoiceMode::Idle, cx);
        }
        let words = words.trim();
        if words.is_empty() || !matches!(self.phase, Phase::Writing { .. }) {
            return;
        }
        let written = append_transcript(self.note.read(cx).value().as_ref(), words);
        self.write_note(&written, window, cx);
    }
    /// Puts `text` in the note, with the caret at its end, where a reader goes on writing.
    fn write_note(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        let line = text.matches('\n').count() as u32;
        let character = text.rsplit('\n').next().map_or(0, |last| last.encode_utf16().count()) as u32;
        self.note.update(cx, |t, cx| {
            t.set_value(text.to_string(), window, cx);
            t.set_cursor_position(Position::new(line, character), window, cx);
        });
        window.focus(&self.note.focus_handle(cx), cx);
        cx.notify();
    }
    /// Whether a button or the box is showing.
    pub fn showing(&self) -> bool {
        !matches!(self.phase, Phase::Idle)
    }

    /// Whether a box is open that a release must leave alone: one the reader is writing in, or one for an earlier reply.
    fn holds(&self) -> bool {
        match &self.phase {
            Phase::Idle => false,
            Phase::Writing { key, quote, .. } => key.is_some() || !quote.is_empty(),
        }
    }
    /// A press was released at `at`. What is selected is read in the next frame's paint (see [`Self::settle`]), not now: a run of
    /// plain text works out its selected words as it paints, and until then the window's selection holds the whole run. With the
    /// box open the reader is writing, so a release elsewhere changes nothing.
    fn released(&mut self, at: gpui_kit::Point<gpui_kit::Pixels>, inside: bool, cx: &mut Context<Self>) {
        if self.holds() {
            return;
        }
        self.released = Some((at, inside));
        cx.notify();
    }
    /// A drag ended inside: the box opens now, in the next frame, with the note focused and the words still to be read, so
    /// the reader can type at once. Waiting for the read first would cost a frame. [`Self::settle`] fills the quote in, or
    /// closes the box when nothing is selected.
    fn offer_now(&mut self, at: gpui_kit::Point<gpui_kit::Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        if self.holds() {
            return;
        }
        if matches!(self.phase, Phase::Idle) {
            self.offers += 1;
            self.note.update(cx, |t, cx| t.set_value("", window, cx));
        }
        self.phase = Phase::Writing { at, quote: SharedString::default(), key: None };
        window.focus(&self.note.focus_handle(cx), cx);
        cx.notify();
    }
    /// The frame has painted the selection: `quote` is what it holds. If words are selected the box shows them, and if not it
    /// closes. A selection that ends beyond the parent belongs to someone else's reply.
    fn settle(&mut self, quote: &str, cx: &mut Context<Self>) {
        let Some((at, inside)) = self.released.take() else { return };
        if self.holds() {
            return;
        }
        let quote = quote.trim();
        if quote.is_empty() || !inside {
            self.phase = Phase::Idle;
        } else {
            if matches!(self.phase, Phase::Idle) {
                self.offers += 1;
            }
            let at = match self.phase {
                Phase::Writing { at, .. } => at,
                Phase::Idle => at,
            };
            self.phase = Phase::Writing { at, quote: quote.to_string().into(), key: None };
        }
        cx.notify();
    }
    /// Opens the box at `at` with `quote` and `note` already in it, to change a reply made before. Adding sends the
    /// reply with `key` handed back.
    pub fn edit(
        &mut self,
        at: gpui_kit::Point<gpui_kit::Pixels>,
        quote: impl Into<SharedString>,
        note: &str,
        key: impl Into<SharedString>,
        window: &mut Window, cx: &mut Context<Self>) {
        self.phase = Phase::Writing { at, quote: quote.into(), key: Some(key.into()) };
        self.write_note(note, window, cx);
    }
    /// Adds the reply with the note of preset `at`.
    fn preset(&mut self, at: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(preset) = self.presets.get(at).cloned() else { return };
        let Phase::Writing { quote, key: None, .. } = &self.phase else { return };
        if quote.is_empty() {
            return;
        }
        let quote = quote.clone();
        self.phase = Phase::Idle;
        self.note.update(cx, |t, cx| t.set_value("", window, cx));
        TextSelection::clear(window, cx);
        cx.emit(SelectionReplyEvent::Reply { quote, note: preset.note, key: None });
        cx.notify();
    }
    fn add(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // While it listens, adding ends the press first: the words are in the note before the reply goes.
        if self.voice == VoiceMode::Listening {
            cx.emit(SelectionReplyEvent::Dictate(VoiceInputEvent::Stop));
            return;
        }
        if matches!(&self.phase, Phase::Writing { quote, .. } if quote.is_empty()) {
            return;
        }
        let Phase::Writing { quote, key, .. } = std::mem::replace(&mut self.phase, Phase::Idle) else { return };
        let note: SharedString = self.note.read(cx).value().trim().to_string().into();
        self.note.update(cx, |t, cx| t.set_value("", window, cx));
        TextSelection::clear(window, cx);
        cx.emit(SelectionReplyEvent::Reply { quote, note, key });
        cx.notify();
    }
    /// Closes the box and lets the selection be: a press outside it may be the start of the next selection.
    fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if matches!(self.phase, Phase::Idle) {
            return;
        }
        if self.voice == VoiceMode::Listening {
            self.go_voice(VoiceMode::Idle, cx);
            cx.emit(SelectionReplyEvent::DictationCancel);
        }
        self.phase = Phase::Idle;
        self.note.update(cx, |t, cx| t.set_value("", window, cx));
        cx.notify();
    }
    /// Closes the box and drops the selection it was for (Escape).
    fn drop_reply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.dismiss(window, cx);
        TextSelection::clear(window, cx);
    }
}
impl Render for SelectionReply {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let this = cx.entity().downgrade();
        // Whatever the reader does, a release is where a selection ends. It is read after the frame's own handlers have run, so
        // the selection is settled by then.
        let watch = canvas(
            // The area takes no hits of its own, but it asks whether it is the one under the pointer: a layer that covers it
            // (a page, a dialog) blocks the pointer, and then a press and a drag are not meant for these words.
            |bounds, window, _| window.insert_hitbox(bounds, gpui_kit::HitboxBehavior::Normal),
            move |bounds, hitbox, window, cx| {
                // This paints after the words it replies to, so their selection is settled by now.
                if this.read_with(cx, |reply, _| reply.released.is_some()).unwrap_or(false) {
                    let quote = TextSelection::selected_text(window, cx);
                    this.update(cx, |reply, cx| reply.settle(&quote, cx)).ok();
                    // The offer is drawn in the next frame; ask for it, so it does not wait for the next input.
                    window.request_animation_frame();
                }
                let (down, area) = (this.clone(), hitbox.clone());
                window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                    if phase == DispatchPhase::Capture && event.button == MouseButton::Left {
                        let at = area.is_hovered_at(event.position, window).then_some(event.position);
                        down.update(cx, |reply, _| reply.down = at).ok();
                    }
                });
                let this = this.clone();
                window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                    if phase != DispatchPhase::Capture || event.button != MouseButton::Left {
                        return;
                    }
                    let (this, at, inside) = (this.clone(), event.position, bounds.contains(&event.position) && hitbox.is_hovered_at(event.position, window));
                    let moved = |from: gpui_kit::Point<gpui_kit::Pixels>| (from.x - at.x).abs() > px(3.) || (from.y - at.y).abs() > px(3.);
                    let dragged = event.click_count >= 2 || this.read_with(cx, |reply, _| reply.down.is_some_and(moved)).unwrap_or(false);
                    if inside && dragged {
                        this.update(cx, |reply, cx| reply.offer_now(at, window, cx)).ok();
                    }
                    // After the frame's own handlers have run, the selection is settled.
                    window.defer(cx, move |window, cx| {
                        this.update(cx, |reply, cx| reply.released(at, inside, cx)).ok();
                        window.refresh();
                    });
                });
            },
        )
        .absolute()
        .inset_0();
        let theme = cx.theme().clone();
        let overlay = match &self.phase {
            Phase::Idle => None,
            Phase::Writing { at, quote, key } => {
                let reduce = cx.reduce_motion();
                let swap = self.mic_swap.value();
                if self.mic_swap.is_running() || (self.voice == VoiceMode::Listening && !reduce) {
                    window.request_animation_frame();
                }
                let listening = self.voice == VoiceMode::Listening;
                let seconds = self.voice_since.map_or(0., |s| s.elapsed().as_secs_f32());
                // The quote sits in a message bubble, as the reader's own words do in the conversation. Until the words are
                // read the bubble holds its place with an ellipsis, so the box does not jump.
                let shown_quote: SharedString = if quote.is_empty() { "…".into() } else { shown(quote, QUOTE_SHOWN).into() };
                let bubble = div().debug_selector(|| "selection-reply-quote".into()).child(
                    MessageBubble::text("selection-reply-quote-bubble", shown_quote)
                        .variant(MessageBubbleVariant::Tint)
                        .align(MessageBubbleAlign::Start)
                        .animate_in(false),
                );
                // One-press replies, as coloured badges with an icon. They are for a new reply, not for changing one.
                let badges = (key.is_none() && !self.presets.is_empty()).then(|| {
                    div().flex().flex_wrap().gap(px(4.)).children(self.presets.iter().enumerate().map(|(at, preset)| {
                        let (ink, fill) = (theme.foreground, theme.foreground.opacity(0.07));
                        let name = format!("selection-reply-preset-{at}");
                        let id = gpui_kit::ElementId::Name(name.clone().into());
                        div().debug_selector(move || name.clone()).child(
                            div()
                                .id(id)
                                .flex()
                                .items_center()
                                .gap(px(4.))
                                .h(px(24.))
                                .pl(px(7.))
                                .pr(px(9.))
                                .rounded(radius::md())
                                .bg(fill)
                                .text_color(ink)
                                .text_size(px(11.5))
                                .font_weight(FontWeight::MEDIUM)
                                .cursor_pointer()
                                .hover(move |s| s.bg(ink.opacity(0.12)))
                                .active(move |s| s.bg(ink.opacity(0.18)))
                                .on_click(cx.listener(move |this, _, window, cx| this.preset(at, window, cx)))
                                .children(preset.icon.map(|icon| Icon::new(icon).size(px(13.)).color(preset.icon_color.unwrap_or(theme.muted_foreground))))
                                .child(preset.label.clone()),
                        )
                    }))
                });
                // The note stays in the tree while the microphone listens, hidden under the voice, so Enter and Escape
                // still reach it.
                let said = div()
                    .relative()
                    .child(
                        div()
                            .opacity(if listening { 0. } else { 1. })
                            .child(Textarea::new(&self.note).appearance(false).px(px(8.)).py(px(4.)).text_size(TextSize::Sm.font_size())),
                    )
                    .children(listening.then(|| {
                        div()
                            .absolute()
                            .inset_0()
                            .flex()
                            .items_center()
                            .px(px(8.))
                            .child(listening_row(self.voice_level, seconds, theme.muted_foreground))
                    }));
                let mic = self.dictation.then(|| {
                    let mic = Mic { id: "selection-reply-mic", mode: self.voice, swap, seconds, blocked: false, theme: theme.clone(), reduce };
                    mic_slot(mic, cx.listener(|this, _, _, cx| match this.voice {
                        VoiceMode::Idle | VoiceMode::Failed => cx.emit(SelectionReplyEvent::Dictate(VoiceInputEvent::Start)),
                        VoiceMode::Listening => cx.emit(SelectionReplyEvent::Dictate(VoiceInputEvent::Stop)),
                        VoiceMode::Setup => {}
                    }))
                    .debug_selector(|| "selection-reply-mic".into())
                });
                let input = div()
                    .flex()
                    .items_end()
                    .gap(px(4.))
                    .p(px(3.))
                    .rounded(radius::lg())
                    .bg(theme.background)
                    .child(div().debug_selector(|| "selection-reply-note".into()).flex_1().min_w_0().child(said))
                    .children(mic)
                    .child(
                        Button::new("selection-reply-add")
                            .debug_name("selection-reply-add")
                            .icon(IconName::ArrowUp)
                            .pill(true)
                            .variant(ButtonVariant::Primary)
                            .size(ButtonSize::Icon)
                            .tooltip(self.add_label.clone())
                            .on_click(cx.listener(|this, _, window, cx| this.add(window, cx))),
                    );
                let card = div()
                    .debug_selector(|| "selection-reply-box".into())
                    .key_context(CONTEXT)
                    .on_action(cx.listener(|this, _: &AddReply, window, cx| this.add(window, cx)))
                    .on_action(cx.listener(|this, _: &DropReply, window, cx| this.drop_reply(window, cx)))
                    .on_mouse_down_out(cx.listener(|this, _, window, cx| this.dismiss(window, cx)))
                    .w(px(320.))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .p(px(8.))
                    .rounded(radius::xl())
                    .bg(theme.card_strong)
                    .shadow_md()
                    .child(bubble)
                    .children(badges)
                    .child(input);
                // It rises a few pixels and fades in from half strength; with Reduce Motion it is there at once.
                let card = if reduce {
                    card.into_any_element()
                } else {
                    card.with_animation(
                        gpui_kit::ElementId::Name(format!("selection-reply-box-{}", self.offers).into()),
                        Animation::new(OFFER_IN).with_easing(|t| cubic_bezier(ease::OUT, t)),
                        |card, t| card.opacity(0.5 + 0.5 * t).mt(px(3. * (1. - t))),
                    )
                    .into_any_element()
                };
                Some(floating(*at, window, card))
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
