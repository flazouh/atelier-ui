use std::{
    hash::{Hash, Hasher},
    rc::Rc,
};

use gpui_kit::{
    AnyElement, App, ElementId, FocusHandle, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, Styled, Window, anchored, deferred, div, point,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    motion::{Animated, Channel, Curve, FrameClock, ease},
    placement::measure,
    popover::PRIORITY,
    theme::ActiveTheme,
};
use super::types::{CORNER, Close, ENTER_Y, PAD, PANEL, SCRIM_SECONDS, VIEW_SECONDS, VIEW_Y};
use super::helpers::{panel_height, scrim};

struct State {
    previous: Option<FocusHandle>,
    took_focus: bool,
    enter: Animated,
    pub(super) scrim: Channel,
    pub(super) height: Animated,
    sized: bool,
    pub(super) content: Option<f32>,
    pub(super) view: u64,
    swap: Channel,
    pub(super) clock: FrameClock,
}

#[derive(IntoElement)]
pub struct Modal {
    id: ElementId,
    pub(super) view: u64,
    pub(super) width: f32,
    pub(super) focus: Option<FocusHandle>,
    pub(super) on_close: Option<Close>,
    pub(super) child: Option<AnyElement>,
    selector: Option<&'static str>,
    pub(super) flush: bool,
}

impl Modal {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), view: 0, width: 384., focus: None, on_close: None, child: None, selector: None, flush: false }
    }

    /// Which view is shown. When it changes, the panel morphs to the new height and the view comes in.
    pub fn view(mut self, view: impl Hash) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        view.hash(&mut hasher);
        self.view = hasher.finish();
        self
    }

    /// The panel's width, in px (384 by default).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// What takes focus when the modal opens.
    pub fn focus(mut self, handle: &FocusHandle) -> Self {
        self.focus = Some(handle.clone());
        self
    }

    /// Runs on Escape and on a press on the scrim, after focus has gone back.
    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }

    /// Takes the padding away, for a view that fills the panel to its corners (a picture, a banner).
    pub fn flush(mut self) -> Self {
        self.flush = true;
        self
    }
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = Some(child.into_any_element());
        self
    }

    /// The name a test finds the panel by with `debug_bounds`.
    pub fn debug_name(mut self, name: &'static str) -> Self {
        self.selector = Some(name);
        self
    }
}

impl RenderOnce for Modal {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let viewport = window.viewport_size();
        let previous = window.focused(cx);
        let view = self.view;
        let flush = self.flush;
        let state = window.use_keyed_state(self.id.clone(), cx, move |_, _| {
            let mut enter = Animated::new(PANEL, 0.);
            enter.set_target(1.);
            let mut scrim = Channel::new(0.);
            scrim.animate(1., Curve::Ease(SCRIM_SECONDS, ease::OUT), 0., false);
            State {
                previous,
                took_focus: false,
                enter,
                scrim,
                height: Animated::new(PANEL, 0.),
                sized: false,
                content: None,
                view,
                swap: Channel::new(1.),
                clock: FrameClock::default(),
            }
        });
        let (enter, scrim_t, height, swap, moving) = state.update(cx, |s, _| {
            if s.view != view {
                s.view = view;
                s.swap = Channel::new(0.);
                s.swap.animate(1., Curve::Ease(VIEW_SECONDS, ease::OUT), 0., reduce);
            }
            if let Some(content) = s.content {
                let want = panel_height(content) - if flush { 2. * PAD } else { 0. };
                if !s.sized || reduce {
                    s.height = Animated::new(PANEL, want);
                    s.sized = true;
                }
                s.height.set_target(want);
            }
            let dt = s.clock.tick();
            let mut moving = s.enter.step(dt, reduce) | s.height.step(dt, reduce) | s.scrim.is_running() | s.swap.is_running();
            // The panel is measured a frame after it first draws.
            moving |= s.content.is_none();
            if !moving {
                s.clock.rest();
            }
            (if reduce { 1. } else { s.enter.value() }, if reduce { 1. } else { s.scrim.value() }, s.sized.then(|| s.height.value()), s.swap.value(), moving)
        });
        if moving {
            window.request_animation_frame();
        }
        if !state.read(cx).took_focus {
            state.update(cx, |s, _| s.took_focus = true);
            if let Some(handle) = self.focus.clone() {
                window.defer(cx, move |window, cx| window.focus(&handle, cx));
            }
        }

        let dismiss = {
            let (state, close) = (state.clone(), self.on_close.clone());
            move |window: &mut Window, cx: &mut App| {
                if let Some(back) = state.read(cx).previous.clone() {
                    window.focus(&back, cx);
                }
                if let Some(close) = &close {
                    close(window, cx);
                }
            }
        };
        let (press, keys) = (dismiss.clone(), dismiss);
        let (w, h) = (viewport.width, viewport.height);
        let content = {
            let report = state.clone();
            div()
                .relative()
                .flex_none()
                .top(px(VIEW_Y * (1. - swap)))
                .opacity(swap)
                .child(measure(move |b, cx| {
                    // Measured in window pixels; the panel\'s height is kept in design pixels, which the zoom turns back.
                    let height = crate::scale::design(b.size.height);
                    report.update(cx, |s, _| {
                        if s.content != Some(height) {
                            s.content = Some(height);
                        }
                    })
                }))
                .children(self.child)
        };
        let panel = div()
            .relative()
            .w(px(self.width))
            .when_some(height, |d, h| d.h(px(h)))
            .overflow_hidden()
            .rounded(px(CORNER))
            .bg(theme.popover)
            .shadow(crate::theme::popover_shadow(&theme))
            .p(px(if flush { 0. } else { PAD }))
            .top(px(ENTER_Y * (1. - enter.min(1.))))
            .opacity(enter.clamp(0., 1.))
            .when_some(self.selector, |d, name| d.debug_selector(move || name.into()))
            .occlude()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_key_down(move |event, window, cx| {
                if event.keystroke.key == "escape" {
                    cx.stop_propagation();
                    keys(window, cx);
                }
            })
            .child(content);
        let layer = div()
            .relative()
            .w(w)
            .h(h)
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .occlude()
                    .bg(scrim(&theme, scrim_t))
                    .cursor_default()
                    .on_mouse_down(MouseButton::Left, {
                        let press = press.clone();
                        move |_, window, cx| {
                            cx.stop_propagation();
                            press(window, cx);
                        }
                    })
                    .on_mouse_down(MouseButton::Right, move |_, window, cx| {
                        cx.stop_propagation();
                        press(window, cx);
                    })
                    .on_scroll_wheel(|_, _, cx| cx.stop_propagation()),
            )
            .child(div().absolute().inset_0().flex().items_center().justify_center().child(panel));
        deferred(anchored().position(point(px(0.), px(0.))).child(layer)).with_priority(PRIORITY).into_any_element()
    }
}
