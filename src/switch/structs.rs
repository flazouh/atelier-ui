use std::rc::Rc;

use gpui_kit::{
    App, BoxShadow, ElementId, FocusHandle, Hsla, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
    point, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    focus::ring_color,
    kbd::Kbd,
    motion::{Animated, Channel, Curve, FrameClock},
    theme::ActiveTheme,
    typography::FONT_FAMILY,
};
use super::types::{
    Change, EASE_IN_OUT, FILL_SECONDS, HEIGHT, PAD, SHAKE_DELAY, SHAKE_SECONDS, SQUEEZE, THUMB,
    THUMB_SPRING, WIDTH,
};
use super::helpers::{shake_offset, span, track_fill};

/// The size of the track and the thumb in it.
#[derive(Clone, Copy)]
pub(super) struct Dims {
    pub(super) width: f32,
    pub(super) height: f32,
    pub(super) pad: f32,
    pub(super) thumb: f32,
}

impl Dims {
    pub(super) const STANDARD: Dims = Dims { width: WIDTH, height: HEIGHT, pad: PAD, thumb: THUMB };
    /// A 26 x 16 track with a 12px thumb: the same motion in a smaller body.
    pub(super) const COMPACT: Dims = Dims { width: 26., height: 16., pad: 2., thumb: 12. };
    pub(super) fn travel(self) -> f32 {
        self.width - 2. * self.pad - self.thumb
    }
}

pub(super) struct Motion {
    pub(super) thumb: Animated,
    pub(super) press: Animated,
    pub(super) fill: Channel,
    pub(super) shake: Channel,
    pub(super) pressed: bool,
    pub(super) pointer: bool,
    clock: FrameClock,
    focus: Option<FocusHandle>,
}

#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    pub(super) on: bool,
    compact: bool,
    pub(super) disabled: bool,
    pub(super) label: Option<SharedString>,
    pub(super) cap: Option<SharedString>,
    pub(super) on_change: Option<Change>,
    selector: Option<&'static str>,
}

impl Switch {
    pub fn new(id: impl Into<ElementId>, on: bool) -> Self {
        Self { id: id.into(), on, compact: false, disabled: false, label: None, cap: None, on_change: None, selector: None }
    }

