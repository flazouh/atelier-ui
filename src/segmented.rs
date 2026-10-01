//! Segmented: beui.dev's inline choice control (`file-upload.preview.tsx`), in atelier's look: a row of small
//! buttons, as the app had before, with no track. A segment is a `Sm` button (28px, 10px across, `rounded LG`):
//! the chosen one wears the `Secondary` fill and the others the `Ghost` one. Hover takes the text to the
//! foreground. Fill and text change over 150ms, and a pressed segment shrinks to 95%. Reduce Motion makes
//! every change a jump.
//!
//! Each segment is a Tab stop. Enter and Space choose it. A key cap belongs to the segment that key picks; a key
//! that toggles between the segments is shown once, after the control ([`Segmented::cap`]).
use std::rc::Rc;

use gpui_kit::{
    App, ElementId, FocusHandle, FontWeight, Hsla, InteractiveElement, IntoElement, MouseButton, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, relative,
};
use crate::scale::px;

use crate::{
    kbd::Kbd,
    motion::{Channel, Curve, FrameClock},
    theme::{ActiveTheme, Theme, mix, radius},
    typography::FONT_FAMILY,
};

/// The gap between segments: the old rows of buttons had 2px.
pub const GAP: f32 = 2.;
/// A segment's height and side padding: a `Sm` button's.
pub const SEGMENT_HEIGHT: f32 = 28.;
pub const SEGMENT_PAD: f32 = 10.;
/// The text: a `Sm` button's, and its line.
pub const TEXT: f32 = 11.;
pub const LINE: f32 = 16.;
/// The gap between the track and a toggle key's cap.
pub const CAP_GAP: f32 = 8.;
/// How long fill and text take to change (`duration-150`).
const CHANGE: f32 = 0.15;
/// How far a pressed segment shrinks (`active:scale-95`).
const PRESS_SCALE: f32 = 0.95;
/// Tailwind's default easing, `cubic-bezier(0.4, 0, 0.2, 1)`.
const EASE: [f32; 4] = [0.4, 0., 0.2, 1.];

type ChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// One choice: its words, and a key cap if it has one.
#[derive(Clone)]
pub struct Segment {
    label: SharedString,
    cap: Option<SharedString>,
    selector: Option<&'static str>,
}

impl Segment {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self { label: label.into(), cap: None, selector: None }
    }

    /// The name a test finds this segment by, instead of the track's name and its index.
    pub fn debug_name(mut self, name: &'static str) -> Self {
        self.selector = Some(name);
        self
    }

    pub fn cap(mut self, keys: impl Into<SharedString>) -> Self {
        self.cap = Some(keys.into());
        self
    }
}

/// A segment's fill at choice progress `chosen` (0 to 1) and hover progress `hover`: the `Ghost` button's,
/// or the `Secondary` button's when chosen.
pub fn segment_fill(theme: &Theme, chosen: f32, hover: f32) -> Hsla {
    let ghost = crate::button::colors(crate::button::ButtonVariant::Ghost, theme, hover, false).0;
    let secondary = crate::button::colors(crate::button::ButtonVariant::Secondary, theme, hover, false).0;
    ghost.opacity(1. - chosen).blend(secondary.opacity(chosen))
}

/// A segment's text: muted, then the foreground when chosen or hovered (`hover:text-foreground`).
pub fn segment_text(theme: &Theme, chosen: f32, hover: f32) -> Hsla {
    mix(theme.muted_foreground, theme.foreground, chosen.max(hover))
}

/// How far a pill pulls in from each side at press progress `press`: a share of its width across, pixels
/// down. At full press the pill is 95% of its size.
pub fn pill_inset(press: f32) -> (f32, f32) {
    let shrink = (1. - PRESS_SCALE) * press;
    (shrink / 2., SEGMENT_HEIGHT * shrink / 2.)
}

#[derive(IntoElement)]
pub struct Segmented {
    id: SharedString,
    segments: Vec<Segment>,
    selected: usize,
    on_change: Option<ChangeHandler>,
    selector: Option<&'static str>,
    cap: Option<SharedString>,
}

impl Segmented {
    pub fn new(id: impl Into<SharedString>, segments: impl IntoIterator<Item = Segment>, selected: usize) -> Self {
        Self { id: id.into(), segments: segments.into_iter().collect(), selected, on_change: None, selector: None, cap: None }
    }

    /// Hears the index of the segment chosen. It does not fire for the segment already chosen.
    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// The key that toggles between the segments, shown once after the control. Use a segment's own cap
    /// only when the key picks that one segment.
    pub fn cap(mut self, keys: impl Into<SharedString>) -> Self {
        self.cap = Some(keys.into());
        self
    }

    /// The name a test finds the track by with `debug_bounds`. Each segment is `<name>-<index>`.
    pub fn debug_name(mut self, name: &'static str) -> Self {
        self.selector = Some(name);
        self
    }
}

/// Per-segment motion, kept across frames by element id.
struct Motion {
    hovered: bool,
    pressed: bool,
    chosen: Channel,
    hover: Channel,
    press: Channel,
    clock: FrameClock,
    focus: Option<FocusHandle>,
}

impl Motion {
    fn new(chosen: bool) -> Self {
        Self {
            hovered: false,
            pressed: false,
            chosen: Channel::new(f32::from(u8::from(chosen))),
            hover: Channel::new(0.),
            press: Channel::new(0.),
            clock: FrameClock::default(),
            focus: None,
        }
    }

