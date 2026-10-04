use std::time::Instant;

use gpui_kit::{
    App,
    AppContext,
    Bounds,
    Context,
    Entity,
    EventEmitter,
    FocusHandle,
    Focusable,
    InteractiveElement,
    IntoElement,
    ParentElement,
    Pixels,
    Render,
    SharedString,
    StatefulInteractiveElement,
    Styled,
    Subscription,
    Window,
    component::input::{Backspace, InputEvent, MoveDown, MoveUp, Position, Textarea, TextareaState},
    div,
    prelude::FluentBuilder,
};

use std::rc::Rc;

use crate::scale::px;
use crate::context_meter::ContextMeter;
use crate::context_usage::{self, ContextPart, ContextUsage};
use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    button_group::ButtonGroup,
    combobox::{ComboEntry, ComboList, ComboRow, ComboStyle},
    command_item::{CommandItem, CommandSource, Trigger, ranked, trigger},
    icon::{Icon, IconName},
    menu::{Choice as MenuChoice, Entry, Menu, MenuItem, Origin},
    morph::Morph,
    motion::{Channel, Curve, Spring, ease},
    popover::{Hang, Popover, Side},
    select::Select,
    theme::{ActiveTheme, radius},
    typography::TextSize,
    voice_input::{self, VoiceDevice, VoiceMode},
    voice_setup::{SetupPhase, VoiceSetup},
};
use super::types::{Chip, ChipLook, LiveWords, Message, Pasted, PICK_GAP, PICK_MOST, PICK_PAD, PICK_ROW, PromptInputEvent, STEER_HINT, Sending};
use super::helpers::{append_transcript, live_text};

/// One choice in the model picker.
#[derive(Clone, Debug)]
pub struct PromptModel {
    pub value: SharedString,
    pub label: SharedString,
    pub icon: Option<IconName>,
    /// The lab's mark, before the label.
    pub mark: Option<crate::model_badge::BrandMark>,
}

impl PromptModel {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self { value: value.into(), label: label.into(), icon: None, mark: None }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn mark(mut self, mark: crate::model_badge::BrandMark) -> Self {
        self.mark = Some(mark);
        self
    }
}

/// One item in the "add to prompt" menu.
#[derive(Clone, Debug)]
pub struct PromptAction {
    pub value: SharedString,
    pub label: SharedString,
    pub description: Option<SharedString>,
    pub icon: Option<IconName>,
}

impl PromptAction {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self { value: value.into(), label: label.into(), description: None, icon: None }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// The list open under the text: its trigger, the rows that match (indices into the commands or the
/// files), and the one in front.
struct Picking {
    pub(super) trigger: Trigger,
    matches: Vec<usize>,
    active: usize,
}

/// The panel that morphs the Plus icon 45° open and fades in the menu above it.
pub(super) struct ActionsMenu {
    pub(super) open: bool,
    /// Pointer over the Plus trigger. A press there is not an outside press, so the trigger alone
    /// decides whether the menu toggles.
    trigger_hovered: bool,
    rotate: Channel,
}

impl ActionsMenu {
    pub(super) fn new() -> Self {
        Self { open: false, trigger_hovered: false, rotate: Channel::new(0.) }
    }

    fn set_open(&mut self, open: bool, reduce: bool) {
        self.open = open;
        self.rotate.animate(if open { 45. } else { 0. }, Curve::Spring(Spring::SWAP), 0., reduce);
    }
}

pub struct PromptInput {
    pub(super) text: Entity<TextareaState>,
    pub(super) models: Vec<PromptModel>,
    /// A second picker beside the model's, such as how much the agent may do without asking.
    pub(super) modes: Vec<SharedString>,
    pub(super) mode: usize,
    pub(super) model: usize,
    actions: Vec<PromptAction>,
    pub(super) menu: ActionsMenu,
    /// 0 shows Send, 1 shows Stop; animates between them on `Spring::SWAP`.
    pub(super) send_swap: Channel,
    /// Dictation: the microphone shows, and the owner answers its events.
    pub(super) dictation: bool,
    pub(super) voice: VoiceMode,
    /// The last mode that was not idle, which the bar keeps showing while it fades out.
    pub(super) voice_face: VoiceMode,
    pub(super) voice_phase: SetupPhase,
    pub(super) voice_total_mb: f32,
    /// Why the last press ended without words, shown while the mode is [`VoiceMode::Failed`].
    pub(super) voice_error: SharedString,
    pub(super) voice_level: f32,
    pub(super) live: LiveWords,
    /// How strongly each of the live words shows while it comes in.
    pub(super) ink: crate::live_ink::Ink,
    /// The microphones in the menu, the one chosen (`None` is the system's default), and whether a press records only while held.
    pub(super) voice_devices: Vec<VoiceDevice>,
    pub(super) voice_device: Option<SharedString>,
    pub(super) voice_hold: bool,
    pub(super) mic_menu: bool,
    /// The button is down in hold mode.
    held: bool,
    pub(super) voice_since: Option<Instant>,
    /// 0 shows the microphone, 1 the stop square.
    pub(super) mic_swap: Channel,
    /// 0 shows Plus, the model and the mode; 1 shows the setup or the bars.
    pub(super) voice_fade: Channel,
    pub(super) running: bool,
    pub(super) disabled: bool,
    /// The tokens the agent's context holds, and its window, once the agent has told both.
    pub(super) context: Option<(u64, u64)>,
    /// What fills the context, as the agent could tell it, for the panel the ring opens.
    context_parts: Vec<ContextPart>,
    context_open: bool,
    /// The messages waiting for the running turn to end, as the owner keeps them.
    pub(super) queued: Vec<SharedString>,
    /// What `/` offers, and `@` ([`crate::command_item`]).
    pub(super) commands: Vec<CommandItem>,
    pub(super) files: Vec<SharedString>,
    /// The chips over the text: the files picked from the `@` list, and whatever the owner adds.
    pub(super) chips: Vec<Chip>,
    /// A skill picked from the list runs at once instead of waiting in the box.
    run_picked_skills: bool,
    /// A command picked from the `/` list that waits for its words, shown as a chip in front: the next message runs it.
    command: Option<SharedString>,
    /// Whether a paste or a drop is handed to the owner ([`PromptInputEvent::Paste`]) instead of going into the text.
    paste_chips: bool,
    picking: Option<Picking>,

