use std::{rc::Rc, time::Instant};

use gpui_kit::{
    AnyElement, App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::FluentBuilder,
};

use super::helpers::{frame, morph_curve, on_key_change};
use super::types::RenderChild;
use crate::motion::Channel;
use crate::scale::px;

#[derive(IntoElement)]
pub struct Morph {
    id: ElementId,
    pub(super) key: SharedString,
    pub(super) child: RenderChild,
    pub(super) enter: bool,
}

impl Morph {
    /// `child` draws the content for `key`. When `key` changes, the last child drawn morphs into this
    /// one.
    pub fn new(
        id: impl Into<ElementId>,
        key: impl Into<SharedString>,
        child: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            key: key.into(),
            child: Rc::new(child),
            enter: false,
        }
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

struct MorphState {
    pub(super) key: SharedString,
    pub(super) current: RenderChild,
    pub(super) old: Option<RenderChild>,
    /// Fades the child in `old`. `None` once settled.
    pub(super) exit: Option<Channel>,
    /// Rises `current` in.
    pub(super) enter: Channel,
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
                MorphState {
                    key,
                    current: child,
                    old: None,
                    exit: None,
                    enter,
                }
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
            (
                m.old.clone(),
                m.exit.as_ref().map(|c| c.value_at(now)),
                m.enter.value_at(now),
                moving,
            )
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
