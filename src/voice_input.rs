//! The whole dictation bar: a microphone that turns into a stop button, with amber bars beside it while it listens.
//!
//! SPIKE, with [`crate::voice_waves`] and [`crate::voice_setup`]. The owner runs the microphone and the model; this is
//! the face of it, and it says what the user did.
//!
//! - Idle: a hint at the left and a round microphone at the right.
//! - First press: the owner finds the speech model missing and calls [`VoiceInput::set_setup`]. The hint morphs into
//!   the setup bar ([`VoiceSetup`]), which the owner feeds with the download, and the microphone waits.
//! - Listening: the setup (or the hint) morphs into rounded amber bars that follow the voice, with the time recorded at
//!   their right. The microphone cross-fades into a stop square on an amber disc, the way the prompt input swaps send
//!   for stop (`Spring::SWAP`, the outgoing glyph rising 3px as it fades, the incoming one falling in), and a soft ring
//!   breathes out from the disc.
//! - Stop: all of it runs back. The bars morph into the hint, the stop square into the microphone.
//! - Reduce Motion: the glyphs swap at once, the ring is left out and the bars hold still.
//!
//! The owner plays the sound cues on [`VoiceInputEvent`]s, before it changes the mode, so the cue lands with the press.
use std::time::Instant;

use gpui_kit::{
    AnyElement, App, Context, Div, EventEmitter, Hsla, InteractiveElement, IntoElement, ParentElement, Render, SharedString, Styled, Window, div,
    prelude::FluentBuilder,
};

use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    icon::{Icon, IconName},
    morph::Morph,
    motion::{Channel, Curve, Spring},
    scale::px,
    theme::{ActiveTheme, Theme, radius},
    typography::TextSize,
    voice_setup::{SetupPhase, VoiceSetup},
    voice_waves::{VoiceWaves, amber_for, on_amber},
};

/// The bar's height and the round button's size, in pixels.
pub const BAR_HEIGHT: f32 = 44.;
pub const BUTTON: f32 = 28.;
/// How far the ring grows past the disc, in pixels, and how long one breath takes, in seconds.
pub const RING_REACH: f32 = 9.;
pub const RING_SECONDS: f32 = 1.4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoiceMode {
    Idle,
    /// The speech model is being fetched or loaded; see [`VoiceInput::set_setup`].
    Setup,
    Listening,
    /// The last press ended without words; the bar says why until the owner sets another mode. The microphone can be pressed.
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoiceInputEvent {
    /// The user pressed the microphone while idle.
    Start,
    /// The user pressed the stop square while listening.
    Stop,
}

/// `m:ss` for a recording that has run `seconds`.
pub fn clock(seconds: u64) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// How far into a breath of the ring it is, 0 to 1, `seconds` after listening began.
pub fn ring_phase(seconds: f32) -> f32 {
    (seconds / RING_SECONDS).fract()
}

/// The ring's opacity and its reach in pixels at `phase`, scaled by how far the swap has gone (`swap`, 0 to 1): it starts
/// strong at the disc and thins out as it grows, and it does not show before the stop square does.
pub fn ring_at(phase: f32, swap: f32) -> (f32, f32) {
    let eased = 1. - (1. - phase).powi(2);
    (0.5 * (1. - phase) * swap.clamp(0., 1.), RING_REACH * eased)
}

pub struct VoiceInput {
    mode: VoiceMode,
    phase: SetupPhase,
    total_mb: f32,
    level: f32,
    since: Option<Instant>,
    /// 0 shows the microphone, 1 the stop square.
    swap: Channel,
}

impl EventEmitter<VoiceInputEvent> for VoiceInput {}

impl Default for VoiceInput {
    fn default() -> Self {
        Self::new()
    }
}

impl VoiceInput {
    pub fn new() -> Self {
        Self { mode: VoiceMode::Idle, phase: SetupPhase::Prepare, total_mb: 164., level: 0., since: None, swap: Channel::new(0.) }
    }

    pub fn mode(&self) -> VoiceMode {
        self.mode
    }

    /// The size of the model being fetched, for the setup's words.
    pub fn set_total_mb(&mut self, total_mb: f32, cx: &mut Context<Self>) {
        self.total_mb = total_mb;
        cx.notify();
    }

    fn go(&mut self, mode: VoiceMode, cx: &mut Context<Self>) {
        let listening = mode == VoiceMode::Listening;
        if listening && self.mode != VoiceMode::Listening {
            self.since = Some(Instant::now());
        }
        if !listening {
            self.level = 0.;
        }
        self.mode = mode;
        let reduce = cx.reduce_motion();
        self.swap.animate(if listening { 1. } else { 0. }, Curve::Spring(Spring::SWAP), 0., reduce);
        cx.notify();
    }

    pub fn set_idle(&mut self, cx: &mut Context<Self>) {
        self.go(VoiceMode::Idle, cx);
    }

    /// Shows the first-use setup at `phase`; call again as it moves.
    pub fn set_setup(&mut self, phase: SetupPhase, cx: &mut Context<Self>) {
        self.phase = phase;
        self.go(VoiceMode::Setup, cx);
    }

    pub fn set_listening(&mut self, cx: &mut Context<Self>) {
        self.go(VoiceMode::Listening, cx);
    }

    /// The microphone's level now, 0 to 1.
    pub fn set_level(&mut self, level: f32, cx: &mut Context<Self>) {
        self.level = level;
        cx.notify();
    }
}

pub(crate) fn key(mode: VoiceMode) -> &'static str {
    match mode {
        VoiceMode::Idle => "idle",
        VoiceMode::Setup => "setup",
        VoiceMode::Listening => "listening",
        VoiceMode::Failed => "failed",
    }
}