    /// The box, as last drawn: the list opens from it.
    pub(super) frame: Option<Bounds<Pixels>>,
    _subscription: Subscription,
}

/// The padding the text field puts inside itself (its medium size), on top of ours.
/// The live words sit where the field's own text would, so these must match it.
const EDITOR_PAD_X: f32 = 10.;
const EDITOR_PAD_Y: f32 = 8.;
/// How much of the field's own top and bottom padding is cut off, so the text row is a line (24px) and 8px of air,
/// the same height as the controls' row under it, not a line and 16px.
const TRIM_Y: f32 = 4.;

impl PromptInput {
    /// `placeholder` shows in the empty box; `default_value` seeds the text, as beui's `defaultValue`.
    pub fn new(
        placeholder: impl Into<SharedString>,
        default_value: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let placeholder = placeholder.into();
        let default_value = default_value.into();
        let text = cx.new(|cx| {
            TextareaState::new(window, cx)
                .auto_grow(1, 8)
                .submit_on_enter(true)
                .placeholder(placeholder)
                .default_value(default_value)
        });
        let subscription = cx.subscribe_in(&text, window, |this, _, event: &InputEvent, window, cx| match event {
            // Enter sends, and ⌘↵ (⌃↵ elsewhere) too, as the brief's key; while a turn runs ⌘↵ queues instead.
            InputEvent::PressEnter { shift: false, secondary } => {
                let sending = if *secondary && this.running { Sending::AfterTurn } else { Sending::Now };
                this.send(sending, window, cx)
            }
            // Send turns on and off with the text, so redraw on every edit; a `/` or an `@` opens a list.
            InputEvent::Change => {
                this.refresh_picking(cx);
                this.retarget_send(cx);
                cx.notify()
            }
            _ => {}
        });
        Self {
            text,
            models: Vec::new(),
            modes: Vec::new(),
            mode: 0,
            model: 0,
            actions: Vec::new(),
            menu: ActionsMenu::new(),
            send_swap: Channel::new(0.),
            dictation: false,
            voice: VoiceMode::Idle,
            voice_face: VoiceMode::Listening,
            voice_phase: SetupPhase::Prepare,
            voice_total_mb: 164.,
            voice_error: SharedString::default(),
            voice_level: 0.,
            live: LiveWords::Off,
            ink: crate::live_ink::Ink::new(std::time::Instant::now()),
            voice_devices: Vec::new(),
            voice_device: None,
            voice_hold: false,
            mic_menu: false,
            held: false,
            voice_since: None,
            mic_swap: Channel::new(0.),
            voice_fade: Channel::new(0.),
            running: false,
            disabled: false,
            context: None,
            context_parts: Vec::new(),
            context_open: false,
            queued: Vec::new(),
            commands: Vec::new(),
            files: Vec::new(),
            chips: Vec::new(),
            run_picked_skills: false,
            paste_chips: false,
            command: None,
            picking: None,
            frame: None,
            _subscription: subscription,
        }
    }

    /// The models the picker offers. The first is selected until [`Self::set_model`] runs.
    pub fn models(mut self, models: Vec<PromptModel>) -> Self {
        self.models = models;
        self
    }

    /// The modes the second picker offers, by their words; empty draws no picker.
    pub fn modes(mut self, modes: Vec<SharedString>) -> Self {
        self.modes = modes;
        self
    }

    /// The mode the picker shows, by its words; `None` when there is no picker.
    pub fn mode(&self) -> Option<&SharedString> {
        self.modes.get(self.mode)
    }

    pub fn set_mode(&mut self, words: &str, cx: &mut Context<Self>) {
        if let Some(i) = self.modes.iter().position(|m| m == words) {
            self.mode = i;
            cx.notify();
        }
    }

    /// The items the "add to prompt" menu offers. Empty renders no Plus button, as beui does.
    pub fn actions(mut self, actions: Vec<PromptAction>) -> Self {
        self.actions = actions;
        self
    }

    /// Preselects the model with this value; falls back to the first model.
    pub fn model(mut self, value: impl Into<SharedString>) -> Self {
        let value = value.into();
        if let Some(i) = self.models.iter().position(|m| m.value == value) {
            self.model = i;
        }
        self
    }

    pub fn set_model(&mut self, value: impl Into<SharedString>, cx: &mut Context<Self>) {
        let value = value.into();
        if let Some(i) = self.models.iter().position(|m| m.value == value) {
            self.model = i;
            cx.notify();
        }
    }

    /// Shows how full the agent's context is: `used` tokens of `window`.
    pub fn set_context(&mut self, used: u64, window: u64, cx: &mut Context<Self>) {
        if self.context != Some((used, window)) {
            self.context = Some((used, window));
            cx.notify();
        }
    }

    /// Tells what fills the context, for the panel the ring opens. With none, the panel shows only what is in use.
    pub fn set_context_parts(&mut self, parts: Vec<ContextPart>, cx: &mut Context<Self>) {
        if self.context_parts != parts {
            self.context_parts = parts;
            cx.notify();
        }
    }

    /// Opens or closes the panel the context ring opens. It shows only while the context is known.
    pub fn set_context_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.context_open != open {
            self.context_open = open;
            cx.notify();
        }
    }

    /// Shows the messages waiting for the running turn to end, over the text.
    pub fn set_queued(&mut self, queued: Vec<SharedString>, cx: &mut Context<Self>) {
        if self.queued != queued {
            self.queued = queued;
            cx.notify();
        }
    }

    /// While the agent works, Send steers the turn when the box has text and turns into Stop when it is empty.
    pub fn set_running(&mut self, running: bool, cx: &mut Context<Self>) {
        self.running = running;
        self.retarget_send(cx);
        // The Plus trigger disables while running, so its menu cannot stay open behind it.
        if running {
            self.menu.set_open(false, cx.reduce_motion());
        }
        cx.notify();
    }

    /// Whether the button stops the turn: it runs, and there is nothing to send into it.
    pub(super) fn stops(&self, cx: &App) -> bool {
        self.running && self.text(cx).trim().is_empty() && self.chips.is_empty() && self.command.is_none()
    }

    fn retarget_send(&mut self, cx: &mut Context<Self>) {
        let target = if self.stops(cx) { 1. } else { 0. };
        if self.send_swap.target() != target {
            self.send_swap.animate(target, Curve::Spring(Spring::SWAP), 0., cx.reduce_motion());
        }
    }

    /// Shows the microphone before Send, and starts hearing it.
    pub fn set_dictation(&mut self, on: bool, cx: &mut Context<Self>) {
        self.dictation = on;
        if !on {
            self.go_voice(VoiceMode::Idle, cx);
        }
        cx.notify();
    }

    /// Which part of dictation shows now.
    pub fn voice_mode(&self) -> VoiceMode {
        self.voice
    }

    pub fn set_voice_idle(&mut self, cx: &mut Context<Self>) {
        self.go_voice(VoiceMode::Idle, cx);
    }

    /// Throws away the words waiting for the model: the box goes idle and the owner hears [`PromptInputEvent::DictationDiscard`].
    pub fn discard_waiting(&mut self, cx: &mut Context<Self>) {
        if self.voice != VoiceMode::Setup {
            return;
        }
        self.go_voice(VoiceMode::Idle, cx);
        cx.emit(PromptInputEvent::DictationDiscard);
    }

    /// Shows the first-use setup at `phase`; call again as it moves.
    pub fn set_voice_setup(&mut self, phase: SetupPhase, cx: &mut Context<Self>) {
        self.voice_phase = phase;
        self.go_voice(VoiceMode::Setup, cx);
    }

    pub fn set_voice_listening(&mut self, cx: &mut Context<Self>) {
        self.go_voice(VoiceMode::Listening, cx);
    }

    /// Says why the press ended without words, in place of Plus, the model and the mode, until the owner sets another mode
    /// (the microphone can be pressed meanwhile).
    pub fn set_voice_error(&mut self, message: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.voice_error = message.into();
        self.go_voice(VoiceMode::Failed, cx);
    }

