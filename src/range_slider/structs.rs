use std::rc::Rc;

use gpui_kit::{
    App, ElementId, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    MouseMoveEvent, MouseUpEvent, ParentElement, RenderOnce, Styled, Window, canvas, div,
    prelude::FluentBuilder,
};

use super::helpers::{compact_geometry, geometry, key_value, percent, snap, ticks};
use super::types::{
    COMPACT_HEIGHT, ChangeHandler, DISABLED, FILL_ALPHA, FILL_INSET, FIRST_WIDTH, FOCUS_ALPHA,
    GRAB_SCALE, HANDLE_HEIGHT, HANDLE_WIDTH, KNOB, RAIL, RAIL_ALPHA, TICK, TICK_ALPHA, TICK_INSET,
    TRACK_HEIGHT,
};
use crate::scale::px;
use crate::{
    motion::{Channel, Curve, Spring},
    placement::measure,
    theme::{ActiveTheme, radius},
};

#[derive(IntoElement)]
pub struct RangeSlider {
    id: ElementId,
    pub(super) value: f32,
    pub(super) min: f32,
    pub(super) max: f32,
    pub(super) step: f32,
    pub(super) ticks: bool,
    pub(super) disabled: bool,
    pub(super) compact: bool,
    pub(super) on_change: Option<ChangeHandler>,
    pub(super) on_end: Option<ChangeHandler>,
}

