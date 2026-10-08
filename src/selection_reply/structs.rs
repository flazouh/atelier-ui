use std::time::Instant;

use gpui_kit::{Animation, AnimationExt};

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
    icon::IconName,
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
    offers: u64,
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

    /// A drag ended inside: the bar comes up now, in the next frame, with the words still to be read. Waiting for the read
    /// first would cost one more frame, which the reader feels as a lag. [`Self::settle`] fills the quote in, or takes the
    /// bar back when nothing is selected.
    fn offer_now(&mut self, at: gpui_kit::Point<gpui_kit::Pixels>, cx: &mut Context<Self>) {
        match self.phase {
            Phase::Writing { .. } => return,
            Phase::Offer { .. } => {}
            Phase::Idle => self.offers += 1,
        }
        self.phase = Phase::Offer { at, quote: SharedString::default() };
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
        let was_offer = matches!(self.phase, Phase::Offer { .. });
        self.phase = if quote.is_empty() || !inside { Phase::Idle } else { Phase::Offer { at, quote: quote.to_string().into() } };
        if matches!(self.phase, Phase::Offer { .. }) && !was_offer {
            self.offers += 1;
        }
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
        self.write_note(note, window, cx);
    }
    fn write(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if matches!(&self.phase, Phase::Offer { quote, .. } if quote.is_empty()) {
            return;
        }
        if let Phase::Offer { at, quote } = std::mem::replace(&mut self.phase, Phase::Idle) {
            self.phase = Phase::Writing { at, quote, key: None };
            window.focus(&self.note.focus_handle(cx), cx);
            cx.notify();
        }
    }

    /// Adds the reply on the offer with the note of preset `at`.
    fn preset(&mut self, at: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(preset) = self.presets.get(at).cloned() else { return };
        if matches!(&self.phase, Phase::Offer { quote, .. } if quote.is_empty()) {
            return;
        }
        let Phase::Offer { quote, .. } = std::mem::replace(&mut self.phase, Phase::Idle) else { return };
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
        let Phase::Writing { quote, key, .. } = std::mem::replace(&mut self.phase, Phase::Idle) else { return };
        let note: SharedString = self.note.read(cx).value().trim().to_string().into();
        self.note.update(cx, |t, cx| t.set_value("", window, cx));
        TextSelection::clear(window, cx);
        cx.emit(SelectionReplyEvent::Reply { quote, note, key });
        cx.notify();
    }

    /// Drops the button or the box, and the selection it was for, so the release that pressed Cancel does not offer it again.
    fn drop_reply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.voice == VoiceMode::Listening {
            self.go_voice(VoiceMode::Idle, cx);
            cx.emit(SelectionReplyEvent::DictationCancel);
        }
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
                    // The offer is drawn in the next frame; ask for it, so it does not wait for the next input.
                    window.request_animation_frame();
                }
                let down = this.clone();
                window.on_mouse_event(move |event: &MouseDownEvent, phase, _, cx| {
                    if phase == DispatchPhase::Capture && event.button == MouseButton::Left {
                        down.update(cx, |reply, _| reply.down = Some(event.position)).ok();
                    }
                });
                let this = this.clone();
                window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                    if phase != DispatchPhase::Capture || event.button != MouseButton::Left {
                        return;
                    }
                    let (this, at, inside) = (this.clone(), event.position, bounds.contains(&event.position));
                    let moved = |from: gpui_kit::Point<gpui_kit::Pixels>| (from.x - at.x).abs() > px(3.) || (from.y - at.y).abs() > px(3.);
                    let dragged = event.click_count >= 2 || this.read_with(cx, |reply, _| reply.down.is_some_and(moved)).unwrap_or(false);
                    if inside && dragged {
                        this.update(cx, |reply, cx| reply.offer_now(at, cx)).ok();
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
            Phase::Offer { at, .. } => {
                let reply = Button::new("selection-reply-offer-reply")
                    .debug_name("selection-reply-offer-reply")
                    .icon(IconName::ChatBubble)
                    .label("Reply")
                    .variant(ButtonVariant::Tinted)
                    .size(ButtonSize::Sm)
                    .on_click(cx.listener(|this, _, window, cx| this.write(window, cx)));
                let divider = (!self.presets.is_empty())
                    .then(|| div().flex_none().w(px(1.)).h(px(14.)).mx(px(3.)).bg(theme.muted_foreground.opacity(0.25)));
                let presets = self.presets.iter().enumerate().map(|(at, preset)| {
                    let name = format!("selection-reply-preset-{at}");
                    let button = Button::new(gpui_kit::ElementId::Name(name.clone().into()))
                        .label(preset.label.clone())
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .on_click(cx.listener(move |this, _, window, cx| this.preset(at, window, cx)));
                    div().debug_selector(move || name.clone()).child(button)
                });
                let card = div()
                    .debug_selector(|| "selection-reply-offer".into())
                    .flex()
                    .items_center()
                    .gap(px(2.))
                    .p(px(3.))
                    .rounded(radius::xl())
                    .bg(theme.card_strong)
                    .shadow_md()
                    .child(reply)
                    .children(divider)
                    .children(presets);
                // It rises a few pixels and fades in; with Reduce Motion it is there at once.
                let card = if cx.reduce_motion() {
                    card.into_any_element()
                } else {
                    card.with_animation(
                        gpui_kit::ElementId::Name(format!("selection-reply-offer-{}", self.offers).into()),
                        Animation::new(OFFER_IN).with_easing(|t| cubic_bezier(ease::OUT, t)),
                        |card, t| card.opacity(0.5 + 0.5 * t).mt(px(3. * (1. - t))),
                    )
                    .into_any_element()
                };
                Some(floating(*at, window, card))
            }
            Phase::Writing { at, quote, .. } => {
                let reduce = cx.reduce_motion();
                let swap = self.mic_swap.value();
                if self.mic_swap.is_running() || (self.voice == VoiceMode::Listening && !reduce) {
                    window.request_animation_frame();
                }
                let listening = self.voice == VoiceMode::Listening;
                let seconds = self.voice_since.map_or(0., |s| s.elapsed().as_secs_f32());
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
                let card = div()
                    .debug_selector(|| "selection-reply-box".into())
                    .key_context(CONTEXT)
                    .on_action(cx.listener(|this, _: &AddReply, window, cx| this.add(window, cx)))
                    .on_action(cx.listener(|this, _: &DropReply, window, cx| this.drop_reply(window, cx)))
                    .w(px(300.))
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .p(px(6.))
                    .rounded(radius::xl())
                    .bg(theme.card_strong)
                    .shadow_md()
                    .child(
                        div()
                            .debug_selector(|| "selection-reply-quote".into())
                            .mx(px(8.))
                            .mt(px(2.))
                            .pl(px(6.))
                            .border_l_2()
                            .border_color(theme.muted_foreground.opacity(0.4))
                            .text_size(TextSize::Xs.font_size())
                            .text_color(theme.muted_foreground)
                            .line_clamp(1)
                            .child(shown(quote, QUOTE_SHOWN)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_end()
                            .gap(px(4.))
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
                            ),
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
