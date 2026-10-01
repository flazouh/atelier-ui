//! The status row shown while the agent works: the agent's mark, then the label and its segments. The
//! mark, the label's colours, and its words come from an [`AgentLook`].
//!
//! It follows the Claude desktop app's status line (`M5` in `c1c1ec7b9-BpMPRL5q.js`): the label follows
//! the phase and, while thinking, the elapsed seconds; every label swap runs [`Morph`]; segments sit
//! after the label with a 12px gap, and no separator glyph. The text shimmer is the Claude Code CLI's glimmer (2.1.283, `Uyt` and its spinner
//! hook): the label in the look's message colour with three clusters in its glimmer colour walking across it, one per 200ms, then
//! leaving the text for a 10 cluster pad. [`Shimmer::Stepped`] is that exact look; [`Shimmer::Smooth`]
//! moves a soft band with the same center, speed, and colors. [`ThinkingStyle::Breath`] is the desktop
//! app's opacity breath instead. The glimmer and band math live in [`crate::glimmer`].
//!
//! While subagents run, the mark orbits. Each subagent gets its own [`SubagentRow`], shown above the
//! composer.
//!
//! Under Reduce Motion nothing glimmers or breathes, labels swap at once, and the mark stands still.

use std::time::{Duration, Instant};

use gpui_kit::{
    App, ElementId, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, StyledText, Window, div,
    prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::{
    agent_look::{AgentLook, Mark, PhaseLabels},
    glimmer,
    morph::Morph,
    motion::{BREATH_LOW, Channel, Curve, duration, ease, keyframes},
    sprite::Strip,
    theme::ActiveTheme,
    typography::{SEGMENT_GAP, TextSize},
    wake::Wake,
};

/// What the agent is doing. The label follows from it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThinkingPhase {
    /// Thinking since `since`. The label changes at 15, 30, 45, and 60 seconds.
    Thinking { since: Instant },
    /// Thinking has ended: "Thought for {seconds}s", with no shimmer.
    Thought { seconds: u64 },
    Connecting,
    Sending,
    Starting,
    Preparing,
    Waiting,
    RunningTools,
}

impl ThinkingPhase {
    /// The CLI's `requesting` mode: the glimmer walks left to right, four times as fast.
    fn requesting(self) -> bool {
        matches!(self, Self::Connecting | Self::Sending | Self::Starting | Self::Preparing | Self::Waiting)
    }

    fn elapsed_s(self) -> f32 {
        match self {
            Self::Thinking { since } => since.elapsed().as_secs_f32(),
            _ => 0.,
        }
    }
}

/// How the label moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThinkingStyle {
    /// The CLI's glyph glimmer.
    Shimmer(Shimmer),
    /// The desktop app's opacity breath: 1 to 0.75 and back over 2s, after 3s.
    Breath,
}