    /// The microphones to offer, and which is chosen (`None` for the system's default, which the list may also name).
    pub fn set_voice_devices(&mut self, devices: Vec<VoiceDevice>, selected: Option<SharedString>, cx: &mut Context<Self>) {
        self.voice_devices = devices;
        self.voice_device = selected;
        cx.notify();
    }

    /// Whether the microphone records only while it is held down.
    pub fn set_voice_hold(&mut self, hold: bool, cx: &mut Context<Self>) {
        self.voice_hold = hold;
        cx.notify();
    }

    /// Whether dictation holds the box: it listens. Send waits for it. Words that wait for the model do not hold it: what was
    /// typed can go meanwhile, and the words come into the box when they are ready.
    pub(super) fn dictating(&self) -> bool {
        self.voice == VoiceMode::Listening
    }

    /// The size of the model being fetched, for the setup's words.
    pub fn set_voice_total_mb(&mut self, total_mb: f32, cx: &mut Context<Self>) {
        self.voice_total_mb = total_mb;
        cx.notify();
    }

    /// The microphone's level now, 0 to 1.
    pub fn set_voice_level(&mut self, level: f32, cx: &mut Context<Self>) {
        self.voice_level = level;
        cx.notify();
    }

    pub(super) fn go_voice(&mut self, mode: VoiceMode, cx: &mut Context<Self>) {
        let reduce = cx.reduce_motion();
        let listening = mode == VoiceMode::Listening;
        if listening && self.voice != VoiceMode::Listening {
            self.voice_since = Some(Instant::now());
        }
        if mode == VoiceMode::Idle {
            self.voice_level = 0.;
        } else {
            self.voice_face = mode;
            // The menus cannot stay open behind the bars that cover their buttons.
            self.menu.set_open(false, reduce);
            self.mic_menu = false;
        }
        self.voice = mode;
        self.mic_swap.animate(if listening { 1. } else { 0. }, Curve::Spring(Spring::SWAP), 0., reduce);
        self.voice_fade.animate(if mode == VoiceMode::Idle { 0. } else { 1. }, Curve::Ease(0.22, ease::OUT), 0., reduce);
        cx.notify();
    }

    /// The microphone was pressed, by the button or the owner's key: it listens at once, and the owner is told to start. The
    /// owner's own work (opening the microphone, fetching the model) comes after, so the press is never kept waiting. A press
    /// while an earlier recording waits for the model starts a new one.
    pub fn press_mic(&mut self, cx: &mut Context<Self>) {
        if !self.dictation || self.disabled || self.running || self.voice == VoiceMode::Listening {
            return;
        }
        self.go_voice(VoiceMode::Listening, cx);
        cx.emit(PromptInputEvent::DictationStart);
    }

    /// The press is over: the owner turns it into words.
    pub fn release_mic(&mut self, cx: &mut Context<Self>) {
        self.held = false;
        if self.voice != VoiceMode::Listening {
            return;
        }
        self.go_voice(VoiceMode::Idle, cx);
        cx.emit(PromptInputEvent::DictationStop);
    }

    /// The press is taken back, such as a key that turned out to be part of a shortcut: no stop, no words.
    pub fn cancel_mic(&mut self, cx: &mut Context<Self>) {
        self.held = false;
        if self.voice != VoiceMode::Listening {
            return;
        }
        self.go_voice(VoiceMode::Idle, cx);
        cx.emit(PromptInputEvent::DictationCancel);
    }

    /// Writes what was said at the end of the text, after a space when the text does not end in one, and puts the caret after
    /// it, so the user reads it, fixes it and sends it.
    pub fn insert_transcript(&mut self, words: &str, window: &mut Window, cx: &mut Context<Self>) {
        let words = words.trim();
        let base = match std::mem::take(&mut self.live) {
            // The live words give way to the final ones.
            LiveWords::Showing { base, shown } if self.text(cx) == live_text(&base, &shown) => base,
            _ => self.text(cx).to_string(),
        };
        if words.is_empty() {
            return self.write(&base, base.len(), window, cx);
        }
        let written = append_transcript(&base, words);
        self.write(&written, written.len(), window, cx);
        cx.notify();
    }

    /// Shows the words heard so far while the press still records, after what was written; each call replaces the last.
    /// [`insert_transcript`](Self::insert_transcript) puts the final words in their place, and
    /// [`end_live_transcript`](Self::end_live_transcript) takes them out. Once the person edits the box meanwhile, the words
    /// shown stay and stop moving.
    pub fn set_live_transcript(&mut self, words: &str, window: &mut Window, cx: &mut Context<Self>) {
        let now = self.text(cx).to_string();
        let (base, shown) = match &self.live {
            LiveWords::Left => return,
            LiveWords::Off => (now.clone(), String::new()),
            LiveWords::Showing { base, shown } => (base.clone(), shown.clone()),
        };
        if now != live_text(&base, &shown) {
            self.live = LiveWords::Left;
            return;
        }
        let written = live_text(&base, words);
        if self.live == LiveWords::Off {
            self.ink = crate::live_ink::Ink::new(std::time::Instant::now());
        }
        self.ink.set_still(cx.reduce_motion());
        self.ink.observe(words.trim());
        self.write(&written, written.len(), window, cx);
        self.live = LiveWords::Showing { base, shown: words.trim().to_string() };
        cx.notify();
    }

    /// The press ended without words: the live words go, unless the person has edited the box since.
    pub fn end_live_transcript(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let LiveWords::Showing { base, shown } = std::mem::take(&mut self.live)
            && self.text(cx) == live_text(&base, &shown)
        {
            self.write(&base, base.len(), window, cx);
            cx.notify();
        }
    }

    /// The text as it is while a press records: each live word in the strength of ink it has come to, laid exactly where the
    /// box lays the same text. `None` when no words are live, or the person has edited the box.
    fn live_overlay(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Option<gpui_kit::AnyElement> {
        let LiveWords::Showing { base, shown } = &self.live else { return None };
        let written = live_text(base, shown);
        if self.text(cx) != written {
            return None;
        }
        let theme = cx.theme().clone();
        self.ink.step(std::time::Instant::now());
        if self.ink.moving() {
            window.request_animation_frame();
        }
        // The live words close `written`; they sit after the base and the space between.
        let from = written.len() - shown.len();
        let surface = theme.card_strong;
        let highlights: Vec<_> = self
            .ink
            .words()
            .iter()
            .filter(|w| w.alpha < 1.)
            .map(|w| {
                let color = theme.foreground.blend(surface.opacity(1. - w.alpha));
                (from + w.range.start..from + w.range.end, gpui_kit::HighlightStyle { color: Some(color), ..Default::default() })
            })
            .collect();
        // The box scrolls once it passes its rows; the words go up with it.
        let scrolled = self.text.read(cx).scroll_offset().y;
        Some(
            div()
                .absolute()
                .inset_0()
                .overflow_hidden()
                .child(
                    div()
                        .absolute()
                        .top(scrolled)
                        .left_0()
                        .right_0()
                        .px(px(4.) + gpui_kit::px(EDITOR_PAD_X))
                        .pt(gpui_kit::px(EDITOR_PAD_Y))
                        .text_size(TextSize::Sm.font_size())
                        .line_height(px(24.))
                        .text_color(theme.foreground)
                        .child(crate::glyph_text::GlyphText::new(written).highlights(highlights)),
                )
                .into_any_element(),
        )
    }

    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        self.disabled = disabled;
        if disabled {
            self.menu.set_open(false, cx.reduce_motion());
        }
        cx.notify();
    }

