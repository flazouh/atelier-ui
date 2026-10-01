//! Modal: beui.dev's Morphing Modal (`components/motion/morphing-modal.tsx`), placed in the centre. One panel
//! over a scrim, for the dialogs that ask something of the reader (the SSH form, the folder picker, a new
//! task). The owner keeps the open state: it draws a `Modal` while the dialog is open and stops when it
//! closes.
//!
//! - Panel: atelier's look, not the web's: the old dialogs' panel: a 12px corner, no border, the popover fill and shadow, and 16px of
//!   padding round the view. Its width is the owner's (384px in the web).
//! - Enter: 20px below and clear, then up into place on `Spring::PANEL` (`{ 420, 40, 0.5 }`). The scrim
//!   fades in over 200ms. The web blurs what is behind by 14px; here a dim theme overlay stands in for it.
//! - Morph: when the view's height changes (its `view` key changed, and its words are longer or shorter),
//!   the panel's height follows on the same spring, and the new view comes in 8px below and clear over 240ms.
//! - Close: Escape or a press on the scrim asks the owner to close (`on_close`), and focus goes back to
//!   what had it when the modal opened. A press in the panel does not close it. The first focusable thing
//!   named by [`Modal::focus`] takes focus on open.
//! - Under Reduce Motion the panel does not move and its height follows at once.
//!
//! What gpui cannot draw is left out: the panel's 0.97 scale, the blur of the views as they change, and the
//! exit (the owner removes the modal at once).
use std::{
    hash::{Hash, Hasher},
    rc::Rc,
};

use gpui_kit::{
    AnyElement, App, ElementId, FocusHandle, InteractiveElement, IntoElement, MouseButton, ParentElement, RenderOnce, Styled, Window, anchored,
    deferred, div, point, prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::{
    motion::{Animated, Channel, Curve, FrameClock, Spring, ease},
    placement::measure,
    popover::PRIORITY,
    theme::{ActiveTheme, Theme},
};

/// The padding round the view, and the border (none: atelier is borderless).
pub const PAD: f32 = 16.;
pub const BORDER: f32 = 0.;
/// The panel's corner: atelier's card radius for a floating surface.
pub const CORNER: f32 = 12.;
/// How far below the panel starts (`enterY` for the centre placement).
pub const ENTER_Y: f32 = 20.;
/// The panel's spring.
pub const PANEL: Spring = Spring::PANEL;
/// How long the scrim takes to come, and a new view.
const SCRIM_SECONDS: f32 = 0.2;
const VIEW_SECONDS: f32 = 0.24;
/// How far below a new view starts.
pub const VIEW_Y: f32 = 8.;

/// The dim over the page, at progress `t` of its fade: the theme's shadow colour.
pub fn scrim(theme: &Theme, t: f32) -> gpui_kit::Hsla {
    gpui_kit::Hsla { a: 0.28 * t, ..theme.shadow }
}

/// The panel's height for a view of `content` px: the view, its padding and its border.
pub fn panel_height(content: f32) -> f32 {
    content + 2. * PAD + 2. * BORDER
}

type Close = Rc<dyn Fn(&mut Window, &mut App)>;

struct State {
    previous: Option<FocusHandle>,
    took_focus: bool,
    enter: Animated,
    scrim: Channel,
    height: Animated,
    sized: bool,
    content: Option<f32>,
    view: u64,
    swap: Channel,
    clock: FrameClock,
}

#[derive(IntoElement)]
pub struct Modal {
    id: ElementId,
    view: u64,
    width: f32,
    focus: Option<FocusHandle>,
    on_close: Option<Close>,
    child: Option<AnyElement>,
    selector: Option<&'static str>,
}

impl Modal {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), view: 0, width: 384., focus: None, on_close: None, child: None, selector: None }
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
                let want = panel_height(content);
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
                    let height = f32::from(b.size.height);
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
            .p(px(PAD))
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

#[cfg(test)]
mod tests;
