use std::{rc::Rc, sync::Arc};

use gpui_kit::{
    App, Bounds, ElementId, FocusHandle, FontWeight, Hsla, InteractiveElement, IntoElement,
    KeyDownEvent, MouseButton, ParentElement, Pixels, Point, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, transparent_black,
};

use super::helpers::{sizes, step, tab_stop};
use super::types::{
    ChangeHandler, DISC, DISC_EDGE, DISC_FILL, DOT_EDGE, GAP, OUTLINE_OFFSET, OUTLINE_WIDTH, PAD,
    PRESS_SCALE, RING_ALPHA, RING_WIDTH, UNAVAILABLE,
};
use crate::scale::px;
use crate::{
    motion::{Channel, Curve, Spring},
    placement::measure,
    theme::{ActiveTheme, mix},
    typography::TextSize,
};

/// One choice: what it is called in the group, the colour it shows, and the words a screen reader says.
#[derive(Clone, Debug)]
pub struct Swatch {
    pub value: SharedString,
    pub color: Hsla,
    pub label: SharedString,
    pub disabled: bool,
}

impl Swatch {
    pub fn new(
        value: impl Into<SharedString>,
        color: impl Into<Hsla>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            value: value.into(),
            color: color.into(),
            label: label.into(),
            disabled: false,
        }
    }

    /// A swatch that cannot be chosen: at 40%, and the keys pass over it.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(IntoElement)]
pub struct ColorSelector {
    id: ElementId,
    pub(super) label: Option<SharedString>,
    pub(super) swatches: Vec<Swatch>,
    pub(super) value: Option<SharedString>,
    pub(super) disabled: bool,
    pub(super) on_change: Option<ChangeHandler>,
}

impl ColorSelector {
    pub fn new(id: impl Into<ElementId>, swatches: impl IntoIterator<Item = Swatch>) -> Self {
        Self {
            id: id.into(),
            label: None,
            swatches: swatches.into_iter().collect(),
            value: None,
            disabled: false,
            on_change: None,
        }
    }

