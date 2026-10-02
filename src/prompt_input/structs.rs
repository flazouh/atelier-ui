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
    component::input::{InputEvent, MoveDown, MoveUp, Position, Textarea, TextareaState},
    div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    combobox::{ComboEntry, ComboList, ComboRow, ComboStyle},
    command_item::{CommandItem, Trigger, mention, ranked, trigger},
    icon::{Icon, IconName},
    menu::{Entry, Menu, MenuItem, Origin},
    morph::Morph,
    motion::{Channel, Curve, Spring, ease},
    popover::{Hang, Popover, Side},
    select::Select,
    theme::{ActiveTheme, radius},
    typography::TextSize,
    voice_input::{self, VoiceMode},
    voice_setup::{SetupPhase, VoiceSetup},
};
use super::types::{PICK_GAP, PICK_MOST, PICK_PAD, PICK_ROW, PromptInputEvent};
use super::helpers::append_transcript;

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
    pub(super) voice_since: Option<Instant>,
    /// 0 shows the microphone, 1 the stop square.
    pub(super) mic_swap: Channel,
    /// 0 shows Plus, the model and the mode; 1 shows the setup or the bars.
    pub(super) voice_fade: Channel,
    pub(super) running: bool,
    pub(super) disabled: bool,
    /// What `/` offers, and `@` ([`crate::command_item`]).
    pub(super) commands: Vec<CommandItem>,
    pub(super) files: Vec<SharedString>,
    picking: Option<Picking>,
    /// The box, as last drawn: the list opens from it.
    pub(super) frame: Option<Bounds<Pixels>>,
    _subscription: Subscription,
}

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
                .auto_grow(2, 8)
                .submit_on_enter(true)
                .placeholder(placeholder)
                .default_value(default_value)
        });
        let subscription = cx.subscribe_in(&text, window, |this, _, event: &InputEvent, window, cx| match event {
            // Enter sends, and ⌘↵ (⌃↵ elsewhere) too, as the brief's key.
            InputEvent::PressEnter { shift: false, .. } => this.submit(window, cx),
            // Send turns on and off with the text, so redraw on every edit; a `/` or an `@` opens a list.
            InputEvent::Change => {
                this.refresh_picking(cx);
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
            voice_since: None,
            mic_swap: Channel::new(0.),
            voice_fade: Channel::new(0.),
            running: false,
            disabled: false,
            commands: Vec::new(),
            files: Vec::new(),
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

    /// Shows Stop instead of Send while the agent works.
    pub fn set_running(&mut self, running: bool, cx: &mut Context<Self>) {
        self.running = running;
        let reduce = cx.reduce_motion();
        self.send_swap.animate(if running { 1. } else { 0. }, Curve::Spring(Spring::SWAP), 0., reduce);
        // The Plus trigger disables while running, so its menu cannot stay open behind it.
        if running {
            self.menu.set_open(false, reduce);
        }
        cx.notify();
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

    /// Whether dictation holds the box: it is being set up or it listens. Send waits for it.
    pub(super) fn dictating(&self) -> bool {
        matches!(self.voice, VoiceMode::Setup | VoiceMode::Listening)
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
            // The menu cannot stay open behind the bars that cover its button.
            self.menu.set_open(false, reduce);
        }
        self.voice = mode;
        self.mic_swap.animate(if listening { 1. } else { 0. }, Curve::Spring(Spring::SWAP), 0., reduce);
        self.voice_fade.animate(if mode == VoiceMode::Idle { 0. } else { 1. }, Curve::Ease(0.22, ease::OUT), 0., reduce);
        cx.notify();
    }

    /// Writes what was said at the end of the text, after a space when the text does not end in one, and puts the caret after
    /// it, so the user reads it, fixes it and sends it.
    pub fn insert_transcript(&mut self, words: &str, window: &mut Window, cx: &mut Context<Self>) {
        let words = words.trim();
        if words.is_empty() {
            return;
        }
        let written = append_transcript(&self.text(cx), words);
        self.write(&written, written.len(), window, cx);
        cx.notify();
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

    /// What `/` offers now.
    pub fn commands(&self) -> &[CommandItem] {
        &self.commands
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
                if command.args_hint.is_some() {
                    let written = format!("/{} ", command.name);
                    self.write(&written, written.len(), window, cx);
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
                let (written, caret) = mention(&text, start, cursor, &self.files[index]);
                self.write(&written, caret, window, cx);
            }
        }
        cx.notify();
    }

    /// Sends the text when there is some. A running turn does not block a new message; the panel queues it.
    /// With a list open, Enter takes its row. A known `/` command typed out is a command, not a message.
    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.picking.is_some() {
            return self.pick(window, cx);
        }
        let text = self.text(cx);
        let text: SharedString = text.trim().to_string().into();
        if text.is_empty() || self.disabled || self.dictating() {
            return;
        }
        if let Some(rest) = text.strip_prefix('/') {
            let (name, args) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
            if self.commands.iter().any(|c| c.name == name) {
                let (name, args): (SharedString, SharedString) = (name.to_string().into(), args.trim().to_string().into());
                self.text.update(cx, |t, cx| t.set_value("", window, cx));
                return cx.emit(PromptInputEvent::Command { name, args });
            }
        }
        self.text.update(cx, |t, cx| t.set_value("", window, cx));
        cx.emit(PromptInputEvent::Submit(text));
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
                    ComboRow::new(path.clone()).debug_name(format!("file-row-{path}")).into()
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
        let empty = self.text(cx).trim().is_empty();
        let can_submit = !empty && !disabled && !self.running && !self.dictating();

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
            Button::new("prompt-send")
                .content(swap)
                .pill(true)
                .size(ButtonSize::Icon)
                .disabled(if self.running { false } else { !can_submit })
                .on_click(move |_, window, cx| {
                    this.update(cx, |this, cx| {
                        if this.running {
                            cx.emit(PromptInputEvent::Stop);
                        } else {
                            this.submit(window, cx);
                        }
                    })
                    .ok();
                })
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
        let said = Morph::new("prompt-voice-said", voice_input::key(face), move |_, _| match face {
            VoiceMode::Listening => voice_input::listening_row(level, seconds, muted),
            VoiceMode::Failed => voice_input::failed_row(error.clone(), &themed),
            _ => div().w_full().child(VoiceSetup::new("prompt-voice-setup", phase).total_mb(total_mb)).into_any_element(),
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
            let mic = voice_input::Mic {
                id: "prompt-mic",
                mode: self.voice,
                swap: self.mic_swap.value(),
                seconds,
                blocked: disabled || self.running,
                theme: theme.clone(),
                reduce,
            };
            voice_input::mic_slot(
                mic,
                move |_, _, cx| {
                    this.update(cx, |this, cx| match this.voice {
                        VoiceMode::Idle | VoiceMode::Failed => cx.emit(PromptInputEvent::DictationStart),
                        VoiceMode::Listening => cx.emit(PromptInputEvent::DictationStop),
                        VoiceMode::Setup => {}
                    })
                    .ok();
                },
            )
            .debug_selector(|| "prompt-mic".into())
        });

        let toolbar = div().flex().items_center().gap(px(4.)).min_h(px(32.)).mt(px(4.)).child(left).children(mic).child(send);

        let this = cx.entity().downgrade();
        let text = self.text.clone();
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
            .child(
                Textarea::new(&self.text)
                    .appearance(false)
                    .disabled(disabled)
                    .px(px(8.))
                    .pt(px(6.))
                    .text_size(TextSize::Sm.font_size())
                    .line_height(px(24.)),
            )
            .child(toolbar)
    }
}
