use std::time::{Duration, Instant};

use gpui_kit::{
    App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled, StyledText,
    Window, div, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    agent_look::{AgentLook},
    glimmer,
    morph::Morph,
    motion::{Channel, Curve, duration, ease},
    theme::ActiveTheme,
    typography::{SEGMENT_GAP, TextSize},
    wake::Wake,
};
use super::types::{Shimmer, ThinkingPhase, ThinkingStyle};
use super::helpers::{
    breath_opacity, label, label_color, mark_strip, next_label_change_s, segment_text,
    tasks_text, tokens_text,
};

#[derive(IntoElement)]
pub struct Thinking {
    pub(super) id: ElementId,
    pub(super) look: AgentLook,
    pub(super) phase: ThinkingPhase,
    pub(super) style: ThinkingStyle,
    pub(super) elapsed: Option<SharedString>,
    pub(super) tokens: Option<u64>,
    pub(super) tasks: Option<u64>,
    pub(super) subagents: usize,
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
        }
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

/// When the row appeared (the glimmer's clock), when the label last changed (the breath's clock), and
/// the next timed wake-up.
pub(super) struct RowMotion {
    pub(super) start: Instant,
    pub(super) label: SharedString,
    pub(super) label_start: Instant,
    pub(super) wake: Wake,
    /// Each segment's width growing from 0 when it first shows.
    pub(super) grow: [Option<Channel>; 3],
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
                label: text,
                label_start: Instant::now(),
                wake: Wake::default(),
                grow: [None, None, None],
            }
        });
        let segments: [Option<SharedString>; 3] = [self.elapsed, self.tokens.map(tokens_text), self.tasks.map(tasks_text)];
        let (elapsed_ms, since_label_ms, grow) = motion.update(cx, |m, _| {
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
            (m.start.elapsed().as_millis() as u64, m.label_start.elapsed().as_millis() as u64, grow)
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
            .children((!done).then(|| self.look.mark.sprite(child("spark"), mark_strip(&self.look.mark, done, self.subagents)).size(px(18.)).playing(true)))
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