    pub fn text(&self, cx: &App) -> SharedString {
        self.text.read(cx).value()
    }

    pub fn set_text(&mut self, text: impl Into<SharedString>, window: &mut Window, cx: &mut Context<Self>) {
        let text = text.into();
        self.text.update(cx, |t, cx| t.set_value(text, window, cx));
    }

    /// What `/` offers: the agent's commands, atelier's and the project's.
    pub fn set_commands(&mut self, commands: Vec<CommandItem>, cx: &mut Context<Self>) {
        self.commands = commands;
        self.refresh_picking(cx);
        cx.notify();
    }

    /// Whether a skill picked from the list runs at once; by default it waits in the box.
    pub fn set_run_picked_skills(&mut self, run: bool) {
        self.run_picked_skills = run;
    }

    /// Whether what is pasted or dropped goes to the owner ([`PromptInputEvent::Paste`]) rather than into the text:
    /// text, images and files alike. Off by default, when text pastes into the box and the rest is ignored.
    pub fn set_paste_chips(&mut self, on: bool) {
        self.paste_chips = on;
    }

    /// A paste, when the owner takes them: images before files before text, as the clipboard offers the most specific thing
    /// first. True when it was taken.
    fn pasted(&mut self, item: &gpui_kit::ClipboardItem, cx: &mut Context<Self>) -> bool {
        if !self.paste_chips || self.disabled {
            return false;
        }
        use gpui_kit::ClipboardEntry;
        let entries = item.entries();
        let found = entries
            .iter()
            .find_map(|e| if let ClipboardEntry::Image(image) = e { Some(Pasted::Image(std::sync::Arc::new(image.clone()))) } else { None })
            .or_else(|| entries.iter().find_map(|e| if let ClipboardEntry::ExternalPaths(paths) = e { Some(Pasted::Files(paths.paths().to_vec())) } else { None }))
            .or_else(|| item.text().filter(|t| !t.is_empty()).map(|t| Pasted::Text(t.into())));
        match found {
            Some(pasted) => {
                cx.emit(PromptInputEvent::Paste(pasted));
                true
            }
            None => false,
        }
    }

    /// What `/` offers now.
    pub fn commands(&self) -> &[CommandItem] {
        &self.commands
    }

    /// The chips on the next message.
    pub fn chips(&self) -> &[Chip] {
        &self.chips
    }

    /// Puts a chip over the text. One with the id of a chip already there is not added again.
    pub fn add_chip(&mut self, chip: Chip, cx: &mut Context<Self>) {
        if self.chips.iter().all(|c| c.id != chip.id) {
            self.chips.push(chip);
            cx.notify();
        }
    }

    /// Backspace in an empty box: the last chip comes off, or else the command in front. False when there is nothing to take
    /// or the box has words (the key is then the text's own).
    fn take_last_chip(&mut self, cx: &mut Context<Self>) -> bool {
        let t = self.text.read(cx);
        if !t.value().is_empty() {
            return false;
        }
        if self.chips.pop().is_some() || self.command.take().is_some() {
            cx.notify();
            return true;
        }
        false
    }

    /// Takes the chip with this id off.
    pub fn remove_chip(&mut self, id: &str, cx: &mut Context<Self>) {
        let before = self.chips.len();
        self.chips.retain(|c| c.id != id);
        if self.chips.len() != before {
            cx.notify();
        }
    }

    /// What `@` offers now.
    pub fn files(&self) -> &[SharedString] {
        &self.files
    }

    /// What `@` offers: the project's files, by their paths.
    pub fn set_files(&mut self, files: Vec<SharedString>, cx: &mut Context<Self>) {
        self.files = files;
        self.refresh_picking(cx);
        cx.notify();
    }

    /// Opens, filters or closes the list from the text and the caret.
    fn refresh_picking(&mut self, cx: &mut Context<Self>) {
        let (text, cursor) = {
            let t = self.text.read(cx);
            (t.value(), t.cursor())
        };
        let matches = match trigger(&text, cursor) {
            Some(Trigger::Command { query }) if !self.commands.is_empty() => Some((Trigger::Command { query: query.clone() }, ranked(&query, &self.commands))),
            Some(Trigger::Mention { start, query }) if !self.files.is_empty() => {
                let found = if query.is_empty() {
                    (0..self.files.len().min(50)).collect()
                } else {
                    crate::fuzzy::rank(&query, self.files.iter().map(|f| f.as_ref()), 50)
                };
                Some((Trigger::Mention { start, query }, found))
            }
            _ => None,
        };
        self.picking = matches.filter(|(_, m)| !m.is_empty()).map(|(trigger, matches)| Picking { trigger, matches, active: 0 });
    }

    /// Fills the box and puts the caret at byte `caret`, as a pick does; the list closes.
    fn write(&mut self, text: &str, caret: usize, window: &mut Window, cx: &mut Context<Self>) {
        let before = &text[..caret];
        let line = before.matches('\n').count() as u32;
        let character = before.rsplit('\n').next().map_or(0, |last| last.encode_utf16().count()) as u32;
        self.text.update(cx, |t, cx| {
            t.set_value(text.to_string(), window, cx);
            t.set_cursor_position(Position::new(line, character), window, cx);
        });
        self.picking = None;
    }

    /// Moves the row in front of the open list by `by`, round the ends; false when no list is open.
    fn step_pick(&mut self, by: isize, cx: &mut Context<Self>) -> bool {
        let Some(picking) = &mut self.picking else { return false };
        let rows = picking.matches.len() as isize;
        picking.active = (picking.active as isize + by).rem_euclid(rows) as usize;
        cx.notify();
        true
    }

    /// Runs or writes the row in front of the open list.
    fn pick(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(picking) = self.picking.take() else { return };
        let Some(&index) = picking.matches.get(picking.active) else { return };
        match picking.trigger {
            Trigger::Command { .. } => {
                let command = self.commands[index].clone();
                let waits = command.source == CommandSource::Skill && !self.run_picked_skills;
                if command.args_hint.is_some() || waits {
                    // It waits as a chip in front of the words, not as text in them.
                    self.write("", 0, window, cx);
                    self.command = Some(command.name);
                } else {
                    self.text.update(cx, |t, cx| t.set_value("", window, cx));
                    cx.emit(PromptInputEvent::Command { name: command.name, args: SharedString::default() });
                }
            }
            Trigger::Mention { start, .. } => {
                let (text, cursor) = {
                    let t = self.text.read(cx);
                    (t.value(), t.cursor())
                };
                let written = format!("{}{}", &text[..start], &text[cursor..]);
                self.write(&written, start, window, cx);
                let path = self.files[index].clone();
                self.add_chip(Chip::file(path), cx);
            }
        }
        cx.notify();
    }

