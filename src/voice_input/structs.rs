use std::time::Instant;

use gpui_kit::{
    AnyElement, Context, EventEmitter, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};

use super::helpers::{key, listening_row, mic_slot};
use super::types::{BAR_HEIGHT, VoiceInputEvent, VoiceMode};
use crate::{
    morph::Morph,
    motion::{Channel, Curve, Spring},
    scale::px,
    theme::{ActiveTheme, Theme, radius},
    typography::TextSize,
    voice_setup::{SetupPhase, VoiceSetup},
};

pub struct VoiceInput {
    pub(super) mode: VoiceMode,
    pub(super) phase: SetupPhase,
    pub(super) total_mb: f32,
    pub(super) level: f32,
    pub(super) since: Option<Instant>,
    /// 0 shows the microphone, 1 the stop square.
    pub(super) swap: Channel,
}

impl EventEmitter<VoiceInputEvent> for VoiceInput {}

impl Default for VoiceInput {
    fn default() -> Self {
        Self::new()
    }
}

impl VoiceInput {
    pub fn new() -> Self {
        Self {
            mode: VoiceMode::Idle,
            phase: SetupPhase::Prepare,
            total_mb: 164.,
            level: 0.,
            since: None,
            swap: Channel::new(0.),
        }
    }

    pub fn mode(&self) -> VoiceMode {
        self.mode
    }

    /// The size of the model being fetched, for the setup's words.
    pub fn set_total_mb(&mut self, total_mb: f32, cx: &mut Context<Self>) {
        self.total_mb = total_mb;
        cx.notify();
    }

    pub(super) fn go(&mut self, mode: VoiceMode, cx: &mut Context<Self>) {
        let listening = mode == VoiceMode::Listening;
        if listening && self.mode != VoiceMode::Listening {
            self.since = Some(Instant::now());
        }
        if !listening {
            self.level = 0.;
        }
        self.mode = mode;
        let reduce = cx.reduce_motion();
        self.swap.animate(
            if listening { 1. } else { 0. },
            Curve::Spring(Spring::SWAP),
            0.,
            reduce,
        );
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
                VoiceMode::Setup => div()
                    .w_full()
                    .child(VoiceSetup::new("voice-input-setup", phase).total_mb(total_mb))
                    .into_any_element(),
                VoiceMode::Listening => listening_row(level, seconds, muted),
            }
        });

        let this = cx.entity().downgrade();
        let mic = Mic {
            id: "voice-input-button",
            mode: self.mode,
            swap: t,
            seconds,
            blocked: false,
            theme: theme.clone(),
            reduce,
        };
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
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .child(div().w_full().child(said)),
            )
            .child(slot)
    }
}
