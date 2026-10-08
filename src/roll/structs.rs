use std::rc::Rc;

use gpui_kit::{
    AnyElement, App, ElementId, IntoElement, ParentElement, Pixels, RenderOnce, Styled, Window,
    div, prelude::FluentBuilder,
};

use super::helpers::{entering, leaving};
use super::types::{Build, Kind};
use crate::motion::{Animated, Channel, Curve, FrameClock, ease};
use crate::scale::px;

struct State<K> {
    shown: K,
    pub(super) leaving: Option<K>,
    pub(super) rise: Animated,
    pub(super) fade: Channel,
    pub(super) exit: Channel,
    pub(super) clock: FrameClock,
}

#[derive(IntoElement)]
pub struct Roll<K: Clone + PartialEq + 'static> {
    id: ElementId,
    pub(super) key: K,
    pub(super) kind: Kind,
    pub(super) height: Pixels,
    pub(super) build: Build<K>,
}

impl<K: Clone + PartialEq + 'static> Roll<K> {
    /// `height` is the content's height, which the travel is measured in.
    pub fn new(
        id: impl Into<ElementId>,
        key: K,
        kind: Kind,
        height: Pixels,
        build: impl Fn(&K) -> AnyElement + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            key,
            kind,
            height,
            build: Rc::new(build),
        }
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
                    s.fade.animate(
                        1.,
                        Curve::Ease(kind.enter_opacity().1, ease::OUT),
                        0.,
                        false,
                    );
                    s.exit = Channel::new(0.);
                    s.exit
                        .animate(1., Curve::Ease(kind.exit_seconds(), ease::OUT), 0., false);
                }
            }
            let dt = s.clock.tick();
            let moving = s.rise.step(dt, reduce) | s.fade.is_running() | s.exit.is_running();
            if !moving {
                s.leaving = None;
                s.clock.rest();
            }
            (
                s.shown.clone(),
                s.leaving.clone(),
                moving,
                (s.rise.value(), s.fade.value(), s.exit.value()),
            )
        });
        if moving {
            window.request_animation_frame();
        }
        let h = f32::from(self.height);
        let (dy, opacity) = if gone.is_some() {
            entering(kind, rise, fade)
        } else {
            (0., 1.)
        };
        let (gone_dy, gone_opacity) = leaving(kind, exit);
        div()
            .relative()
            .flex_none()
            .h(self.height)
            .overflow_hidden()
            .child(
                div()
                    .relative()
                    .top(px(dy * h))
                    .opacity(opacity)
                    .child((self.build)(&now)),
            )
            .when_some(gone.filter(|_| exit < 1.), |d, old| {
                d.child(
                    div()
                        .absolute()
                        .left_0()
                        .top(px(gone_dy * h))
                        .opacity(gone_opacity)
                        .child((self.build)(&old)),
                )
            })
    }
}
