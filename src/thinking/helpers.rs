use gpui_kit::{Hsla, SharedString};

use crate::{
    agent_look::{AgentLook, Mark, PhaseLabels},
    motion::{BREATH_LOW, duration, ease, keyframes},
    sprite::Strip,
    };
use super::types::{SEGMENT_GAP_TEXT, ThinkingPhase};

pub fn label(labels: &PhaseLabels, phase: ThinkingPhase, elapsed_s: f32) -> SharedString {
    match phase {
        ThinkingPhase::Thinking { .. } => {
            let thresholds = labels.thinking;
            let text = thresholds.iter().rev().find(|(at, _)| elapsed_s >= *at).or(thresholds.first()).map_or("", |(_, text)| text);
            SharedString::new_static(text)
        }
        ThinkingPhase::Thought { seconds } => format!("Thought for {seconds}s").into(),
        ThinkingPhase::Connecting => labels.connecting.clone(),
        ThinkingPhase::Sending => labels.sending.clone(),
        ThinkingPhase::Starting => labels.starting.clone(),
        ThinkingPhase::Preparing => labels.preparing.clone(),
        ThinkingPhase::Waiting => labels.waiting.clone(),
        ThinkingPhase::RunningTools => labels.running_tools.clone(),
    }
}

/// Seconds until [`label`] changes, or `None` if it never will.
pub fn next_label_change_s(labels: &PhaseLabels, phase: ThinkingPhase, elapsed_s: f32) -> Option<f32> {
    match phase {
        ThinkingPhase::Thinking { .. } => {
            labels.thinking.iter().map(|(at, _)| *at).find(|at| *at > elapsed_s).map(|at| at - elapsed_s)
        }
        _ => None,
    }
}

/// The label's colour: the look's message colour while it glimmers, else muted.
pub(crate) fn label_color(look: &AgentLook, moving: bool, muted: Hsla) -> Hsla {
    if moving { look.message } else { muted }
}

pub fn tokens_text(count: u64) -> SharedString {
    match count {
        1 => "1 token".into(),
        0..1000 => format!("{count} tokens").into(),
        _ => {
            let k = format!("{:.1}", count as f64 / 1000.);
            format!("{}k tokens", k.trim_end_matches(".0")).into()
        }
    }
}

pub fn tasks_text(count: u64) -> SharedString {
    if count == 1 { "1 task".into() } else { format!("{count} tasks").into() }
}

/// A segment as shown: its own text, with no separator glyph.
pub(crate) fn segment_text(segment: &str) -> SharedString {
    SharedString::from(format!("{SEGMENT_GAP_TEXT}{segment}"))
}

/// The mark's strip: orbiting while subagents run, else working. A finished turn keeps the working strip,
/// standing still.
pub fn mark_strip(mark: &Mark, done: bool, subagents: usize) -> Strip {
    if !done && subagents > 0 { mark.orbiting } else { mark.working }
}

/// The CSS keyframes `0%, to {opacity: 1} 50% {opacity: .75}`, `2s ease-in-out 3s infinite`.
pub(super) fn breath_opacity(since_label_ms: u64) -> f32 {
    let delay = duration::BREATH_DELAY.as_millis() as u64;
    if since_label_ms < delay {
        return 1.;
    }
    let period = duration::BREATH.as_millis() as u64;
    let t = ((since_label_ms - delay) % period) as f32 / 1000.;
    keyframes(&[1., BREATH_LOW, 1.], &[0., 0.5, 1.], period as f32 / 1000., ease::BREATH, t)
}