    /// The group's label above the swatches: `text-sm font-medium text-muted-foreground`, `mb-3`.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// The value chosen now. The owner keeps it and passes the new one from `on_change`.
    pub fn value(mut self, value: Option<SharedString>) -> Self {
        self.value = value;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, f: impl Fn(&SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

/// What the group remembers between frames.
pub(super) struct Motion {
    pub(super) focus: Vec<FocusHandle>,
    /// Where each swatch was in the last layout, in the window, and where the list starts.
    bounds: Vec<Option<Bounds<Pixels>>>,
    origin: Option<Point<Pixels>>,
    /// The ring's centre in the list in design pixels, and how far its colour has come from the old to the new.
    pub(super) x: Channel,
    pub(super) y: Channel,
    blend: Channel,
    /// The swatch the ring belongs to, its colour, and the colour it left.
    pub(super) at: Option<usize>,
    pub(super) color: Hsla,
    from_color: Hsla,
    /// Each swatch's press scale.
    pub(super) scale: Vec<Channel>,
    /// The last input was the keyboard: only then does a focused swatch show its outline.
    pub(super) keyboard: bool,
}

impl Motion {
    pub(super) fn new() -> Self {
        Self {
            focus: Vec::new(),
            bounds: Vec::new(),
            origin: None,
            x: Channel::new(0.),
            y: Channel::new(0.),
            blend: Channel::new(1.),
            at: None,
            color: transparent_black(),
            from_color: transparent_black(),
            scale: Vec::new(),
            keyboard: false,
        }
    }
}

impl RenderOnce for ColorSelector {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let motion = window.use_keyed_state(self.id.clone(), cx, |_, _| Motion::new());
        let count = self.swatches.len();
        let chosen = self
            .value
            .as_ref()
            .and_then(|v| self.swatches.iter().position(|s| &s.value == v));
        let stop = tab_stop(&self.swatches, self.value.as_ref());
        let group_off = self.disabled;

        // Keep a focus handle and a press channel for each swatch, and send the ring to the chosen one.
        let (handles, focused_visible) = motion.update(cx, |m, cx| {
            m.focus.resize_with(count, || cx.focus_handle());
            m.bounds.resize(count, None);
            m.scale.resize_with(count, || Channel::new(1.));
            if let (Some(i), Some(origin)) = (chosen, m.origin) {
                let target = m.bounds[i].map(|b| b.center() - origin);
                if let Some(target) = target {
                    let (tx, ty) = (
                        crate::scale::design(target.x),
                        crate::scale::design(target.y),
                    );
                    let color = self.swatches[i].color;
                    if m.at.is_none() {
                        // The first time the ring is drawn it is simply there.
                        m.x = Channel::new(tx);
                        m.y = Channel::new(ty);
                        m.color = color;
                        m.from_color = color;
                        m.blend = Channel::new(1.);
                        m.at = Some(i);
                    } else if m.at != Some(i)
                        || (m.x.target() - tx).abs() > 0.5
                        || (m.y.target() - ty).abs() > 0.5
                    {
                        if m.at != Some(i) {
                            m.from_color =
                                mix(m.from_color, m.color, m.blend.value().clamp(0., 1.));
                            m.color = color;
                            m.blend = Channel::new(0.);
                            m.blend
                                .animate(1., Curve::Spring(Spring::LAYOUT), 0., reduce);
                        }
                        crate::trace::motion(
                            "color-selector",
                            "ring told to move (first render with the new choice)",
                        );
                        m.x.animate(tx, Curve::Spring(Spring::LAYOUT), 0., reduce);
                        m.y.animate(ty, Curve::Spring(Spring::LAYOUT), 0., reduce);
                        m.at = Some(i);
                    }
                }
            }
            let running = m.x.is_running()
                || m.y.is_running()
                || m.blend.is_running()
                || m.scale.iter().any(|c| c.is_running());
            if running {
                window.request_animation_frame();
            }
            (m.focus.clone(), m.keyboard)
        });

        let m = motion.read(cx);
        if crate::trace::on() && m.x.is_running() {
            crate::trace::motion(
                "color-selector",
                &format!(
                    "render while the ring moves: x={:.1} of {:.1}",
                    m.x.value(),
                    m.x.target()
                ),
            );
        }
        let ring = chosen.and_then(|i| {
            m.origin?;
            m.at?;
            let s = m.scale.get(i).map_or(1., Channel::value);
            let color = mix(m.from_color, m.color, m.blend.value().clamp(0., 1.));
            Some((m.x.value(), m.y.value(), s, color.opacity(RING_ALPHA)))
        });
        let scales: Vec<f32> = m.scale.iter().map(Channel::value).collect();

        let child =
            |name: &str| ElementId::NamedChild(Arc::new(self.id.clone()), name.to_string().into());
        let list_measure = {
            let motion = motion.clone();
            measure(move |b, cx| {
                let changed = motion.read(cx).origin != Some(b.origin);
                if changed {
                    motion.update(cx, |m, cx| {
                        m.origin = Some(b.origin);
                        cx.notify();
                    });
                }
            })
        };

        // `border-black/5`: black has no place in the theme; the darkest of its two ends is the nearest.
        let dot_edge = if theme.appearance == crate::theme::Appearance::Dark {
            theme.background
        } else {
            theme.foreground
        }
        .opacity(DOT_EDGE);
        let swatches = self.swatches.clone();
        let items = self.swatches.iter().enumerate().map(|(i, swatch)| {
            let off = swatch.disabled || group_off;
            let scale = scales.get(i).copied().unwrap_or(1.);
            let (disc, dot, _) = sizes(scale);
            let is_chosen = chosen == Some(i);
            let item_measure = {
                let motion = motion.clone();
                measure(move |b, cx| {
                    let changed = motion.read(cx).bounds.get(i).copied().flatten() != Some(b);
                    if changed {
                        motion.update(cx, |m, cx| {
                            if let Some(slot) = m.bounds.get_mut(i) {
                                *slot = Some(b);
                            }
                            cx.notify();
                        });
                    }
                })
            };
            let handle = handles[i].clone();
            let select = self.on_change.clone();
            let value = swatch.value.clone();
            let (press, release, leave) = (motion.clone(), motion.clone(), motion.clone());
            let (key_motion, key_swatches, key_change) =
                (motion.clone(), swatches.clone(), self.on_change.clone());
            let (click_motion, click_handle) = (motion.clone(), handle.clone());
            let outline = focused_visible && handle.is_focused(window) && !off;
            div()
                .id(child(&format!("swatch-{i}")))
                .debug_selector(move || format!("color-swatch-{i}"))
                .relative()
                .flex_none()
                .size(px(DISC))
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .when(off, |d| d.opacity(UNAVAILABLE).cursor_not_allowed())
                .when(!off, |d| d.cursor_pointer())
                .track_focus(&handle.tab_stop(!off && stop == Some(i)))
                .child(item_measure)
                .when(!off, |d| {
                    d.on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        let reduce = cx.reduce_motion();
                        press.update(cx, |m, cx| {
                            m.keyboard = false;
                            if !reduce && let Some(c) = m.scale.get_mut(i) {
                                c.animate(PRESS_SCALE, Curve::Spring(Spring::PRESS), 0., false);
                            }
                            cx.notify();
                        });
                    })
                    .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                        release.update(cx, |m, cx| {
                            if let Some(c) = m.scale.get_mut(i) {
                                c.animate(1., Curve::Spring(Spring::PRESS), 0., false);
                            }
                            cx.notify();
                        });
                    })
                    .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                        leave.update(cx, |m, cx| {
                            if let Some(c) = m.scale.get_mut(i) {
                                c.animate(1., Curve::Spring(Spring::PRESS), 0., false);
                            }
                            cx.notify();
                        });
                    })
                    .on_click(move |_, window, cx| {
                        click_motion.update(cx, |m, _| m.keyboard = false);
                        click_handle.focus(window, cx);
                        crate::trace::motion("color-selector", "click handled");
                        // Choosing what is chosen already changes nothing, and says nothing.
                        if !is_chosen && let Some(f) = &select {
                            f(&value, window, cx);
                        }
                    })
                    .on_key_down(move |event: &KeyDownEvent, window, cx| {
                        let forward = match event.keystroke.key.as_str() {
                            "right" | "down" => true,
                            "left" | "up" => false,
                            _ => return,
                        };
                        cx.stop_propagation();
                        if let Some(next) = step(&key_swatches, i, forward) {
                            key_motion.update(cx, |m, cx| {
                                m.keyboard = true;
                                let focus = m.focus.get(next).cloned();
                                cx.notify();
                                if let Some(focus) = focus {
                                    window.focus(&focus, cx);
                                }
                            });
                            if let Some(f) = &key_change {
                                f(&key_swatches[next].value, window, cx);
                            }
                        }
                    })
                })
                .child(
                    // The disc, sunk to the press scale about its centre.
                    div()
                        .debug_selector(move || format!("color-disc-{i}"))
                        .absolute()
                        .size(px(disc))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_full()
                        .border_1()
                        .border_color(theme.foreground.opacity(DISC_EDGE))
                        .bg(theme.card.opacity(DISC_FILL))
                        .child(
                            div()
                                .size(px(dot))
                                .rounded_full()
                                .border_1()
                                .border_color(dot_edge)
                                .bg(swatch.color),
                        ),
                )
                .when(outline, |d| {
                    let reach = OUTLINE_OFFSET + OUTLINE_WIDTH;
                    d.child(
                        div()
                            .debug_selector(|| "color-outline".into())
                            .absolute()
                            .top(px(-reach))
                            .left(px(-reach))
                            .size(px(DISC + 2. * reach))
                            .rounded_full()
                            .border(px(OUTLINE_WIDTH))
                            .border_color(theme.foreground),
                    )
                })
        });

        let list = div()
            .id(child("list"))
            .relative()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(GAP))
            .p(px(PAD))
            .child(list_measure)
            .children(items)
            .children(ring.map(|(x, y, scale, color)| {
                let (_, _, outer) = sizes(scale);
                div()
                    .debug_selector(|| "color-ring".into())
                    .absolute()
                    .left(px(x - outer / 2.))
                    .top(px(y - outer / 2.))
                    .size(px(outer))
                    .rounded_full()
                    .border(px(RING_WIDTH * scale))
                    .border_color(color)
            }));

        div()
            .id(self.id.clone())
            .debug_selector(|| "color-selector".into())
            .min_w_0()
            .flex()
            .flex_col()
            .children(self.label.map(|label| {
                div()
                    .mb(px(12.))
                    .text_size(TextSize::Sm.font_size())
                    .line_height(px(20.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.muted_foreground)
                    .child(label)
            }))
            .child(list)
    }
}
