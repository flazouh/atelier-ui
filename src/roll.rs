//! Roll: the swap of beui.dev's Animated Badge (`components/motion/animated-badge.tsx`), for anything that
//! changes in place. When its `key` changes, the old content leaves upward and the new content comes up from
//! below into its place, inside a box that clips both.
//!
//! - Enter: from 80% (85% for words) of the content's height below, at 72% (76%) opacity, to rest. The rise is
//!   a spring `{ stiffness: 210, damping: 24, mass: 0.85 }`; the opacity takes 280ms (300ms for words).
//! - Exit: to 80% (85%) of the height above, to 50% opacity, over 220ms (200ms for words). The old content
//!   leaves the flow at once, so the box takes the new content's size.
//! - Under Reduce Motion the content changes at once.
//!
//! What gpui cannot draw is left out: the blur (6px), the 0.92 scale and the 8 degrees of turn of an icon.
//! The content is drawn by the `build` function, once for what is here and once more for what is leaving.
use std::rc::Rc;

use gpui_kit::{AnyElement, App, ElementId, IntoElement, ParentElement, Pixels, RenderOnce, Styled, Window, div, prelude::FluentBuilder, };
use crate::scale::px;

use crate::motion::{Animated, Channel, Curve, FrameClock, Spring, ease};

/// The rise, for icons and marks.
pub const RISE: Spring = Spring { stiffness: 210., damping: 24., mass: 0.85 };
/// What a roll is made for: a mark (an icon) or words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Icon,
    Words,
    /// Action Swap's roll (`components/motion/action-swap.tsx`): 90% below on `Spring::SWAP`, clear to full over 250ms,
    /// and out through the top to clear in 140ms.
    Swap,
    /// Digit Swap's glyph (`components/motion/digit-swap.tsx`): 45% below, clear to full, over 180ms, and out through the top.
    Digit,
}

impl Kind {
    /// How far below, as a share of the height, the new content starts.
    pub fn start(self) -> f32 {
        match self {
            Kind::Icon => 0.8,
            Kind::Words => 0.85,
            Kind::Swap => 0.9,
            Kind::Digit => 0.45,
        }
    }

    /// The spring the new content rises on.
    pub fn spring(self) -> Spring {
        match self {
            Kind::Swap => Spring::SWAP,
            // A critical spring settles in about the 180ms of the web's tween.
            Kind::Digit => Spring::critical(32.),
            _ => RISE,
        }
    }

    /// The opacity the old content leaves at.
    pub fn exit_opacity(self) -> f32 {
        match self {
            Kind::Swap | Kind::Digit => 0.,
            _ => 0.5,
        }
    }

    /// The opacity the new content starts at, and how long it takes to reach 1.
    pub fn enter_opacity(self) -> (f32, f32) {
        match self {
            Kind::Icon => (0.72, 0.28),
            Kind::Words => (0.76, 0.3),
            Kind::Swap => (0., 0.25),
            Kind::Digit => (0., 0.18),
        }
    }

    /// How long the old content takes to leave.
    pub fn exit_seconds(self) -> f32 {
        match self {
            Kind::Icon => 0.22,
            Kind::Words => 0.2,
            Kind::Swap => 0.14,
            Kind::Digit => 0.18,
        }
    }
}

/// The new content's offset from its place, in shares of its height (positive is below), and its opacity,
/// with the rise at `rise` (1 at the start, 0 at rest) and the opacity at progress `fade` (0 to 1).
pub fn entering(kind: Kind, rise: f32, fade: f32) -> (f32, f32) {
    let (from, _) = kind.enter_opacity();
    (kind.start() * rise, from + (1. - from) * fade)
}

/// The old content's offset (negative is above) and opacity at exit progress `t` (0 to 1).
pub fn leaving(kind: Kind, t: f32) -> (f32, f32) {
    (-kind.start() * t, 1. - (1. - kind.exit_opacity()) * t)
}

type Build<K> = Rc<dyn Fn(&K) -> AnyElement>;

struct State<K> {
    shown: K,
    leaving: Option<K>,
    rise: Animated,
    fade: Channel,
    exit: Channel,
    clock: FrameClock,
}

#[derive(IntoElement)]
pub struct Roll<K: Clone + PartialEq + 'static> {
    id: ElementId,
    key: K,
    kind: Kind,
    height: Pixels,
    build: Build<K>,
}

impl<K: Clone + PartialEq + 'static> Roll<K> {
    /// `height` is the content's height, which the travel is measured in.
    pub fn new(id: impl Into<ElementId>, key: K, kind: Kind, height: Pixels, build: impl Fn(&K) -> AnyElement + 'static) -> Self {
        Self { id: id.into(), key, kind, height, build: Rc::new(build) }
    }
}

impl<K: Clone + PartialEq + 'static> RenderOnce for Roll<K> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduce = cx.reduce_motion();
        let kind = self.kind;
        let key = self.key.clone();
        let state = window.use_keyed_state(self.id.clone(), cx, move |_, _| State {
            shown: key,
            leaving: None,
            rise: Animated::new(kind.spring(), 0.),
            fade: Channel::new(1.),
            exit: Channel::new(1.),
            clock: FrameClock::default(),
        });
        let (now, gone, moving, (rise, fade, exit)) = state.update(cx, |s, _| {
            if s.shown != self.key {
                if reduce {
                    s.shown = self.key.clone();
                    s.leaving = None;
                } else {
                    s.leaving = Some(std::mem::replace(&mut s.shown, self.key.clone()));
                    s.rise = Animated::new(kind.spring(), 1.);
                    s.rise.set_target(0.);
                    s.fade = Channel::new(0.);
                    s.fade.animate(1., Curve::Ease(kind.enter_opacity().1, ease::OUT), 0., false);
                    s.exit = Channel::new(0.);
                    s.exit.animate(1., Curve::Ease(kind.exit_seconds(), ease::OUT), 0., false);
                }
            }
            let dt = s.clock.tick();
            let moving = s.rise.step(dt, reduce) | s.fade.is_running() | s.exit.is_running();
            if !moving {
                s.leaving = None;
                s.clock.rest();
            }
            (s.shown.clone(), s.leaving.clone(), moving, (s.rise.value(), s.fade.value(), s.exit.value()))
        });
        if moving {
            window.request_animation_frame();
        }
        let h = f32::from(self.height);
        let (dy, opacity) = if gone.is_some() { entering(kind, rise, fade) } else { (0., 1.) };
        let (gone_dy, gone_opacity) = leaving(kind, exit);
        div()
            .relative()
            .flex_none()
            .h(self.height)
            .overflow_hidden()
            .child(div().relative().top(px(dy * h)).opacity(opacity).child((self.build)(&now)))
            .when_some(gone.filter(|_| exit < 1.), |d, old| {
                d.child(div().absolute().left_0().top(px(gone_dy * h)).opacity(gone_opacity).child((self.build)(&old)))
            })
    }
}

#[cfg(test)]
mod tests;
