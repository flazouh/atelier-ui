//! beui's RangeSlider (`components/motion/range-slider.tsx`, with `lib/hooks/use-slider.ts` for the number):
//! a 40px track with `rounded-lg` and the `muted` fill, a `foreground/15` fill behind a 4x24px handle, and a
//! dot 4px across at each step. The handle sits 8px inside the track and the fill starts 8px past its left edge,
//! as `InlineSlider` does. Both follow the value through a spring (`SPRING_GLIDE`: stiff and critically damped, so
//! they follow a drag and never rebound off an end); grabbing the handle stretches it to 135% of its height on a
//! bouncy spring. A press anywhere on the track moves the handle there and starts a drag that goes on outside
//! the track. Arrow keys step, Page keys step ten, Home and End go to the ends. Disabled is 50%.
//!
//! The number is snapped to the step grid; a range the step does not divide (0 to 10 by 4) also allows its
//! maximum, so a drag near the end does not fall back a whole step ([`snap`]).
use std::rc::Rc;

use gpui_kit::{
    App, ElementId, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, MouseButton, MouseMoveEvent, MouseUpEvent,
    ParentElement, RenderOnce, Styled, Window, canvas, div, prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::{
    motion::{Channel, Curve, Spring},
    placement::measure,
    theme::{ActiveTheme, radius},
};

/// The track: `h-10`, `rounded-lg`. The fill is inset `2px` at each end, the handle starts `8px` in and its
/// travel is `width - 20`, and the ticks run `10px` in from each end.
const TRACK_HEIGHT: f32 = 40.;
const FILL_INSET: f32 = 2.;
const HANDLE_START: f32 = 8.;
const TRAVEL_LOSS: f32 = 20.;
const TICK_INSET: f32 = 10.;
const HANDLE_WIDTH: f32 = 4.;
const HANDLE_HEIGHT: f32 = 24.;
const TICK: f32 = 4.;
/// A grabbed handle stretches to this.
const GRAB_SCALE: f32 = 1.35;
/// `bg-foreground/15`, `bg-foreground/25`, `ring-foreground/30`.
const FILL_ALPHA: f32 = 0.15;
const TICK_ALPHA: f32 = 0.25;
const FOCUS_ALPHA: f32 = 0.3;
const DISABLED: f32 = 0.5;
/// More steps than this draw no dots.
const MOST_TICKS: usize = 50;
/// The width assumed before the first layout.
const FIRST_WIDTH: f32 = 292.;

pub type ChangeHandler = Rc<dyn Fn(f32, &mut Window, &mut App)>;

/// The nearest legal value on `[min, max]` for `step` (`snapSliderValue`). `max` counts as a candidate when the
/// step does not divide the range.
pub fn snap(next: f32, min: f32, max: f32, step: f32) -> f32 {
    if max <= min {
        return min;
    }
    if step <= 0. {
        return next.clamp(min, max);
    }
    let whole = (((max - min) / step) * 1e6).round() / 1e6;
    let whole = whole.floor();
    let last = min + whole * step;
    let to_grid = (((next - min) / step).round() * step + min).clamp(min, last);
    let snapped = if last < max && (next - max).abs() <= (next - to_grid).abs() { max } else { to_grid };
    (snapped * 1e6).round() / 1e6
}

/// The value as a percent of the range.
pub fn percent(value: f32, min: f32, max: f32) -> f32 {
    if max > min { (value.clamp(min, max) - min) / (max - min) * 100. } else { 0. }
}

/// The values that get a dot, when there are any: one for each step, if there are 50 steps or fewer.
pub fn ticks(min: f32, max: f32, step: f32) -> Vec<f32> {
    if max <= min || step <= 0. {
        return Vec::new();
    }
    let steps = ((((max - min) / step) * 1e6).round() / 1e6).floor() as usize;
    if steps == 0 || steps > MOST_TICKS {
        return Vec::new();
    }
    (0..=steps).map(|i| ((min + i as f32 * step) * 1e6).round() / 1e6).collect()
}

/// Where the key goes: the value a key press asks for, or `None` for a key the slider does not use.
pub fn key_value(key: &str, current: f32, min: f32, max: f32, step: f32) -> Option<f32> {
    Some(match key {
        "right" | "up" => current + step,
        "left" | "down" => current - step,
        "pageup" => current + step * 10.,
        "pagedown" => current - step * 10.,
        "home" => min,
        "end" => max,
        _ => return None,
    })
}

/// The handle's left edge and the fill's offset, for a track `width` wide and the value at `percent`.
pub fn geometry(width: f32, percent: f32) -> (f32, f32) {
    let handle = HANDLE_START + (width - TRAVEL_LOSS).max(0.) * percent / 100.;
    let clip = width - 2. * FILL_INSET;
    let fill = if percent >= 100. { 0. } else { (percent - 100.) / 100. * clip + 14. - 0.16 * percent };
    (handle, fill)
}

#[derive(IntoElement)]
pub struct RangeSlider {
    id: ElementId,
    value: f32,
    min: f32,
    max: f32,
    step: f32,
    ticks: bool,
    disabled: bool,
    on_change: Option<ChangeHandler>,
}

impl RangeSlider {
    pub fn new(id: impl Into<ElementId>, value: f32) -> Self {
        Self { id: id.into(), value, min: 0., max: 100., step: 1., ticks: true, disabled: false, on_change: None }
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    /// A dot at each step (the default).
    pub fn ticks(mut self, ticks: bool) -> Self {
        self.ticks = ticks;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, f: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

struct Motion {
    focus: FocusHandle,
    /// The track's left edge and width in the last layout.
    left: f32,
    width: f32,
    dragging: bool,
    keyboard: bool,
    /// The value the handle and the fill follow, as a percent, and how far the handle is stretched.
    pos: Channel,
    grab: Channel,
}

impl RenderOnce for RangeSlider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let (min, max) = (self.min, if self.max > self.min { self.max } else { self.min });
        let step = if self.step > 0. { self.step } else { 1. };
        let value = self.value.clamp(min, max);
        let target = percent(value, min, max);
        let motion = window.use_keyed_state(self.id.clone(), cx, move |_, cx| Motion {
            focus: cx.focus_handle(),
            left: 0.,
            width: FIRST_WIDTH,
            dragging: false,
            keyboard: false,
            pos: Channel::new(target),
            grab: Channel::new(1.),
        });
        motion.update(cx, |m, _| {
            if (m.pos.target() - target).abs() > 1e-4 {
                m.pos.animate(target, Curve::Spring(Spring::GLIDE), 0., reduce);
            }
            let want = if m.dragging { GRAB_SCALE } else { 1. };
            if (m.grab.target() - want).abs() > 1e-4 {
                m.grab.animate(want, Curve::Spring(Spring::GRAB), 0., reduce);
            }
            if m.pos.is_running() || m.grab.is_running() {
                window.request_animation_frame();
            }
        });
        let m = motion.read(cx);
        if crate::trace::on() && (m.pos.is_running() || m.grab.is_running()) {
            crate::trace::motion("range-slider", &format!("render while the handle moves: {:.1} of {:.1}", m.pos.value(), m.pos.target()));
        }
        let (focus, width, pos, grab) = (m.focus.clone(), m.width, m.pos.value(), m.grab.value());
        let (dragging, keyboard) = (m.dragging, m.keyboard);
        let (handle_x, fill_x) = geometry(width, pos);
        let clip = width - 2. * FILL_INSET;
        let dots = if self.ticks { ticks(min, max, step) } else { Vec::new() };
        let off = self.disabled;
        let on_change = self.on_change.clone();

        // The value for a pointer at window x, and telling the owner.
        let commit_at = {
            let (motion, on_change) = (motion.clone(), on_change.clone());
            Rc::new(move |x: f32, window: &mut Window, cx: &mut App| {
                let (left, width) = {
                    let m = motion.read(cx);
                    (m.left, m.width)
                };
                if width <= 0. {
                    return;
                }
                let ratio = ((x - left) / width).clamp(0., 1.);
                let next = snap(min + ratio * (max - min), min, max, step);
                crate::trace::motion("range-slider", &format!("input handled: {next}"));
                if let Some(f) = &on_change {
                    f(next, window, cx);
                }
            })
        };
        let track_measure = {
            let motion = motion.clone();
            measure(move |b, cx| {
                let (left, width) = (f32::from(b.origin.x), f32::from(b.size.width));
                if motion.read(cx).width != width || motion.read(cx).left != left {
                    motion.update(cx, |m, cx| {
                        m.left = left;
                        m.width = width;
                        cx.notify();
                    });
                }
            })
        };
        // While a drag is on, the pointer is followed wherever it goes, and the release ends it.
        let follow = {
            let (motion, commit) = (motion.clone(), commit_at.clone());
            canvas(
                |_, _, _| {},
                move |_, _, window, _| {
                    let (moving, commit_move) = (motion.clone(), commit.clone());
                    window.on_mouse_event(move |event: &MouseMoveEvent, _, window, cx| {
                        if moving.read(cx).dragging {
                            commit_move(f32::from(event.position.x), window, cx);
                        }
                    });
                    let ending = motion.clone();
                    window.on_mouse_event(move |_: &MouseUpEvent, _, _, cx| {
                        if ending.read(cx).dragging {
                            ending.update(cx, |m, cx| {
                                m.dragging = false;
                                cx.notify();
                            });
                        }
                    });
                },
            )
            .absolute()
            .size_0()
        };

        let (down_motion, down_commit, down_focus) = (motion.clone(), commit_at.clone(), focus.clone());
        let (key_motion, key_change) = (motion.clone(), on_change.clone());
        let key_focus = focus.clone();
        let ring = keyboard && key_focus.is_focused(window) && !off;

        div()
            .id(self.id)
            .debug_selector(|| "range-track".into())
            .relative()
            .flex()
            .h(px(TRACK_HEIGHT))
            .w_full()
            .items_center()
            .overflow_hidden()
            .rounded(radius::lg())
            .bg(theme.card)
            .when(off, |d| d.opacity(DISABLED))
            .when(!off, |d| {
                d.cursor_grab().on_mouse_down(MouseButton::Left, move |event, window, cx| {
                    down_motion.update(cx, |m, cx| {
                        m.dragging = true;
                        m.keyboard = false;
                        cx.notify();
                    });
                    down_focus.focus(window, cx);
                    down_commit(f32::from(event.position.x), window, cx);
                })
            })
            .child(track_measure)
            .child(follow)
            .child(
                // The fill: a full-size block that slides in behind the rounded clip.
                div().absolute().left(px(FILL_INSET)).right(px(FILL_INSET)).top_0().bottom_0().overflow_hidden().rounded(radius::lg()).child(
                    div()
                        .debug_selector(|| "range-fill".into())
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .left(px(fill_x))
                        .w(px(clip))
                        .rounded(radius::lg())
                        .bg(theme.foreground.opacity(FILL_ALPHA)),
                ),
            )
            .child(
                // Tick centres follow the handle's inset path.
                div().absolute().left(px(TICK_INSET)).right(px(TICK_INSET)).top_0().bottom_0().children(dots.iter().map(|t| {
                    let at = percent(*t, min, max) / 100. * (width - 2. * TICK_INSET);
                    div()
                        .absolute()
                        .top(px((TRACK_HEIGHT - TICK) / 2.))
                        .left(px(at - TICK / 2.))
                        .size(px(TICK))
                        .rounded_full()
                        .bg(theme.foreground.opacity(TICK_ALPHA))
                })),
            )
            .child({
                let height = HANDLE_HEIGHT * grab;
                div()
                    .id("range-handle")
                    .debug_selector(|| "range-handle".into())
                    .absolute()
                    .left(px(handle_x))
                    .top(px((TRACK_HEIGHT - height) / 2.))
                    .w(px(HANDLE_WIDTH))
                    .h(px(height))
                    .rounded_full()
                    .bg(theme.foreground)
                    .track_focus(&focus.tab_stop(!off))
                    .when(ring, |d| d.border(px(4.)).border_color(theme.foreground.opacity(FOCUS_ALPHA)))
                    .when(!off, |d| {
                        d.on_key_down(move |event: &KeyDownEvent, window, cx| {
                            let current = {
                                let m = key_motion.read(cx);
                                min + m.pos.target() / 100. * (max - min)
                            };
                            if let Some(next) = key_value(event.keystroke.key.as_str(), snap(current, min, max, step), min, max, step) {
                                cx.stop_propagation();
                                key_motion.update(cx, |m, _| m.keyboard = true);
                                if let Some(f) = &key_change {
                                    f(snap(next, min, max, step), window, cx);
                                }
                            }
                        })
                    })
            })
            .when(dragging, |d| d.cursor_grabbing())
    }
}

#[cfg(test)]
mod tests;