/// The words for a press that ended without any: a warning mark and the reason, in the warning color.
pub(crate) fn failed_row(message: SharedString, theme: &Theme) -> AnyElement {
    div()
        .w_full()
        .flex()
        .items_center()
        .gap(px(6.))
        .text_size(TextSize::Xs.font_size())
        .text_color(theme.warning)
        .child(div().flex_none().child(Icon::new(IconName::Warning).size(px(14.)).color(theme.warning)))
        .child(div().min_w_0().overflow_hidden().text_ellipsis().whitespace_nowrap().child(message))
        .into_any_element()
}

/// The bars and the time beside them: what the bar says while it listens.
pub(crate) fn listening_row(level: f32, seconds: f32, muted: Hsla) -> AnyElement {
    div()
        .w_full()
        .flex()
        .items_center()
        .gap(px(12.))
        .child(div().flex_1().min_w_0().child(VoiceWaves::new("voice-input-waves").level(level).height(px(26.)).bars(44)))
        .child(
            div()
                .flex_none()
                .w(px(34.))
                .text_size(TextSize::Xs.font_size())
                .text_color(muted)
                .child(SharedString::from(clock(seconds as u64))),
        )
        .into_any_element()
}

/// The round button that is a microphone while idle and a stop square on an amber disc while it listens, with the ring
/// breathing out from the disc. `swap` is 0 for the microphone and 1 for the stop square; `seconds` is how long it has
/// listened. Pressing it calls `on_click`; it cannot be pressed while `mode` is [`VoiceMode::Setup`] or when `blocked`.
pub(crate) struct Mic {
    pub id: &'static str,
    pub mode: VoiceMode,
    pub swap: f32,
    pub seconds: f32,
    pub blocked: bool,
    pub theme: Theme,
    pub reduce: bool,
}

pub(crate) fn mic_slot(mic: Mic, on_click: impl Fn(&gpui_kit::ClickEvent, &mut Window, &mut App) + 'static) -> Div {
    let Mic { id, mode, swap, seconds, blocked, theme, reduce } = mic;
    let (tone, foreground) = (amber_for(&theme), theme.foreground);
    // The microphone and the stop square share one slot, blended by `swap` so neither ever pops.
    let ink = on_amber(&theme);
    let glyphs = div()
        .relative()
        .size(px(16.))
        .child(
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .top(px(-3. * swap))
                .opacity(1. - swap)
                .child(Icon::new(IconName::Mic).size(px(16.)).color(foreground)),
        )
        .child(
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .top(px(3. * (1. - swap)))
                .opacity(swap)
                .child(Icon::new(IconName::Stop).size(px(24.)).color(ink)),
        );
    let (ring_opacity, ring_reach) = if reduce { (0., 0.) } else { ring_at(ring_phase(seconds), swap) };
    let button = Button::new(id)
        .content(glyphs)
        .pill(true)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Icon)
        .disabled(blocked || mode == VoiceMode::Setup)
        .tooltip(if mode == VoiceMode::Listening { "Stop" } else { "Dictate" })
        .on_click(on_click);
    div()
        .relative()
        .flex_none()
        .size(px(BUTTON))
        // The disc fills with amber as the square comes in.
        .child(div().absolute().inset_0().rounded_full().bg(tone.opacity(swap)))
        .when(ring_opacity > 0.01, |d| {
            d.child(
                div()
                    .absolute()
                    .top(px(-ring_reach))
                    .left(px(-ring_reach))
                    .size(px(BUTTON + 2. * ring_reach))
                    .rounded_full()
                    .border_1()
                    .border_color(tone.opacity(ring_opacity)),
            )
        })
        .child(button)
}

impl Render for VoiceInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let t = self.swap.value();
        let seconds = self.since.map_or(0., |s| s.elapsed().as_secs_f32());
        if self.swap.is_running() || (self.mode == VoiceMode::Listening && !reduce) {
            window.request_animation_frame();
        }

        // What the bar says, by mode. The closure is drawn again for the outgoing mode while it fades, so it holds its own
        // copy of everything it shows.
        let (mode, phase, total_mb, level) = (self.mode, self.phase, self.total_mb, self.level);
        let muted = theme.muted_foreground;
        let said = Morph::new("voice-input-said", key(mode), move |_, _| -> AnyElement {
            match mode {
                VoiceMode::Idle | VoiceMode::Failed => div()
                    .w_full()
                    .text_size(TextSize::Xs.font_size() + gpui_kit::px(1.))
                    .text_color(muted)
                    .child(SharedString::from("Press the microphone to dictate"))
                    .into_any_element(),
                VoiceMode::Setup => div().w_full().child(VoiceSetup::new("voice-input-setup", phase).total_mb(total_mb)).into_any_element(),
                VoiceMode::Listening => listening_row(level, seconds, muted),
            }
        });

        let this = cx.entity().downgrade();
        let mic = Mic { id: "voice-input-button", mode: self.mode, swap: t, seconds, blocked: false, theme: theme.clone(), reduce };
        let slot = mic_slot(mic, move |_, _, cx| {
            this.update(cx, |this, cx| match this.mode {
                VoiceMode::Idle | VoiceMode::Failed => cx.emit(VoiceInputEvent::Start),
                VoiceMode::Listening => cx.emit(VoiceInputEvent::Stop),
                VoiceMode::Setup => {}
            })
            .ok();
        });

        div()
            .id("voice-input")
            .w_full()
            .h(px(BAR_HEIGHT))
            .flex()
            .items_center()
            .gap(px(12.))
            .pl(px(16.))
            .pr(px(8.))
            .rounded(radius::xl())
            .bg(theme.card)
            .child(div().flex_1().min_w_0().flex().items_center().child(div().w_full().child(said)))
            .child(slot)
    }
}

#[cfg(test)]
mod tests;
