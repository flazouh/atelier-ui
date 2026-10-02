//! The one-time setup of dictation: the first press of the microphone has to fetch the speech model, so the bar says so
//! in one quiet line: the words, a small segmented bar, and the size.
//!
//! SPIKE, with [`crate::voice_waves`]. The owner reports the work; this only draws it.
//!
//! - One row that hugs its content: the words in muted text, then 14 small cells (4px wide, 6px tall, 2px apart, 1.5px
//!   corners), then the size (`69 / 164 MB`) in mono. It never stretches across the bar.
//! - Download: cells fill from the left in step with the number; the one at the head fills part way. Lit cells are amber,
//!   the rest a faint amber. The number eases so a jumpy download does not step.
//! - Prepare: the model is on disk and loads, with no number to give. One cell steps along the row, a little slower than
//!   the eye follows, and that is the only motion.
//! - Ready: every cell is lit and the words turn to `Ready` with a check.
//! - Reduce Motion: the fill jumps to the number, and the loading cell rests in the middle.
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

/// How many cells the bar has, and each one's width, height and the space between, in pixels.
pub const CELLS: usize = 14;
pub const CELL_WIDTH: f32 = 4.;
pub const CELL_HEIGHT: f32 = 6.;
pub const CELL_GAP: f32 = 2.;
/// How many cells the loading cell passes in a second.
pub const STEPS_PER_SECOND: f32 = 5.;
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

/// The words, and the size at the right when there is one, for a model of `total_mb`.
pub fn copy(phase: SetupPhase, total_mb: f32) -> (&'static str, Option<String>) {
    match phase {
        SetupPhase::Download(_) => {
            let done = fraction(phase).unwrap_or(0.);
            ("Downloading speech model", Some(format!("{:.0} / {:.0} MB", done * total_mb, total_mb)))
        }
        SetupPhase::Prepare => ("Getting ready", None),
        SetupPhase::Ready => ("Ready", None),
    }
}

/// The fill after `dt` seconds toward `target`.
pub fn pour(current: f32, target: f32, dt: f32) -> f32 {
    current + (target - current) * (1. - (-POUR_RATE * dt).exp())
}

/// How lit cell `i` of `cells` is, 0 to 1, when the bar is `fill` full.
pub fn lit(i: usize, cells: usize, fill: f32) -> f32 {
    (fill.clamp(0., 1.) * cells as f32 - i as f32).clamp(0., 1.)
}

/// The cell the loading light is on `seconds` in: it goes along the row and starts again at the left. With `moving` off it
/// rests in the middle.
pub fn loading_cell(seconds: f32, cells: usize, moving: bool) -> usize {
    if moving { (seconds * STEPS_PER_SECOND) as usize % cells.max(1) } else { cells / 2 }
}

/// The color of a cell: a faint amber when dark, full amber when lit.
pub fn cell_color(base: Hsla, lit: f32) -> Hsla {
    hsla(base.h, base.s, base.l, 0.14 + 0.86 * lit.clamp(0., 1.))
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
        // Frames only while something moves: the loading cell, or a fill still on its way to its number.
        let pouring = target.is_some_and(|t| (t - shown).abs() > 1e-3);
        if !reduce && (target.is_none() || pouring) {
            window.request_animation_frame();
        }

        let (words, size) = copy(self.phase, self.total_mb);
        let ready = matches!(self.phase, SetupPhase::Ready);
        let base = amber();
        let loading = loading_cell(seconds, CELLS, !reduce);
        let cells = (0..CELLS).map(|i| {
            let on = match target {
                Some(_) => lit(i, CELLS, shown),
                None => (i == loading) as u8 as f32,
            };
            div().flex_none().w(px(CELL_WIDTH)).h(px(CELL_HEIGHT)).rounded(px(1.5)).bg(cell_color(base, on))
        });

        div()
            .flex()
            .items_center()
            .gap(px(10.))
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
            .child(div().flex_none().flex().items_center().gap(px(CELL_GAP)).children(cells))
            .when_some(size, |d, size| d.child(div().flex_none().font_family(MONO_FONT_FAMILY).text_color(theme.muted_foreground).child(SharedString::from(size))))
    }
}

#[cfg(test)]
mod tests;
