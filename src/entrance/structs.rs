use std::{collections::HashSet, sync::Arc, time::Duration};

use gpui_kit::{
    AnyElement, App, Div, ElementId, IntoElement, ParentElement, RenderOnce, Styled, Window,
    div,
};

use crate::scale::px;
use crate::motion::Channel;
use super::helpers::{curve, frame, stagger_delay};

/// Where an entering item is at eased progress `p` (0 when it arrives, 1 when settled).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EntranceFrame {
    pub opacity: f32,
    pub y: f32,
}

/// Which ids a list has shown. The first call only records what is there.
#[derive(Default)]
pub struct Arrivals {
    pub(super) painted: bool,
    pub(super) seen: HashSet<ElementId>,
}

impl Arrivals {
    /// The entrance delay for each of `ids`, or `None` for an item that must not animate.
    pub fn arrive(&mut self, ids: &[ElementId]) -> Vec<Option<Duration>> {
        let first = !std::mem::replace(&mut self.painted, true);
        let fresh: Vec<bool> = ids.iter().map(|id| self.seen.insert(id.clone()) && !first).collect();
        let count = fresh.iter().filter(|&&f| f).count();
        let mut index = 0;
        fresh
            .into_iter()
            .map(|f| {
                f.then(|| {
                    index += 1;
                    stagger_delay(index - 1, count)
                })
            })
            .collect()
    }
}

/// Plays the entrance on its child once, the first time `id` renders.
#[derive(IntoElement)]
pub struct Entrance {
    pub(super) id: ElementId,
    pub(super) skip_initial: bool,
    pub(super) delay: Duration,
    pub(super) child: AnyElement,
}

impl Entrance {
    pub fn new(id: impl Into<ElementId>, child: impl IntoElement) -> Self {
        Self { id: id.into(), skip_initial: true, delay: Duration::ZERO, child: child.into_any_element() }
    }

    /// On by default: the child shows at once the first time it renders. A list turns it off for items
    /// that arrive after its first paint.
    pub fn skip_initial(mut self, skip: bool) -> Self {
        self.skip_initial = skip;
        self
    }

    /// Waits this long before entering, for a stagger.
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }
}

pub(super) struct EntranceMotion {
    pub(super) progress: Channel,
    pub(super) reduce: bool,
}

impl RenderOnce for Entrance {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduce = cx.reduce_motion();
        let (skip, delay) = (self.skip_initial, self.delay.as_secs_f32());
        let motion = window.use_keyed_state(self.id, cx, move |_, _| {
            let mut progress = Channel::new(if skip { 1. } else { 0. });
            if !skip {
                // Reduce Motion keeps a short fade, so the channel never jumps here.
                progress.animate(1., curve(reduce), delay, false);
            }
            EntranceMotion { progress, reduce }
        });
        let m = motion.read(cx);
        if m.progress.is_running() {
            window.request_animation_frame();
        }
        let f = frame(m.progress.value(), m.reduce);
        div().relative().top(px(f.y)).opacity(f.opacity).child(self.child)
    }
}

/// A column of chat items that enter as they arrive. Items present on its first paint show at once;
/// every later id enters once.
#[derive(IntoElement)]
pub struct EntranceList {
    pub(super) id: ElementId,
    pub(super) container: Div,
    pub(super) items: Vec<(ElementId, AnyElement)>,
}

impl EntranceList {
    /// `container` lays the items out, for example `div().flex().flex_col().gap(px(16.))`.
    pub fn new(id: impl Into<ElementId>, container: Div) -> Self {
        Self { id: id.into(), container, items: Vec::new() }
    }

    pub fn item(mut self, id: impl Into<ElementId>, child: impl IntoElement) -> Self {
        self.items.push((id.into(), child.into_any_element()));
        self
    }
}

impl RenderOnce for EntranceList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let list = Arc::new(self.id.clone());
        let arrivals = window.use_keyed_state(self.id, cx, |_, _| Arrivals::default());
        let ids: Vec<ElementId> = self.items.iter().map(|(id, _)| id.clone()).collect();
        let delays = arrivals.update(cx, |a, _| a.arrive(&ids));
        self.container.children(self.items.into_iter().zip(delays).map(|((id, child), delay)| {
            let key = ElementId::NamedChild(list.clone(), id.to_string().into());
            Entrance::new(key, child).skip_initial(delay.is_none()).delay(delay.unwrap_or_default())
        }))
    }
}
