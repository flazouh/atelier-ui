//! The one-time setup of dictation: the first press of the microphone has to fetch the speech model, so the bar
//! shows a line of words and a segmented amber progress bar instead of a dead button.
//!
//! SPIKE, with [`crate::voice_waves`]. The owner reports the work; this only draws it.
//!
//! - The track is a row of 32 small cells, each a rounded rectangle. A cell is dark, lit, or in between at the head.
//! - Download: the head cell fills in step with the number, and the few cells behind it burn hotter, near white, so the
//!   edge reads as a charge arriving. A pale ripple runs through the lit cells, and a faint pulse travels out ahead of the
//!   head over the dark ones, as if the bar were scanning for the rest. The number eases, so a jumpy download looks like
//!   one smooth pour.
//! - Prepare: the model is on disk and loads (the first run compiles shaders, a few seconds, with no number to give). A
//!   short bright cluster with a trail sweeps back and forth across the cells.
//! - Ready: every cell is lit, the ripple slows, and the words turn to `Ready` with a check.
//! - Words at the left, the percent in mono at the right.
//! - Reduce Motion: the fill jumps to the number; there is no ripple, no scan and no sweep.
use std::time::Instant;

use gpui_kit::{App, ElementId, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div, hsla, prelude::FluentBuilder};

use crate::{
    icon::{Icon, IconName},
    scale::px,
    theme::ActiveTheme,
    typography::{MONO_FONT_FAMILY, TextSize},
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

/// How many cells the track has, how tall each is and the space between, in pixels.
pub const CELLS: usize = 32;
pub const CELL_HEIGHT: f32 = 8.;
pub const CELL_GAP: f32 = 2.;
/// How long the bright cluster takes to cross the cells and come back, in seconds.
pub const SWEEP_SECONDS: f32 = 1.5;
/// How many cells behind the head burn hot.
const HOT_REACH: f32 = 3.;
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

/// How lit one cell is and how hot (near white) it burns, both 0 to 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub lit: f32,
    pub hot: f32,
}

/// Where the bright cluster is, in cells, `seconds` in: out to the last cell and back, slowing at both turns.
pub fn sweep_at(seconds: f32, cells: usize) -> f32 {
    let cycle = (seconds / SWEEP_SECONDS).rem_euclid(2.);
    let leg = if cycle < 1. { cycle } else { 2. - cycle };
    smoothstep(leg) * (cells.saturating_sub(1)) as f32
}

/// Cell `i` of `cells` when the bar is `fill` full (`None` while there is no number), `seconds` in. With `moving` off the
/// ripple, the scan and the sweep are left out and the cluster rests in the middle.
pub fn cell(i: usize, cells: usize, fill: Option<f32>, seconds: f32, moving: bool) -> Cell {
    let at = i as f32;
    let Some(fill) = fill else {
        let head = if moving { sweep_at(seconds, cells) } else { (cells as f32 - 1.) / 2. };
        let near = (1. - (at - head).abs() / 4.5).max(0.);
        // The trail is the cluster seen a little earlier, so it stretches behind the way it moves.
        let lit = near.powf(1.6);
        return Cell { lit, hot: lit * lit };
    };
    let head = fill.clamp(0., 1.) * cells as f32;
    let full = fill >= 1.;
    let ripple = if moving { ((seconds * if full { 2.5 } else { 5. } - at * 0.35).sin() * 0.5 + 0.5) * 0.22 } else { 0.11 };
    if at + 1. <= head + 1e-4 {
        // Lit. The few just behind the head burn hotter.
        let behind = head - at - 1.;
        let hot = if full { 0. } else { (1. - behind / HOT_REACH).max(0.) * 0.85 };
        Cell { lit: 0.78 + ripple, hot }
    } else if at < head {
        // The head cell, part way: it comes up as the number does.
        Cell { lit: (head - at) * 0.95, hot: 1. }
    } else {
        // Dark, with a faint pulse running out ahead of the head.
        let ahead = at - head;
        let reach = (cells as f32 - head).max(1.) + 6.;
        let pulse = if moving { (seconds * 16.) % reach } else { -10. };
        let bump = (-(ahead - pulse).powi(2) / 3.).exp() * 0.4;
        Cell { lit: bump, hot: 0. }
    }
}

/// The color of a cell: dark amber to full amber by `lit`, then toward white by `hot`.
pub fn cell_color(base: Hsla, cell: Cell) -> Hsla {
    let alpha = 0.1 + 0.9 * cell.lit.clamp(0., 1.);
    hsla(base.h, base.s * (1. - 0.6 * cell.hot), base.l + (0.97 - base.l) * 0.8 * cell.hot, alpha)
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
        let cells = (0..CELLS).map(|i| {
            let c = cell(i, CELLS, target.map(|_| shown), seconds, !reduce);
            div().flex_1().h(px(CELL_HEIGHT)).rounded(px(2.)).bg(cell_color(base, c))
        });
        let track = div().w_full().flex().items_center().gap(px(CELL_GAP)).children(cells);

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(8.))
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
                    .when_some(percent, |d, percent| d.child(div().flex_none().font_family(MONO_FONT_FAMILY).text_color(theme.foreground).child(SharedString::from(percent)))),
            )
            .child(track)
    }
}

#[cfg(test)]
mod tests;
