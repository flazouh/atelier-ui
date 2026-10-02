//! The one-time setup of dictation: the first press of the microphone has to fetch the speech model, so the bar
//! shows a line of words and a thin amber progress bar instead of a dead button.
//!
//! SPIKE, with [`crate::voice_waves`]. The owner reports the work; this only draws it.
//!
//! - Download: the words say what is coming and how much (`Downloading speech model · 69 of 164 MB`), the percent
//!   sits at the right, and the bar fills to it. The fill eases toward each new number, so a jumpy download looks
//!   like one smooth pour instead of steps.
//! - Prepare: the model is on disk and loads (the first run compiles shaders, a few seconds, with no number to
//!   give). A short amber segment sweeps back and forth across the track instead of a fraction.
//! - Ready: the bar is full, the words turn to `Ready` with a check.
//! - A soft light travels along the filled part, so even a stalled download shows that the app is alive.
//! - Reduce Motion: the fill jumps to the number and holds, with no sweep and no light.
use std::time::Instant;

use gpui_kit::{
    App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div, hsla, linear_color_stop, linear_gradient,
    prelude::FluentBuilder, relative,
};

use crate::{
    icon::{Icon, IconName},
    scale::px,
    theme::ActiveTheme,
    typography::TextSize,
    voice_waves::amber,
};

/// How far along the setup is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SetupPhase {
    /// The model is coming down: 0 to 1.
    Download(f32),
    /// The model is on disk and loads; there is no number to show.
    Prepare,
    Ready,
}

/// The height of the track in pixels.
pub const TRACK_HEIGHT: f32 = 5.;
/// How long the light takes to cross the fill, and the sweep to cross the track, in seconds.
pub const SHEEN_SECONDS: f32 = 1.6;
pub const SWEEP_SECONDS: f32 = 1.5;
/// How wide the sweep is, as a fraction of the track.
pub const SWEEP_WIDTH: f32 = 0.34;
/// How fast the fill eases toward its number: per second.
const POUR_RATE: f32 = 9.;

/// How full the bar is for `phase`, or `None` while there is no number.
pub fn fraction(phase: SetupPhase) -> Option<f32> {
    match phase {
        SetupPhase::Download(done) => Some(if done.is_nan() { 0. } else { done.clamp(0., 1.) }),
        SetupPhase::Prepare => None,
        SetupPhase::Ready => Some(1.),
    }
}

/// The words at the left and the percent at the right, for a model of `total_mb`.
pub fn copy(phase: SetupPhase, total_mb: f32) -> (String, Option<String>) {
    match phase {
        SetupPhase::Download(_) => {
            let done = fraction(phase).unwrap_or(0.);
            let words = format!("Downloading speech model · {:.0} of {:.0} MB", done * total_mb, total_mb);
            (words, Some(format!("{:.0}%", done * 100.)))
        }
        SetupPhase::Prepare => ("Getting it ready for this Mac…".to_string(), None),
        SetupPhase::Ready => ("Ready".to_string(), None),
    }
}

/// The fill after `dt` seconds toward `target`.
pub fn pour(current: f32, target: f32, dt: f32) -> f32 {
    current + (target - current) * (1. - (-POUR_RATE * dt).exp())
}

fn smoothstep(x: f32) -> f32 {
    let x = x.clamp(0., 1.);
    x * x * (3. - 2. * x)
}

/// Where the left edge of the light is, as a fraction of the fill, `seconds` in: it starts off to the left and
/// leaves off to the right, then starts again.
pub fn sheen_left(seconds: f32) -> f32 {
    -0.45 + 1.45 * (seconds / SHEEN_SECONDS).fract()
}

/// Where the left edge of the sweep is, as a fraction of the track, `seconds` in: out to the right and back, slowing at
/// both turns, and never leaving the track.
pub fn sweep_left(seconds: f32) -> f32 {
    let cycle = (seconds / SWEEP_SECONDS).rem_euclid(2.);
    let leg = if cycle < 1. { cycle } else { 2. - cycle };
    smoothstep(leg) * (1. - SWEEP_WIDTH)
}

struct SetupState {
    shown: f32,
    at: Instant,
    born: Instant,
}

#[derive(IntoElement)]
pub struct VoiceSetup {
    id: ElementId,
    phase: SetupPhase,
    total_mb: f32,
}

impl VoiceSetup {
    pub fn new(id: impl Into<ElementId>, phase: SetupPhase) -> Self {
        Self { id: id.into(), phase, total_mb: 164. }
    }

    /// The size of the download, for the words.
    pub fn total_mb(mut self, total_mb: f32) -> Self {
        self.total_mb = total_mb;
        self
    }
}

impl RenderOnce for VoiceSetup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let target = fraction(self.phase);
        let state = window.use_keyed_state(self.id, cx, |_, _| {
            let now = Instant::now();
            SetupState { shown: 0., at: now, born: now }
        });
        let (shown, seconds) = state.update(cx, |s, _| {
            let now = Instant::now();
            let dt = now.duration_since(s.at).as_secs_f32().min(0.1);
            s.at = now;
            let want = target.unwrap_or(0.);
            s.shown = if reduce { want } else { pour(s.shown, want, dt) };
            (s.shown, now.duration_since(s.born).as_secs_f32())
        });
        let settled = matches!(self.phase, SetupPhase::Ready) && (1. - shown) < 1e-3;
        if !reduce && !settled {
            window.request_animation_frame();
        }

        let (words, percent) = copy(self.phase, self.total_mb);
        let ready = matches!(self.phase, SetupPhase::Ready);
        let base = amber();
        let lighter = hsla(base.h, base.s, 0.68, 1.);
        let fill = linear_gradient(90., linear_color_stop(lighter, 0.), linear_color_stop(base, 1.));
        let light = |a: f32| gpui_kit::white().opacity(a);
        // A gradient takes two stops, so the light is a rising half and a falling half side by side.
        let glow = |left: f32| {
            div()
                .absolute()
                .top_0()
                .bottom_0()
                .left(relative(left))
                .w(relative(0.45))
                .flex()
                .child(div().flex_1().bg(linear_gradient(90., linear_color_stop(light(0.), 0.), linear_color_stop(light(0.5), 1.))))
                .child(div().flex_1().bg(linear_gradient(90., linear_color_stop(light(0.5), 0.), linear_color_stop(light(0.), 1.))))
        };

        let track = div().relative().w_full().h(px(TRACK_HEIGHT)).overflow_hidden().rounded_full().bg(theme.card_strong).map(|track| {
            match target {
                Some(_) => track.child(
                    div()
                        .relative()
                        .h_full()
                        .w(relative(shown))
                        .rounded_full()
                        .overflow_hidden()
                        .bg(fill)
                        .when(!reduce, |d| d.child(glow(sheen_left(seconds)))),
                ),
                None => track.child(
                    div()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .left(relative(if reduce { (1. - SWEEP_WIDTH) / 2. } else { sweep_left(seconds) }))
                        .w(relative(SWEEP_WIDTH))
                        .rounded_full()
                        .bg(fill),
                ),
            }
        });

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(7.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(12.))
                    .text_size(TextSize::Xs.font_size())
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .min_w_0()
                            .text_color(if ready { theme.foreground } else { theme.muted_foreground })
                            .when(ready, |d| d.child(Icon::new(IconName::Check).size(px(14.)).color(theme.success)))
                            .child(SharedString::from(words)),
                    )
                    .when_some(percent, |d, percent| d.child(div().flex_none().text_color(theme.foreground).child(SharedString::from(percent)))),
            )
            .child(track)
    }
}

#[cfg(test)]
mod tests;