    fn retarget(&mut self, chosen: bool, reduce: bool) {
        let go = |channel: &mut Channel, to: f32| {
            if channel.target() != to {
                channel.animate(to, Curve::Ease(CHANGE, EASE), 0., reduce);
            }
        };
        go(&mut self.chosen, f32::from(u8::from(chosen)));
        go(&mut self.hover, f32::from(u8::from(self.hovered)));
        go(&mut self.press, f32::from(u8::from(self.pressed)));
    }

    fn moving(&mut self) -> bool {
        self.clock.tick();
        let moving = self.chosen.is_running() | self.hover.is_running() | self.press.is_running();
        if !moving {
            self.clock.rest();
        }
        moving
    }
}

impl RenderOnce for Segmented {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let handler = self.on_change.clone();
        let selected = self.selected;
        let mut moving = false;
        let segments: Vec<_> = self
            .segments
            .into_iter()
            .enumerate()
            .map(|(i, segment)| {
                let key = ElementId::Name(format!("{}-{i}", self.id).into());
                let chosen = i == selected;
                let motion = window.use_keyed_state(key.clone(), cx, |_, _| Motion::new(chosen));
                let (chosen_p, hover_p, press, focus, running) = motion.update(cx, |m, cx| {
                    let focus = m.focus.get_or_insert_with(|| cx.focus_handle()).clone();
                    m.retarget(chosen, reduce);
                    let running = m.moving();
                    (m.chosen.value(), m.hover.value(), m.press.value(), focus, running)
                });
                let (fill, ink) = (segment_fill(&theme, chosen_p, hover_p), segment_text(&theme, chosen_p, hover_p));
                moving |= running;
                let keyed = focus.is_focused(window) && window.last_input_was_keyboard();
                let (across, down) = pill_inset(press);
                let choose = handler.clone().map(|f| move |window: &mut Window, cx: &mut App| if !chosen { f(i, window, cx) });
                let named = segment.selector;
                let selector = self.selector;
                div()
                    .id(key)
                    .relative()
                    .flex()
                    .flex_none()
                    .items_center()
                    .h(px(SEGMENT_HEIGHT))
                    .px(px(SEGMENT_PAD))
                    .gap(px(6.))
                    .rounded(radius::lg())
                    .when(keyed, |d| d.shadow(crate::focus::ring_shadow(&theme, theme.card)))
                    .font_family(FONT_FAMILY)
                    .font_weight(FontWeight::MEDIUM)
                    .text_size(px(TEXT))
                    .line_height(px(LINE))
                    .whitespace_nowrap()
                    .text_color(ink)
                    .cursor_pointer()
                    .child(
                        div()
                            .absolute()
                            .top(px(down))
                            .bottom(px(down))
                            .left(relative(across))
                            .right(relative(across))
                            .rounded(radius::lg())
                            .bg(fill),
                    )
                    .child(div().relative().child(segment.label))
                    .when_some(segment.cap, |d, cap| d.child(div().relative().child(Kbd::new(cap).ink(ink))))
                    .when_some(named, |d, name| d.debug_selector(move || name.into()))
                    .when(named.is_none(), |d| d.when_some(selector, |d, name| d.debug_selector(move || format!("{name}-{i}"))))
                    .track_focus(&focus.tab_stop(true))
                    .when_some(choose.clone(), |d, choose| {
                        d.on_key_down(move |event, window, cx| {
                            if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                cx.stop_propagation();
                                choose(window, cx);
                            }
                        })
                    })
                    .on_hover({
                        let motion = motion.clone();
                        move |on, _, cx| {
                            let reduce = cx.reduce_motion();
                            motion.update(cx, |m, cx| {
                                m.hovered = *on;
                                m.retarget(chosen, reduce);
                                cx.notify();
                            });
                        }
                    })
                    .on_mouse_down(MouseButton::Left, {
                        let motion = motion.clone();
                        move |_, _, cx| {
                            let reduce = cx.reduce_motion();
                            motion.update(cx, |m, cx| {
                                m.pressed = true;
                                m.retarget(chosen, reduce);
                                cx.notify();
                            });
                        }
                    })
                    .on_mouse_up(MouseButton::Left, {
                        let motion = motion.clone();
                        move |_, _, cx| {
                            let reduce = cx.reduce_motion();
                            motion.update(cx, |m, cx| {
                                m.pressed = false;
                                m.retarget(chosen, reduce);
                                cx.notify();
                            });
                        }
                    })
                    .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                        let reduce = cx.reduce_motion();
                        motion.update(cx, |m, cx| {
                            m.pressed = false;
                            m.retarget(chosen, reduce);
                            cx.notify();
                        });
                    })
                    .when_some(choose, |d, choose| d.on_click(move |_, window, cx| choose(window, cx)))
            })
            .collect();
        if moving {
            window.request_animation_frame();
        }
        let track = div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(GAP))
            .when_some(self.selector, |d, name| d.debug_selector(move || name.into()))
            .children(segments);
        match self.cap {
            None => track.into_any_element(),
            Some(cap) => {
                let selector = self.selector;
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(CAP_GAP))
                    .child(track)
                    .child(
                        div()
                            .when_some(selector, |d, name| d.debug_selector(move || format!("{name}-cap")))
                            .child(Kbd::new(cap).ink(theme.muted_foreground)),
                    )
                    .into_any_element()
            }
        }
    }
}

#[cfg(test)]
mod tests;