impl RangeSlider {
    pub fn new(id: impl Into<ElementId>, value: f32) -> Self {
        Self {
            id: id.into(),
            value,
            min: 0.,
            max: 100.,
            step: 1.,
            ticks: true,
            disabled: false,
            compact: false,
            on_change: None,
            on_end: None,
        }
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

    /// A thin rail and a round knob, no dots, 20px tall: for a setting in a row.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
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

    /// The value when a change is done: on the release after a press or a drag, and after each key. For a value that
    /// is costly to apply as it moves, such as the interface's zoom.
    pub fn on_end(mut self, f: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_end = Some(Rc::new(f));
        self
    }
}

pub(super) struct Motion {
    focus: FocusHandle,
    /// The track's left edge and width in the last layout, in window pixels.
    pub(super) left: f32,
    pub(super) width: f32,
    dragging: bool,
    keyboard: bool,
    /// The value the pointer last asked for in this press.
    last: Option<f32>,
    /// The value the handle and the fill follow, as a percent, and how far the handle is stretched.
    pos: Channel,
    grab: Channel,
}

impl RenderOnce for RangeSlider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let (min, max) = (
            self.min,
            if self.max > self.min {
                self.max
            } else {
                self.min
            },
        );
        let step = if self.step > 0. { self.step } else { 1. };
        let value = self.value.clamp(min, max);
        let target = percent(value, min, max);
        let motion = window.use_keyed_state(self.id.clone(), cx, move |_, cx| Motion {
            focus: cx.focus_handle(),
            left: 0.,
            width: FIRST_WIDTH * crate::scale::zoom(),
            dragging: false,
            keyboard: false,
            last: None,
            pos: Channel::new(target),
            grab: Channel::new(1.),
        });
        motion.update(cx, |m, _| {
            if (m.pos.target() - target).abs() > 1e-4 {
                m.pos
                    .animate(target, Curve::Spring(Spring::GLIDE), 0., reduce);
            }
            let want = if m.dragging { GRAB_SCALE } else { 1. };
            if (m.grab.target() - want).abs() > 1e-4 {
                m.grab
                    .animate(want, Curve::Spring(Spring::GRAB), 0., reduce);
            }
            if m.pos.is_running() || m.grab.is_running() {
                window.request_animation_frame();
            }
        });
        let m = motion.read(cx);
        if crate::trace::on() && (m.pos.is_running() || m.grab.is_running()) {
            crate::trace::motion(
                "range-slider",
                &format!(
                    "render while the handle moves: {:.1} of {:.1}",
                    m.pos.value(),
                    m.pos.target()
                ),
            );
        }
        let (focus, width, pos, grab) = (
            m.focus.clone(),
            m.width / crate::scale::zoom(),
            m.pos.value(),
            m.grab.value(),
        );
        let (dragging, keyboard) = (m.dragging, m.keyboard);
        let (handle_x, fill_x) = geometry(width, pos);
        let clip = width - 2. * FILL_INSET;
        let dots = if self.ticks && !self.compact {
            ticks(min, max, step)
        } else {
            Vec::new()
        };
        let compact = self.compact;
        let off = self.disabled;
        let on_change = self.on_change.clone();
        let on_end = self.on_end.clone();

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
                motion.update(cx, |m, _| m.last = Some(next));
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
            let (motion, commit, on_end) = (motion.clone(), commit_at.clone(), on_end.clone());
            canvas(
                |_, _, _| {},
                move |_, _, window, _| {
                    let (moving, commit_move) = (motion.clone(), commit.clone());
                    window.on_mouse_event(move |event: &MouseMoveEvent, _, window, cx| {
                        if moving.read(cx).dragging {
                            commit_move(f32::from(event.position.x), window, cx);
                        }
                    });
                    let (ending, on_end) = (motion.clone(), on_end.clone());
                    window.on_mouse_event(move |_: &MouseUpEvent, _, window, cx| {
                        if ending.read(cx).dragging {
                            let last = ending.update(cx, |m, cx| {
                                m.dragging = false;
                                cx.notify();
                                m.last.take()
                            });
                            if let (Some(value), Some(f)) = (last, &on_end) {
                                f(value, window, cx);
                            }
                        }
                    });
                },
            )
            .absolute()
            .size_0()
        };

        let (down_motion, down_commit, down_focus) =
            (motion.clone(), commit_at.clone(), focus.clone());
        let (key_motion, key_change, key_end) = (motion.clone(), on_change.clone(), on_end.clone());
        let key_focus = focus.clone();
        let ring = keyboard && key_focus.is_focused(window) && !off;

        // The handle, in either look, takes the focus and the keys.
        let ring_color = theme.foreground.opacity(FOCUS_ALPHA);
        let keys = move |handle: gpui_kit::Stateful<gpui_kit::Div>| {
            handle
                .track_focus(&focus.tab_stop(!off))
                .when(ring, |d| d.border(px(4.)).border_color(ring_color))
                .when(!off, |d| {
                    d.on_key_down(move |event: &KeyDownEvent, window, cx| {
                        let current = {
                            let m = key_motion.read(cx);
                            min + m.pos.target() / 100. * (max - min)
                        };
                        if let Some(next) = key_value(
                            event.keystroke.key.as_str(),
                            snap(current, min, max, step),
                            min,
                            max,
                            step,
                        ) {
                            let next = snap(next, min, max, step);
                            cx.stop_propagation();
                            key_motion.update(cx, |m, _| m.keyboard = true);
                            if let Some(f) = &key_change {
                                f(next, window, cx);
                            }
                            if let Some(f) = &key_end {
                                f(next, window, cx);
                            }
                        }
                    })
                })
        };

        let base = div()
            .id(self.id)
            .debug_selector(|| "range-track".into())
            .relative()
            .flex()
            .h(px(if compact {
                COMPACT_HEIGHT
            } else {
                TRACK_HEIGHT
            }))
            .w_full()
            .items_center()
            .when(off, |d| d.opacity(DISABLED))
            .when(!off, |d| {
                d.cursor_grab()
                    .on_mouse_down(MouseButton::Left, move |event, window, cx| {
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
            .child(follow);
        let base = if compact {
            let (knob_left, fill) = compact_geometry(width, pos);
            let knob = KNOB * (1. + (grab - 1.) / 2.);
            let rail_top = (COMPACT_HEIGHT - RAIL) / 2.;
            base.child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(px(rail_top))
                    .h(px(RAIL))
                    .rounded_full()
                    .bg(theme.foreground.opacity(RAIL_ALPHA)),
            )
            .child(
                div()
                    .debug_selector(|| "range-fill".into())
                    .absolute()
                    .left_0()
                    .top(px(rail_top))
                    .w(px(fill))
                    .h(px(RAIL))
                    .rounded_full()
                    .bg(theme.foreground),
            )
            .child(keys(
                div()
                    .id("range-handle")
                    .debug_selector(|| "range-handle".into())
                    .absolute()
                    .left(px(knob_left + (KNOB - knob) / 2.))
                    .top(px((COMPACT_HEIGHT - knob) / 2.))
                    .size(px(knob))
                    .rounded_full()
                    .bg(theme.foreground),
            ))
        } else {
            base.overflow_hidden()
                .rounded(radius::lg())
                .bg(theme.card)
                .child(
                    // The fill: a full-size block that slides in behind the rounded clip.
                    div()
                        .absolute()
                        .left(px(FILL_INSET))
                        .right(px(FILL_INSET))
                        .top_0()
                        .bottom_0()
                        .overflow_hidden()
                        .rounded(radius::lg())
                        .child(
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
                    div()
                        .absolute()
                        .left(px(TICK_INSET))
                        .right(px(TICK_INSET))
                        .top_0()
                        .bottom_0()
                        .children(dots.iter().map(|t| {
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
                    keys(
                        div()
                            .id("range-handle")
                            .debug_selector(|| "range-handle".into())
                            .absolute()
                            .left(px(handle_x))
                            .top(px((TRACK_HEIGHT - height) / 2.))
                            .w(px(HANDLE_WIDTH))
                            .h(px(height))
                            .rounded_full()
                            .bg(theme.foreground),
                    )
                })
        };
        base.when(dragging, |d| d.cursor_grabbing())
    }
}
