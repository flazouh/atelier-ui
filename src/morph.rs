//! Swaps one child for another with the Claude app's text morph (`Mr` in `cb6930dae-j25SuahN.js`): the
//! old child rises out 3px and fades while the new one rises in from 3px below, both over 180ms on
//! `cubic-bezier(0.2, 0, 0, 1)`. Like Motion's `mode: "popLayout"`, the old child leaves the layout at
//! once, so both cross at the same time. Under Reduce Motion the new child shows at once.
//!
//! The child is a function, not an element, because GPUI elements live for one frame and the old
//! child must keep drawing after its key has changed.
//!
//! The exit (fading-out) and enter (rising-in) child each run on their own [`Channel`]. A key change
//! while settled starts both fresh, but a key change while a morph is already running leaves the
//! exit channel alone: the child that is fading out keeps its own value and never jumps. Only the
//! child that was rising in gets discarded and replaced; the truly new child restarts the enter
//! channel from nothing.

use std::{rc::Rc, time::Instant};

use gpui_kit::{
    AnyElement, App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::motion::{Channel, Curve, MORPH_RISE, duration, ease};

type RenderChild = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

#[derive(IntoElement)]
pub struct Morph {
    id: ElementId,
    key: SharedString,
    child: RenderChild,
    enter: bool,
}

impl Morph {
    /// `child` draws the content for `key`. When `key` changes, the last child drawn morphs into this
    /// one.
    pub fn new(
        id: impl Into<ElementId>,
        key: impl Into<SharedString>,
        child: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        Self { id: id.into(), key: key.into(), child: Rc::new(child), enter: false }
    }

    /// Also rise in when first shown, for content that is new to the screen.
    pub fn enter(mut self, enter: bool) -> Self {
        self.enter = enter;
        self
    }
}

/// Where both children are at eased progress `p` (0 at the swap, 1 when settled).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MorphFrame {
    pub old_opacity: f32,
    pub old_y: f32,
    pub new_opacity: f32,
    pub new_y: f32,
}

pub fn frame(p: f32) -> MorphFrame {
    MorphFrame { old_opacity: 1. - p, old_y: -MORPH_RISE * p, new_opacity: p, new_y: MORPH_RISE * (1. - p) }
}

fn morph_curve() -> Curve {
    Curve::Ease(duration::MORPH.as_secs_f32(), ease::MORPH)
}

/// What a key change does to the running channels, kept as a pure function so the no-jump behavior
/// is testable without GPUI. `exit` is the channel already fading a previous child, or `None` when
/// the morph is settled.
///
/// - Settled (`exit` is `None`): the child that was current becomes the new exit, fading from full
///   opacity, and a fresh enter channel rises the new child in.
/// - Running (`exit` is `Some`): the exit channel is returned untouched, so the child fading out
///   keeps its exact value and never jumps. Only the enter channel restarts, from nothing.
fn on_key_change(exit: Option<Channel>, reduce_motion: bool, now: Instant) -> (Channel, Channel) {
    let exit = exit.unwrap_or_else(|| {
        let mut c = Channel::new(0.);
        c.animate_at(1., morph_curve(), 0., reduce_motion, now);
        c
    });
    let mut enter = Channel::new(0.);
    enter.animate_at(1., morph_curve(), 0., reduce_motion, now);
    (exit, enter)
}

struct MorphState {
    key: SharedString,
    current: RenderChild,
    old: Option<RenderChild>,
    /// Fades the child in `old`. `None` once settled.
    exit: Option<Channel>,
    /// Rises `current` in.
    enter: Channel,
}

impl RenderOnce for Morph {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduce = cx.reduce_motion();
        let (key, child) = (self.key, self.child);
        let enter_on_show = self.enter;
        let state = window.use_keyed_state(self.id, cx, {
            let (key, child) = (key.clone(), child.clone());
            move |_, _| {
                let mut enter = Channel::new(if enter_on_show { 0. } else { 1. });
                enter.animate(1., morph_curve(), 0., reduce);
                MorphState { key, current: child, old: None, exit: None, enter }
            }
        });
        let (old, exit_p, enter_p, moving) = state.update(cx, |m, _| {
            let now = Instant::now();
            if m.key != key {
                // Only take over as the new `old` when nothing is fading yet; a morph already in
                // flight keeps its exit channel (and the child it draws) exactly as it was.
                let was_settled = m.exit.is_none();
                let (exit, enter) = on_key_change(m.exit.take(), reduce, now);
                if was_settled {
                    m.old = Some(m.current.clone());
                }
                m.current = child.clone();
                m.exit = Some(exit);
                m.enter = enter;
                m.key = key;
            } else {
                m.current = child.clone();
            }
            if let Some(exit) = &m.exit
                && !exit.is_running_at(now)
            {
                m.exit = None;
                m.old = None;
            }
            let moving = m.enter.is_running_at(now) || m.exit.is_some();
            (m.old.clone(), m.exit.as_ref().map(|c| c.value_at(now)), m.enter.value_at(now), moving)
        });
        if moving {
            window.request_animation_frame();
        }
        let enter_frame = frame(enter_p);
        div()
            .relative()
            .child(
                div()
                    .relative()
                    .top(px(enter_frame.new_y))
                    .opacity(enter_frame.new_opacity)
                    .child(child(window, cx)),
            )
            .when_some(old.zip(exit_p), |d, (old, exit_p)| {
                let exit_frame = frame(exit_p);
                d.child(
                    div()
                        .absolute()
                        .left_0()
                        .top(px(exit_frame.old_y))
                        .whitespace_nowrap()
                        .opacity(exit_frame.old_opacity)
                        .child(old(window, cx)),
                )
            })
    }
}

#[cfg(test)]
mod tests;