    /// A 26 x 16 track with a 12px thumb, for a dense row. The motion is the same.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// The key that toggles the switch, shown once after the label.
    pub fn cap(mut self, keys: impl Into<SharedString>) -> Self {
        self.cap = Some(keys.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }

    /// The name a test finds the track by with `debug_bounds`; its thumb is `<name>-thumb`.
    pub fn debug_name(mut self, name: &'static str) -> Self {
        self.selector = Some(name);
        self
    }
}

impl RenderOnce for Switch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let on = self.on;
        let motion = window.use_keyed_state(self.id.clone(), cx, |_, _| Motion {
            thumb: Animated::new(THUMB_SPRING, f32::from(u8::from(on))),
            press: Animated::new(THUMB_SPRING, 0.),
            fill: Channel::new(f32::from(u8::from(on))),
            shake: Channel::new(1.),
            pressed: false,
            pointer: false,
            clock: FrameClock::default(),
            focus: None,
        });
        let disabled = self.disabled;
        let (t, press, fill, shake, moving, focus) = motion.update(cx, |m, cx| {
            let focus = m.focus.get_or_insert_with(|| cx.focus_handle()).clone();
            m.thumb.set_target(f32::from(u8::from(on)));
            let squeeze = !disabled && m.pointer && m.pressed && !reduce;
            m.press.set_target(f32::from(u8::from(squeeze)));
            let to = f32::from(u8::from(on));
            if m.fill.target() != to {
                m.fill.animate(to, Curve::Ease(FILL_SECONDS, EASE_IN_OUT), 0., reduce);
            }
            let dt = m.clock.tick();
            let moving = m.thumb.step(dt, reduce) | m.press.step(dt, reduce) | m.fill.is_running() | m.shake.is_running();
            if !moving {
                m.clock.rest();
            }
            (m.thumb.value(), m.press.value(), m.fill.value(), m.shake.value(), moving, focus)
        });
        if moving {
            window.request_animation_frame();
        }
        let keyed = focus.is_focused(window) && window.last_input_was_keyboard() && !disabled;
        let page = theme.background;
        let dims = if self.compact { Dims::COMPACT } else { Dims::STANDARD };
        let (left, width) = span(dims, t, press, on);
        let scale = 1. - (1. - SQUEEZE) * press;
        let (w, h) = (width * scale, dims.thumb * scale);
        let shake_x = if reduce || !disabled { 0. } else { shake_offset(shake * (SHAKE_DELAY + SHAKE_SECONDS)) };
        let shadow_color = Hsla { a: 0.1, ..theme.shadow };
        let thumb = div()
            .absolute()
            .left(px(left + (width - w) / 2. + shake_x))
            .top(px(dims.pad + (dims.thumb - h) / 2.))
            .w(px(w))
            .h(px(h))
            .rounded_full()
            .bg(page)
            .shadow(vec![
                BoxShadow { color: shadow_color, offset: point(px(0.), px(4.)), blur_radius: px(6.), spread_radius: px(-1.), inset: false },
                BoxShadow { color: shadow_color, offset: point(px(0.), px(2.)), blur_radius: px(4.), spread_radius: px(-2.), inset: false },
            ])
            .when_some(self.selector, |d, name| d.debug_selector(move || format!("{name}-thumb")));
        let change = self.on_change.clone().filter(|_| !disabled);
        let toggle = change.clone().map(|f| move |window: &mut Window, cx: &mut App| f(!on, window, cx));
        let hold = {
            let (down, up, away) = (motion.clone(), motion.clone(), motion.clone());
            (
                move |pointer: bool, cx: &mut App| down.update(cx, |m, _| {
                    m.pressed = true;
                    m.pointer = pointer;
                    if disabled {
                        m.shake = Channel::new(0.);
                        m.shake.animate(1., Curve::Ease(SHAKE_DELAY + SHAKE_SECONDS, [0., 0., 1., 1.]), 0., false);
                    }
                }),
                move |cx: &mut App| up.update(cx, |m, _| m.pressed = false),
                move |cx: &mut App| away.update(cx, |m, _| m.pressed = false),
            )
        };
        let (down, up, away) = hold;
        let ring = keyed.then(|| {
            vec![
                BoxShadow { color: page, offset: point(px(0.), px(0.)), blur_radius: px(0.), spread_radius: px(2.), inset: false },
                BoxShadow { color: ring_color(&theme, page), offset: point(px(0.), px(0.)), blur_radius: px(0.), spread_radius: px(4.), inset: false },
            ]
        });
        let track = div()
            .id(self.id.clone())
            .relative()
            .flex_none()
            .w(px(dims.width))
            .h(px(dims.height))
            .rounded_full()
            .bg(track_fill(&theme, page, fill))
            .when_some(ring, |d, ring| d.shadow(ring))
            .when(disabled, |d| d.opacity(0.6))
            .when(!disabled, |d| d.cursor_pointer().track_focus(&focus.tab_stop(true)))
            .when_some(self.selector, |d, name| d.debug_selector(move || name.into()))
            .child(thumb)
            .on_mouse_down(MouseButton::Left, move |_, _, cx| down(true, cx))
            .on_mouse_up(MouseButton::Left, move |_, _, cx| up(cx))
            .on_mouse_up_out(MouseButton::Left, move |_, _, cx| away(cx))
            .when_some(toggle.clone(), |d, toggle| {
                let key = toggle.clone();
                d.on_click(move |_, window, cx| toggle(window, cx)).on_key_down(move |event, window, cx| {
                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                        cx.stop_propagation();
                        key(window, cx);
                    }
                })
            });
        let label = self.label.map(|words| {
            let click = toggle.clone();
            div()
                .id(ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), "label".into()))
                .text_size(px(14.))
                .line_height(px(20.))
                .text_color(theme.foreground)
                .when(!disabled, |d| d.cursor_pointer())
                .when_some(click, |d, f| d.on_click(move |_, window, cx| f(window, cx)))
                .child(words)
        });
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(12.))
            .font_family(FONT_FAMILY)
            .child(track)
            .children(label)
            .children(self.cap.map(Kbd::new))
    }
}