impl Default for ThinkingStyle {
    fn default() -> Self {
        Self::Shimmer(Shimmer::default())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Shimmer {
    /// A soft band that glides between clusters.
    #[default]
    Smooth,
    /// The CLI exactly: three whole clusters, stepping.
    Stepped,
}

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

#[derive(IntoElement)]
pub struct Thinking {
    id: ElementId,
    look: AgentLook,
    phase: ThinkingPhase,
    style: ThinkingStyle,
    elapsed: Option<SharedString>,
    tokens: Option<u64>,
    tasks: Option<u64>,
    subagents: usize,
    loading: Vec<Strip>,
}

impl Thinking {
    pub fn new(id: impl Into<ElementId>, look: AgentLook, phase: ThinkingPhase) -> Self {
        Self {
            id: id.into(),
            look,
            phase,
            style: ThinkingStyle::default(),
            elapsed: None,
            tokens: None,
            tasks: None,
            subagents: 0,
            loading: Vec::new(),
        }
    }

    /// The strips the loading mark picks among. Each row picks one at random when it first shows and keeps
    /// it for its whole run. Without any, the look's working strip plays. Orbiting still takes over while
    /// subagents run.
    pub fn loading(mut self, strips: Vec<Strip>) -> Self {
        self.loading = strips;
        self
    }

    /// How many subagents run now. While any run, the mark orbits.
    pub fn subagents(mut self, running: usize) -> Self {
        self.subagents = running;
        self
    }

    pub fn style(mut self, style: ThinkingStyle) -> Self {
        self.style = style;
        self
    }

    /// How long the agent has worked, such as "12s".
    pub fn elapsed(mut self, elapsed: impl Into<SharedString>) -> Self {
        self.elapsed = Some(elapsed.into());
        self
    }

    pub fn tokens(mut self, tokens: u64) -> Self {
        self.tokens = Some(tokens);
        self
    }

    pub fn tasks(mut self, tasks: u64) -> Self {
        self.tasks = Some(tasks);
        self
    }
}

/// What sits between segments in text form, for tests and accessibility: nothing but the gap.
pub(crate) const SEGMENT_GAP_TEXT: &str = "";

/// A segment as shown: its own text, with no separator glyph.
pub(crate) fn segment_text(segment: &str) -> SharedString {
    SharedString::from(format!("{SEGMENT_GAP_TEXT}{segment}"))
}

/// The mark's strip: orbiting while subagents run, else working. A finished turn keeps the working strip,
/// standing still.
pub fn mark_strip(mark: &Mark, done: bool, subagents: usize) -> Strip {
    if !done && subagents > 0 { mark.orbiting } else { mark.working }
}

/// The strip the loading mark plays: one of `variants` chosen by `roll`, or `working` when there are none.
pub(crate) fn loading_strip(variants: &[Strip], working: Strip, roll: u64) -> Strip {
    if variants.is_empty() { working } else { variants[(roll % variants.len() as u64) as usize] }
}

/// A fresh random number, from the standard library's randomly seeded hasher. Nothing here needs more.
fn roll() -> u64 {
    use std::hash::BuildHasher;
    std::collections::hash_map::RandomState::new().hash_one(0u8)
}

/// When the row appeared (the glimmer's clock), when the label last changed (the breath's clock), and
/// the next timed wake-up.
struct RowMotion {
    start: Instant,
    /// Picks the loading strip, once per row.
    roll: u64,
    label: SharedString,
    label_start: Instant,
    wake: Wake,
    /// Each segment's width growing from 0 when it first shows.
    grow: [Option<Channel>; 3],
}

/// The CSS keyframes `0%, to {opacity: 1} 50% {opacity: .75}`, `2s ease-in-out 3s infinite`.
fn breath_opacity(since_label_ms: u64) -> f32 {
    let delay = duration::BREATH_DELAY.as_millis() as u64;
    if since_label_ms < delay {
        return 1.;
    }
    let period = duration::BREATH.as_millis() as u64;
    let t = ((since_label_ms - delay) % period) as f32 / 1000.;
    keyframes(&[1., BREATH_LOW, 1.], &[0., 0.5, 1.], period as f32 / 1000., ease::BREATH, t)
}

impl RenderOnce for Thinking {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduce = cx.reduce_motion();
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let phase = self.phase;
        let elapsed_s = phase.elapsed_s();
        let text = label(&self.look.labels, phase, elapsed_s);
        let width = glimmer::cluster_count(&text);
        let requesting = phase.requesting();
        let done = matches!(phase, ThinkingPhase::Thought { .. });

        let motion = window.use_keyed_state(self.id.clone(), cx, {
            let text = text.clone();
            move |_, _| RowMotion {
                start: Instant::now(),
                roll: roll(),
                label: text,
                label_start: Instant::now(),
                wake: Wake::default(),
                grow: [None, None, None],
            }
        });
        let segments: [Option<SharedString>; 3] = [self.elapsed, self.tokens.map(tokens_text), self.tasks.map(tasks_text)];
        let (elapsed_ms, since_label_ms, grow, picked) = motion.update(cx, |m, _| {
            if m.label != text {
                m.label = text.clone();
                m.label_start = Instant::now();
            }
            let grow: [Option<f32>; 3] = std::array::from_fn(|i| {
                let slot = &mut m.grow[i];
                if segments[i].is_none() {
                    *slot = None;
                    return None;
                }
                let channel = slot.get_or_insert_with(|| {
                    let mut c = Channel::new(0.);
                    c.animate(1., Curve::Ease(duration::MORPH.as_secs_f32(), ease::MORPH), 0., reduce);
                    c
                });
                channel.is_running().then(|| channel.value())
            });
            (m.start.elapsed().as_millis() as u64, m.label_start.elapsed().as_millis() as u64, grow, m.roll)
        });

        // When the next change is due. The thinking breath and the smooth glimmer band would
        // otherwise ask for every display frame; capping their wake to REPAINT_CAP (30fps) is plenty
        // for an opacity fade or a band gliding a few px, and it lets several rows share one redraw.
        let mut next: Option<Duration> = next_label_change_s(&self.look.labels, phase, elapsed_s).map(Duration::from_secs_f32);
        let (message, glimmer_color) = (self.look.message, self.look.glimmer);
        let mut soonest = |wait: Duration| next = Some(next.map_or(wait, |n| n.min(wait)));
        let mut opacity = 1.;
        let highlights = match self.style {
            _ if done => None,
            // The CLI sets the index to -100 under Reduce Motion, so nothing lights.
            ThinkingStyle::Shimmer(_) if reduce => Some(glimmer::glimmer_highlights(&text, message, glimmer_color, |_| 0.)),
            ThinkingStyle::Breath if reduce => None,
            ThinkingStyle::Shimmer(Shimmer::Stepped) => {
                let index = glimmer::glimmer_index(elapsed_ms, width, requesting);
                let step = glimmer::step_ms(requesting);
                soonest(Duration::from_millis(step - elapsed_ms % step));
                Some(glimmer::glimmer_highlights(&text, message, glimmer_color, |g| {
                    if glimmer::stepped_lit(g, index) { 1. } else { 0. }
                }))
            }
            ThinkingStyle::Shimmer(Shimmer::Smooth) => {
                match glimmer::band_wait_ms(elapsed_ms, width, requesting) {
                    0 => soonest(duration::REPAINT_CAP),
                    wait => soonest(Duration::from_millis(wait)),
                }
                let center = glimmer::glimmer_center(elapsed_ms, width, requesting);
                Some(glimmer::glimmer_highlights(&text, message, glimmer_color, |g| glimmer::glimmer_weight(g, center)))
            }
            ThinkingStyle::Breath => {
                let delay = duration::BREATH_DELAY.as_millis() as u64;
                if since_label_ms < delay {
                    soonest(Duration::from_millis(delay - since_label_ms));
                } else {
                    soonest(duration::REPAINT_CAP);
                }
                opacity = breath_opacity(since_label_ms);
                None
            }
        };
        if grow.iter().any(Option::is_some) {
            window.request_animation_frame();
        }
        motion.update(cx, |m, cx| match next {
            Some(wait) => m.wake.at(Instant::now() + wait, cx),
            None => m.wake.cancel(),
        });

        let label_color = label_color(&self.look, !done && highlights.is_some(), muted);
        let label_key = text.clone();
        let label_el = move |_: &mut Window, _: &mut App| {
            let styled = StyledText::new(text.clone());
            match &highlights {
                Some(h) => styled.with_highlights(h.clone()).into_any_element(),
                None => styled.into_any_element(),
            }
        };
        let child = |name: &'static str| ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), name.into());
        let segment_names = ["elapsed", "tokens", "tasks"];
        let segment_style = {
            let mut style = window.text_style();
            style.font_size = TextSize::Xs.font_size().into();
            style
        };

        div()
            .flex()
            .items_center()
            .gap(px(16.))
            .h(px(28.))
            .text_size(TextSize::Xs.font_size())
            .line_height(TextSize::Xs.line_height())
            .text_color(muted)
            // The spark plays while the agent thinks. Once it has, the row is only its words: a still mark would
            // still read as something loading.
            .children((!done).then(|| {
                let mark = &self.look.mark;
                let strip = match mark_strip(mark, done, self.subagents) {
                    working if working == mark.working => loading_strip(&self.loading, working, picked),
                    orbiting => orbiting,
                };
                mark.sprite(child("spark"), strip).size(px(18.)).playing(true)
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(SEGMENT_GAP))
                    .whitespace_nowrap()
                    .child(div().opacity(opacity).text_color(label_color).child(Morph::new(child("label"), label_key, label_el)))
                    .children(segments.into_iter().zip(segment_names).zip(grow).filter_map(|((segment, name), grow)| {
                        let segment = segment_text(&segment?);
                        // A new segment grows from no width to its own over the morph's 180ms.
                        let width = grow.map(|p| {
                            let run = segment_style.to_run(segment.len());
                            let full = window.text_system().shape_line(segment.clone(), TextSize::Xs.font_size(), &[run], None).width;
                            full * p
                        });
                        let key = segment.clone();
                        Some(
                            div()
                                .flex_none()
                                .whitespace_nowrap()
                                .when_some(width, |d, w| d.w(w).overflow_hidden())
                                .child(Morph::new(child(name), key, move |_, _| segment.clone().into_any_element()).enter(true)),
                        )
                    })),
            )
    }
}

#[cfg(test)]
mod tests;