    /// Sends the text when there is some. A running turn does not block a new message; the panel queues it.
    /// With a list open, Enter takes its row. A known `/` command typed out is a command, not a message.
    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.send(Sending::Now, window, cx)
    }

    /// Gives up what the box holds: a command at once, a message as `sending` says.
    fn send(&mut self, sending: Sending, window: &mut Window, cx: &mut Context<Self>) {
        if self.picking.is_some() {
            return self.pick(window, cx);
        }
        let text = self.text(cx);
        let text = text.trim();
        if (text.is_empty() && self.chips.is_empty() && self.command.is_none()) || self.disabled || self.dictating() {
            return;
        }
        let mentions = self.chips.iter().filter_map(|c| c.mention.as_deref()).collect::<Vec<_>>().join(" ");
        let with = |words: &str| -> SharedString {
            match (mentions.is_empty(), words.is_empty()) {
                (true, _) => words.to_string().into(),
                (false, true) => mentions.clone().into(),
                (false, false) => format!("{mentions} {words}").into(),
            }
        };
        if let Some(name) = self.command.clone() {
            let args = with(text);
            self.clear(window, cx);
            return cx.emit(PromptInputEvent::Command { name, args });
        }
        if let Some(rest) = text.strip_prefix('/') {
            let (name, args) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
            if self.commands.iter().any(|c| c.name == name) {
                let (name, args): (SharedString, SharedString) = (name.to_string().into(), with(args.trim()));
                self.clear(window, cx);
                return cx.emit(PromptInputEvent::Command { name, args });
            }
        }
        let message = Message { text: with(text), chips: self.chips.clone() };
        self.clear(window, cx);
        self.retarget_send(cx);
        cx.emit(match sending {
            Sending::Now => PromptInputEvent::Submit(message),
            Sending::AfterTurn => PromptInputEvent::Queue(message),
        });
    }

    /// Empties the text and takes the chips off.
    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.text.update(cx, |t, cx| t.set_value("", window, cx));
        self.chips.clear();
        self.command = None;
    }

    /// The chips over the text, each with its picture, its label and a button that takes it off.
    fn chip_row(&self, cx: &mut Context<Self>) -> Option<gpui_kit::AnyElement> {
        if self.chips.is_empty() && self.command.is_none() {
            return None;
        }
        let theme = cx.theme().clone();
        let this = cx.entity().downgrade();
        // The command goes first: it is the beginning of the message.
        let command = self.command.as_ref().map(|name| {
            let mut chip = Chip::new("command", format!("/{name}")).look(ChipLook::Icon(IconName::Command));
            chip.mention = None;
            (chip, true)
        });
        let all = command.into_iter().chain(self.chips.iter().cloned().map(|c| (c, false))).collect::<Vec<_>>();
        Some(div().flex().flex_wrap().gap(px(4.)).px(px(2.)).pb(px(6.)).children(all.into_iter().map(|(chip, is_command)| {
            let (id, remove_id, gone, pressed) = (chip.id.clone(), chip.id.clone(), chip.id.clone(), chip.id.clone());
            let owner = this.clone();
            let presser = this.clone();
            let picture = match &chip.look {
                ChipLook::None => None,
                ChipLook::Icon(name) => Some(Icon::new(*name).size(px(12.)).color(theme.muted_foreground).into_any_element()),
                ChipLook::File(path) => Some(crate::file_icon::FileIcon::file(path).size(px(12.)).into_any_element()),
                ChipLook::Image(image) => Some(
                    <gpui_kit::Img as gpui_kit::StyledImage>::object_fit(gpui_kit::img(image.clone()), gpui_kit::ObjectFit::Cover).flex_none().size(px(16.)).rounded(px(3.)).into_any_element(),
                ),
            };
            div()
                .id(SharedString::from(format!("chip-{id}")))
                .debug_selector(move || format!("chip-{id}"))
                .flex()
                .items_center()
                .gap(px(4.))
                .h(px(24.))
                .pl(px(6.))
                .pr(px(2.))
                .rounded(radius::md())
                .bg(theme.chip_rest)
                .hover(|d| d.bg(theme.chip_hover))
                .text_size(TextSize::Xs.font_size())
                .text_color(theme.foreground)
                .when(!is_command, |d| {
                    d.on_click(move |_, _, cx| {
                        presser.update(cx, |_, cx| cx.emit(PromptInputEvent::ChipPressed(pressed.clone()))).ok();
                    })
                })
                .tooltip(crate::tooltip::Tooltip::text(chip.detail.clone().unwrap_or_else(|| chip.label.clone())))
                .children(picture)
                .child(chip.label.clone())
                .child(
                    div()
                        .id(SharedString::from(format!("chip-remove-{remove_id}")))
                        .debug_selector({
                            let remove_id = remove_id.clone();
                            move || format!("chip-remove-{remove_id}")
                        })
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(18.))
                        .rounded(px(4.))
                        .text_color(theme.muted_foreground)
                        .hover(|d| d.bg(theme.muted_hover()).text_color(theme.foreground))
                        .child(Icon::new(IconName::Close).size(px(12.)))
                        .on_mouse_down(gpui_kit::MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .on_click(move |_, _, cx| {
                            cx.stop_propagation();
                            owner
                                .update(cx, |p, cx| {
                                    if is_command {
                                        p.command = None;
                                        cx.notify();
                                    } else {
                                        p.remove_chip(&gone, cx);
                                    }
                                })
                                .ok();
                        }),
                )
        })).into_any_element())
    }

    /// The open list, over or under the box, as a Select's list: the elevation's fill and shadow.
    pub(super) fn picker(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let picking = self.picking.as_ref()?;
        let theme = cx.theme().clone();
        let entries: Vec<ComboEntry> = picking
            .matches
            .iter()
            .map(|&i| match &picking.trigger {
                Trigger::Command { .. } => {
                    let c = &self.commands[i];
                    ComboRow::new(format!("/{}", c.name))
                        .detail(format!("{} · {}", c.summary, c.source.words()))
                        .debug_name(format!("command-row-{}", c.name))
                        .into()
                }
                Trigger::Mention { .. } => {
                    let path = &self.files[i];
                    ComboRow::new(path.clone())
                        .leading(crate::file_icon::FileIcon::file(path))
                        .debug_name(format!("file-row-{path}"))
                        .into()
                }
            })
            .collect();
        let rows = entries.len() as f32;
        let height = (2. * PICK_PAD + rows * PICK_ROW + (rows - 1.).max(0.) * PICK_GAP).min(PICK_MOST);
        let width = self.frame.map_or(px(360.), |f| f.size.width);
        let this = cx.entity().downgrade();
        let (hover, choose, close) = (this.clone(), this.clone(), this);
        let level = crate::design_preview::elevation();
        let panel = div()
            .debug_selector(|| "prompt-picker".into())
            .w(width)
            .h(px(height))
            .rounded(px(12.))
            .overflow_hidden()
            .bg(crate::design_preview::panel_fill(&theme, level, theme.card))
            .shadow(crate::design_preview::panel_shadows(&theme, level, crate::theme::popover_shadow(&theme)))
            .child(
                ComboList::new("prompt-picker-list", entries)
                    .style(ComboStyle::Task)
                    .checks(false)
                    .active(Some(picking.active))
                    .on_hover(move |i, _, cx| {
                        hover
                            .update(cx, |p, cx| {
                                if let Some(picking) = &mut p.picking {
                                    picking.active = i;
                                    cx.notify();
                                }
                            })
                            .ok();
                    })
                    .on_pick(move |i, window, cx| {
                        choose
                            .update(cx, |p, cx| {
                                if let Some(picking) = &mut p.picking {
                                    picking.active = i;
                                }
                                p.pick(window, cx);
                            })
                            .ok();
                    }),
            );
        Some(
            Popover::new("prompt-picker-popover")
                .open(true)
                .anchor(self.frame)
                // The box sits at the foot of its panel: the list always opens above it, never over the status line.
                .side(Side::Above)
                .gap(4.)
                .height(height)
                .width(width)
                .keep_focus()
                .on_close(move |_, cx| {
                    close
                        .update(cx, |p, cx| {
                            p.picking = None;
                            cx.notify();
                        })
                        .ok();
                })
                .child(panel),
        )
    }
}

impl EventEmitter<PromptInputEvent> for PromptInput {}

impl Focusable for PromptInput {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.text.focus_handle(cx)
    }
}

impl Render for PromptInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let disabled = self.disabled;
        let empty = self.text(cx).trim().is_empty() && self.chips.is_empty() && self.command.is_none();
        let can_submit = !empty && !disabled && !self.dictating();
        let stops = self.stops(cx);

        let reduce = cx.reduce_motion();
        if self.menu.rotate.is_running()
            || self.send_swap.is_running()
            || self.mic_swap.is_running()
            || self.voice_fade.is_running()
            || (self.voice == VoiceMode::Listening && !reduce)
        {
            window.request_animation_frame();
        }

        // The send/stop icon slot: both icons render, blended by `t` so neither ever pops.
        let t = self.send_swap.value();
        let swap = div()
            .relative()
            .size(px(16.))
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .top(px(-3. * t))
                    .opacity(1. - t)
                    .child(Icon::new(IconName::ArrowUp).size(px(16.))),
            )
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .top(px(3. * (1. - t)))
                    .opacity(t)
                    // The glyph fills half its box, so at 24 it is a 12px square: the stop reads as large as the arrow.
                    .child(Icon::new(IconName::Stop).size(px(24.))),
            );
        let send = {
            let this = cx.entity().downgrade();
            let button = Button::new("prompt-send")
                .content(swap)
                .pill(true)
                .size(ButtonSize::Icon)
                .disabled(!stops && !can_submit)
                .on_click(move |_, window, cx| {
                    this.update(cx, |this, cx| {
                        if this.stops(cx) {
                            cx.emit(PromptInputEvent::Stop);
                        } else {
                            this.submit(window, cx);
                        }
                    })
                    .ok();
                });
            let steers = self.running && !stops;
            let button = button.debug_name("prompt-send");
            if steers { button.tooltip(STEER_HINT) } else { button }
        };

        // The Plus trigger and its menu, shown only when there is something to add: as beui does.
        let add = (!self.actions.is_empty()).then(|| {
            let rotate = self.menu.rotate.value();
            let open = self.menu.open;
            let this = cx.entity().downgrade();
            let plus = div().relative().size(px(16.)).flex().items_center().justify_center().child(
                Icon::new(IconName::Add).size(px(16.)).turn(rotate / 360.),
            );
            let hover = this.clone();
            let trigger = div()
                .id("prompt-add-hover")
                .on_hover(move |on, _, cx| {
                    hover.update(cx, |this, _| this.menu.trigger_hovered = *on).ok();
                })
                .child(Button::new("prompt-add")
                .content(plus)
                .pill(true)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Icon)
                .disabled(disabled || self.running)
                .on_click({
                    let this = this.clone();
                    move |_, _, cx| {
                        this.update(cx, |this, cx| {
                            let reduce = cx.reduce_motion();
                            this.menu.set_open(!this.menu.open, reduce);
                            cx.notify();
                        })
                        .ok();
                    }
                }));
            let menu = open.then(|| {
                let text_focus = self.text.focus_handle(cx);
                let items = self.actions.iter().cloned().map(|action| {
                    let (this, value) = (this.clone(), action.value.clone());
                    let item = MenuItem::new(action.label).on_select(move |_, cx| {
                        this.update(cx, |_, cx| {
                            cx.emit(PromptInputEvent::Action(value.clone()));
                            cx.notify();
                        })
                        .ok();
                    });
                    let item = match action.icon {
                        Some(icon) => item.icon(icon),
                        None => item,
                    };
                    Entry::from(match action.description {
                        Some(words) => item.description(words),
                        None => item,
                    })
                });
                let close = this.clone();
                let done = this.clone();
                let focus_back = text_focus.clone();
                let panel = Menu::new("prompt-actions-menu", items).look(crate::menu::MenuLook::PROMPT).origin(Origin::BottomLeft).on_dismiss(move |window, cx| {
                    done.update(cx, |this, cx| {
                        this.menu.set_open(false, cx.reduce_motion());
                        cx.notify();
                    })
                    .ok();
                    window.focus(&focus_back, cx);
                });
                Popover::new("prompt-actions-popover")
                    .open(true)
                    .hang(Hang::Left(0., 0.))
                    .side(Side::Auto)
                    .gap(8.)
                    .height(220.)
                    .return_focus(&text_focus)
                    .on_close(move |_, cx| {
                        close
                            .update(cx, |this, cx| {
                                this.menu.set_open(false, cx.reduce_motion());
                                cx.notify();
                            })
                            .ok();
                    })
                    .child(panel)
            });
            div().id("prompt-add-wrap").relative().child(trigger).children(menu)
        });

        let model_select = (!self.models.is_empty()).then(|| {
            let this = cx.entity().downgrade();
            div().flex_none().max_w(px(208.)).debug_selector(|| "prompt-model-select".into()).child(
                Select::new("prompt-model", self.models.iter())
                    .selected(Some(self.model))
                    .placeholder("Choose model")
                    .disabled(disabled || self.running)
                    .compact(true)
                    .chevron(false)
                    .shadow(false)
                    .panel_width(px(208.))
                    .on_change(move |i, _, cx| {
                        this.update(cx, |this, cx| {
                            if let Some(model) = this.models.get(i) {
                                this.model = i;
                                cx.emit(PromptInputEvent::ModelChanged(model.value.clone()));
                                cx.notify();
                            }
                        })
                        .ok();
                    }),
            )
        });

        let mode_select = (!self.modes.is_empty()).then(|| {
            let this = cx.entity().downgrade();
            div().flex_none().max_w(px(180.)).debug_selector(|| "prompt-mode-select".into()).child(
                Select::new("prompt-mode", self.modes.iter().cloned())
                    .selected(Some(self.mode))
                    .disabled(disabled)
                    .compact(true)
                    .chevron(false)
                    .shadow(false)
                    .panel_width(px(200.))
                    .on_change(move |i, _, cx| {
                        this.update(cx, |this, cx| {
                            if let Some(mode) = this.modes.get(i) {
                                this.mode = i;
                                cx.emit(PromptInputEvent::ModeChanged(mode.clone()));
                                cx.notify();
                            }
                        })
                        .ok();
                    }),
            )
        });

        // The left of the row: Plus, the model and the mode, which cross-fade with the setup or the bars while dictation runs.
        let fade = self.voice_fade.value().clamp(0., 1.);
        let seconds = self.voice_since.map_or(0., |s| s.elapsed().as_secs_f32());
        let (face, phase, total_mb, level) = (self.voice_face, self.voice_phase, self.voice_total_mb, self.voice_level);
        let (muted, error, themed) = (theme.muted_foreground, self.voice_error.clone(), theme.clone());
        let discarding = cx.entity().downgrade();
        let said = Morph::new("prompt-voice-said", voice_input::key(face), move |_, _| match face {
            VoiceMode::Listening => voice_input::listening_row(level, seconds, muted),
            VoiceMode::Failed => voice_input::failed_row(error.clone(), &themed),
            _ => {
                let discarding = discarding.clone();
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .child(div().min_w_0().child(VoiceSetup::new("prompt-voice-setup", phase).total_mb(total_mb)))
                    .child(
                        Button::new("prompt-voice-discard")
                            .icon(IconName::Close)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip("Discard what you said")
                            .debug_name("prompt-voice-discard")
                            .on_click(move |_, _, cx| {
                                discarding.update(cx, |p, cx| p.discard_waiting(cx)).ok();
                            }),
                    )
                    .into_any_element()
            }
        });
        let left = div()
            .relative()
            .flex_1()
            .min_w_0()
            .min_h(px(32.))
            .flex()
            .items_center()
            .when(fade < 0.999, |d| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.))
                        .opacity(1. - fade)
                        .children(add)
                        .children(model_select)
                        .children(mode_select),
                )
            })
            .when(fade > 0.001, |d| {
                d.child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .pl(px(8.))
                        .top(px(3. * (1. - fade)))
                        .opacity(fade)
                        // Nothing under the bars hears a press while they show.
                        .occlude()
                        .child(div().w_full().child(said)),
                )
            });

        let mic = self.dictation.then(|| {
            let this = cx.entity().downgrade();
            let (hold, mode, blocked) = (self.voice_hold, self.voice, disabled || self.running);
            let mic = voice_input::Mic {
                id: "prompt-mic",
                mode,
                swap: self.mic_swap.value(),
                seconds,
                blocked,
                theme: theme.clone(),
                reduce,
            };
            let click = this.clone();
            // A tap starts and a second tap stops it; with hold on, the press itself records and letting go stops.
            let on_click: crate::ClickHandler = Rc::new(move |_, _, cx| {
                click
                    .update(cx, |this, cx| {
                        if this.voice_hold {
                            return;
                        }
                        if this.voice == VoiceMode::Listening {
                            this.release_mic(cx);
                        } else {
                            this.press_mic(cx);
                        }
                    })
                    .ok();
            });
            let (press, release) = (this.clone(), this.clone());
            // Held down, it records; let go, it stops.
            let hold_down = move |_: &mut Window, cx: &mut App| {
                press
                    .update(cx, |this, cx| {
                        if this.voice != VoiceMode::Listening {
                            this.press_mic(cx);
                            this.held = this.voice == VoiceMode::Listening;
                        }
                    })
                    .ok();
            };
            let hold_up = move |_: &mut Window, cx: &mut App| {
                release
                    .update(cx, |this, cx| {
                        if this.held {
                            this.release_mic(cx);
                        }
                    })
                    .ok();
            };
            // While it listens (and while the stop square swaps in or out) the microphone is the amber disc with its ring;
            // otherwise it is the first segment of the group.
            let listening = mode == VoiceMode::Listening || self.mic_swap.value() > 0.001;
            let first = if listening {
                let click = on_click.clone();
                let (down, up) = (hold_down.clone(), hold_up.clone());
                let slot = voice_input::mic_slot(mic, move |e, w, cx| click(e, w, cx)).debug_selector(|| "prompt-mic".into());
                let slot = if hold {
                    slot.on_mouse_down(gpui_kit::MouseButton::Left, {
                        let down = down.clone();
                        move |_, window, cx| down(window, cx)
                    })
                    .on_mouse_up(gpui_kit::MouseButton::Left, {
                        let up = up.clone();
                        move |_, window, cx| up(window, cx)
                    })
                    .on_mouse_up_out(gpui_kit::MouseButton::Left, move |_, window, cx| up(window, cx))
                } else {
                    slot
                };
                Some(slot)
            } else {
                None
            };
            let segment = Button::new("prompt-mic")
                .icon(IconName::Mic)
                .disabled(blocked)
                .tooltip("Dictate")
                .debug_name("prompt-mic")
                .on_click(move |e, w, cx| on_click(e, w, cx))
                .when(hold, |b| b.on_hold(hold_down, hold_up));

            // The arrow beside it opens the microphones and the hold switch; both wait while it listens.
            let toggle = this.clone();
            let arrow = Button::new("prompt-mic-menu")
                .icon(IconName::ChevronDown)
                .disabled(blocked || self.dictating())
                .tooltip("Microphone")
                .debug_name("prompt-mic-menu")
                .open(self.mic_menu)
                .on_click(move |_, _, cx| {
                    toggle
                        .update(cx, |this, cx| {
                            this.mic_menu = !this.mic_menu;
                            if this.mic_menu {
                                cx.emit(PromptInputEvent::DictationDevices);
                            }
                            cx.notify();
                        })
                        .ok();
                });
            let menu = self.mic_menu.then(|| {
                let text_focus = self.text.focus_handle(cx);
                let mut entries: Vec<Entry> = vec![Entry::Label("Microphone".into())];
                entries.extend(self.voice_devices.iter().cloned().map(|device| {
                    let (this, id) = (this.clone(), device.id.clone());
                    let chosen = self.voice_device.as_ref() == Some(&device.id);
                    MenuItem::new(device.label)
                        .choice(MenuChoice::Selected(chosen))
                        .debug_name(format!("prompt-mic-device-{}", device.id))
                        .on_select(move |_, cx| {
                            this.update(cx, |this, cx| {
                                this.voice_device = Some(id.clone());
                                cx.emit(PromptInputEvent::DictationDevice(Some(id.clone())));
                                cx.notify();
                            })
                            .ok();
                        })
                        .into()
                }));
                entries.push(Entry::Separator);
                let (this_hold, hold_now) = (this.clone(), self.voice_hold);
                entries.push(
                    MenuItem::new("Hold to record")
                        .choice(MenuChoice::Switch(hold_now))
                        .close_on_select(false)
                        .debug_name("prompt-mic-hold")
                        .on_select(move |_, cx| {
                            this_hold
                                .update(cx, |this, cx| {
                                    this.voice_hold = !hold_now;
                                    cx.emit(PromptInputEvent::DictationHold(!hold_now));
                                    cx.notify();
                                })
                                .ok();
                        })
                        .into(),
                );
                let rows = self.voice_devices.len() + 1;
                let (close, done, focus_back) = (this.clone(), this.clone(), text_focus.clone());
                let panel = Menu::new("prompt-mic-menu-list", entries).look(crate::menu::MenuLook::SELECT).origin(Origin::BottomRight).on_dismiss(move |window, cx| {
                    done.update(cx, |this, cx| {
                        this.mic_menu = false;
                        cx.notify();
                    })
                    .ok();
                    window.focus(&focus_back, cx);
                });
                Popover::new("prompt-mic-popover")
                    .open(true)
                    .hang(Hang::Right(0., 0.))
                    .side(Side::Auto)
                    .gap(8.)
                    .height(crate::menu::height_in(crate::menu::MenuLook::SELECT, rows + 1) + crate::menu::MenuLook::SELECT.group)
                    .return_focus(&text_focus)
                    .on_close(move |_, cx| {
                        close
                            .update(cx, |this, cx| {
                                this.mic_menu = false;
                                cx.notify();
                            })
                            .ok();
                    })
                    .child(panel)
            });
            // One group of two: the microphone and the arrow that opens its menu.
            let control = match first {
                Some(slot) => div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(crate::button_group::SEAM))
                    .child(slot)
                    .child(
                        arrow
                            .variant(ButtonVariant::Tinted)
                            .size(ButtonSize::Icon)
                            .corners(gpui_kit::Corners { top_left: false, top_right: true, bottom_left: false, bottom_right: true })
                            .focusable(true),
                    )
                    .into_any_element(),
                None => ButtonGroup::new("prompt-mic-buttons")
                    .variant(ButtonVariant::Tinted)
                    .size(ButtonSize::Icon)
                    .child(segment)
                    .child(arrow)
                    .into_any_element(),
            };
            div().id("prompt-mic-group").relative().flex_none().child(control).children(menu)
        });

        let meter = self.context.map(|(used, window)| {
            let this = cx.entity().downgrade();
            let open = self.context_open;
            let ring = ContextMeter::new("prompt-context", used, window).tip(!open).on_click({
                let this = this.clone();
                move |_, _, cx| {
                    this.update(cx, |this, cx| {
                        this.context_open = !this.context_open;
                        cx.notify();
                    })
                    .ok();
                }
            });
            let panel = open.then(|| {
                let text_focus = self.text.focus_handle(cx);
                let (close, cross) = (this.clone(), this.clone());
                let rows = self.context_parts.len();
                let usage = ContextUsage::new(used, window).parts(self.context_parts.clone()).on_close(move |_, _, cx| {
                    cross.update(cx, |this, cx| {
                        this.context_open = false;
                        cx.notify();
                    })
                    .ok();
                });
                Popover::new("prompt-context-popover")
                    .open(true)
                    .hang(Hang::Right(0., 0.))
                    .side(Side::Auto)
                    .gap(8.)
                    .height(context_usage::height(rows))
                    .return_focus(&text_focus)
                    .on_close(move |_, cx| {
                        close
                            .update(cx, |this, cx| {
                                this.context_open = false;
                                cx.notify();
                            })
                            .ok();
                    })
                    .child(usage)
            });
            div().id("prompt-context-wrap").relative().flex_none().child(ring).children(panel)
        });
        let toolbar = div().debug_selector(|| "prompt-toolbar".into()).flex().items_center().gap(px(4.)).min_h(px(32.)).child(left).children(meter).children(mic).child(send);

        let this = cx.entity().downgrade();
        let (pasting, dropping, backing) = (this.clone(), this.clone(), this.clone());
        let text = self.text.clone();
        let overlay = self.live_overlay(window, cx);
        let chips = self.chip_row(cx);
        let queued = self.queued_rows(cx);
        let picker = self.picker(cx);

        let (keys, measured, down, up) = (this.clone(), this.clone(), this.clone(), this.clone());
        div()
            .debug_selector(|| "prompt-frame".into())
            .relative()
            // With a list open, its keys are its own before the text sees them.
            .capture_key_down(move |event, window, cx| {
                keys.update(cx, |p, cx| {
                    if p.picking.is_none() { return }
                    match event.keystroke.key.as_str() {
                        "tab" => p.pick(window, cx),
                        "escape" => p.picking = None,
                        _ => return,
                    }
                    cx.stop_propagation();
                    cx.notify();
                })
                .ok();
            })
            // The text binds Up and Down to its caret as actions, which a key listener never sees first.
            // Backspace in an empty box takes off the last chip, then the command in front: the text binds it as an action too.
            .capture_action(move |_: &Backspace, _, cx| {
                if backing.update(cx, |p, cx| p.take_last_chip(cx)).unwrap_or(false) {
                    cx.stop_propagation();
                }
            })
            .capture_action(move |_: &MoveDown, _, cx| {
                if down.update(cx, |p, cx| p.step_pick(1, cx)).unwrap_or(false) {
                    cx.stop_propagation();
                }
            })
            .capture_action(move |_: &MoveUp, _, cx| {
                if up.update(cx, |p, cx| p.step_pick(-1, cx)).unwrap_or(false) {
                    cx.stop_propagation();
                }
            })
            .child(crate::placement::measure(move |bounds, cx| {
                measured
                    .update(cx, |p, cx| {
                        if p.frame != Some(bounds) {
                            p.frame = Some(bounds);
                            if p.picking.is_some() {
                                cx.notify();
                            }
                        }
                    })
                    .ok();
            }))
            .children(picker)
            // Files dropped on the box go to the owner like a paste, when it takes them.
            .on_drop(move |paths: &gpui_kit::ExternalPaths, _, cx| {
                dropping
                    .update(cx, |p, cx| {
                        if p.paste_chips && !p.disabled {
                            cx.emit(PromptInputEvent::Paste(Pasted::Files(paths.paths().to_vec())));
                        }
                    })
                    .ok();
            })
            // A press anywhere in the box writes in it. A picker's own click comes after, and takes the
            // focus it needs.
            .on_mouse_down(gpui_kit::MouseButton::Left, move |_, window, cx| text.update(cx, |t, cx| t.focus(window, cx)))
            .flex()
            .flex_col()
            .p(px(8.))
            .rounded(radius::xxl())
            // The panel it sits in is a card; a surface inside a card is `card_strong`, or the box would not part from it.
            .bg(theme.card_strong)
            .when(disabled, |d| d.opacity(0.6))
            // The menu has no focus of its own, so Escape is caught here: it bubbles up from whichever
            // descendant (the textarea, almost always) currently holds focus.
            .on_key_down(move |event, _, cx| {
                if event.keystroke.key == "escape" {
                    this.update(cx, |this, cx| {
                        if this.menu.open {
                            this.menu.set_open(false, cx.reduce_motion());
                            cx.notify();
                        }
                    })
                    .ok();
                }
            })
            .children(queued)
            .children(chips)
            .child(
                // While words come in, the words are drawn over the box (see `live_overlay`) and the box itself is
                // not seen; it stays where it is, so the caret, the focus and the height are the same as ever.
                // The field keeps its own padding; the row crops it (see `TRIM_Y`) so the text row is as high as the
                // controls' row, and the box has the same room above the text as below the controls.
                div().debug_selector(|| "prompt-text".into()).overflow_hidden().child(
                    div()
                        .relative()
                        .my(px(-TRIM_Y))
                        .child(
                            div().when(overlay.is_some(), |d| d.opacity(0.)).child(
                                Textarea::new(&self.text)
                                    .on_paste(move |item, _, cx| pasting.update(cx, |p, cx| p.pasted(item, cx)).unwrap_or(false))
                                    .appearance(false)
                                    .disabled(disabled)
                                    .px(px(4.))
                                    .text_size(TextSize::Sm.font_size())
                                    .line_height(px(24.)),
                            ),
                        )
                        .children(overlay),
                ),
            )
            .child(toolbar)
    }
}
