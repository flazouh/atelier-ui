use std::{rc::Rc, sync::Arc};

use gpui_kit::{
    App, ElementId, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, canvas,
    div, prelude::FluentBuilder, px,
};

use crate::{
    motion::{Channel, Curve, Spring, ease},
    theme::{ActiveTheme, mix},
    typography::TextSize,
};
use super::types::{
    BOX, CORNER, ChangeHandler, DISABLED, DRAW_DASH, DRAW_DELAY, DRAW_TICK, EDGE, EDGE_ALPHA,
    FADE, GAP, MARK, Mark, PRESS_SCALE, RING_ALPHA, TINT, Toggle,
};
use super::helpers::stroke;

#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    pub(super) checked: bool,
    pub(super) indeterminate: bool,
    pub(super) disabled: bool,
    pub(super) label: Option<SharedString>,
    pub(super) on_change: Option<ChangeHandler>,
    pub(super) focus: Option<FocusHandle>,
}

impl Checkbox {
    pub fn new(id: impl Into<ElementId>, checked: bool) -> Self {
        Self { id: id.into(), checked, indeterminate: false, disabled: false, label: None, on_change: None, focus: None }
    }

    /// The box's focus, from its owner, so the owner can place it in a Tab order or focus it.
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus = Some(handle.clone());
        self
    }

    /// A partial choice, such as "select all" over a mixed list. It shows a dash whatever `checked` says.
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Called with the value to move to: the opposite of `checked`.
    pub fn on_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

pub(super) struct Motion {
    pub(super) focus: FocusHandle,
    /// The mark on show, how far it has come in, and how far its stroke has drawn.
    pub(super) mark: Mark,
    appear: Channel,
    draw: Channel,
    /// The mark that is leaving, and how much of it is left.
    pub(super) leaving: Option<(Mark, Channel)>,
    /// 1 when the box wears the primary, 0 when it wears the plain edge.
    tone: Channel,
    pub(super) hover: Channel,
    scale: Channel,
    pub(super) keyboard: bool,
}

impl Motion {
    pub(super) fn new(cx: &mut App, mark: Mark) -> Self {
        let on = if mark == Mark::None { 0. } else { 1. };
        Self {
            focus: cx.focus_handle(),
            mark,
            appear: Channel::new(on),
            draw: Channel::new(on),
            leaving: None,
            tone: Channel::new(on),
            hover: Channel::new(0.),
            scale: Channel::new(1.),
            keyboard: false,
        }
    }
}

impl RenderOnce for Checkbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let want = Mark::of(self.checked, self.indeterminate);
        let motion = window.use_keyed_state(self.id.clone(), cx, move |_, cx| Motion::new(cx, want));

        motion.update(cx, |m, _| {
            if m.mark != want {
                // The old mark leaves while the new one comes in, as `AnimatePresence` does.
                if m.mark != Mark::None && !reduce {
                    let mut exit = Channel::new(m.appear.value().clamp(0., 1.));
                    exit.animate(0., Curve::Ease(FADE, ease::OUT), 0., false);
                    m.leaving = Some((m.mark, exit));
                } else {
                    m.leaving = None;
                }
                crate::trace::motion("checkbox", "mark told to change (first render with the new value)");
                m.mark = want;
                if want != Mark::None {
                    m.appear = Channel::new(0.);
                    m.appear.animate(1., Curve::Ease(FADE, ease::OUT), 0., reduce);
                    m.draw = Channel::new(0.);
                    let duration = if want == Mark::Dash { DRAW_DASH } else { DRAW_TICK };
                    m.draw.animate(1., Curve::Ease(duration, ease::OUT), DRAW_DELAY, reduce);
                }
            }
            let on = if want == Mark::None { 0. } else { 1. };
            if (m.tone.target() - on).abs() > 0.5 {
                m.tone.animate(on, TINT, 0., reduce);
            }
            if m.leaving.as_ref().is_some_and(|(_, c)| !c.is_running()) {
                m.leaving = None;
            }
            let running = m.appear.is_running() || m.draw.is_running() || m.tone.is_running() || m.hover.is_running() || m.scale.is_running() || m.leaving.is_some();
            if running {
                window.request_animation_frame();
            }
        });

        let m = motion.read(cx);
        if crate::trace::on() && (m.appear.is_running() || m.draw.is_running()) {
            crate::trace::motion("checkbox", &format!("render while the mark moves: appear={:.2} draw={:.2}", m.appear.value(), m.draw.value()));
        }
        let (focus, keyboard) = (self.focus.clone().unwrap_or_else(|| m.focus.clone()), m.keyboard);
        let (tone, hover, scale) = (m.tone.value().clamp(0., 1.), m.hover.value().clamp(0., 1.), m.scale.value());
        let (mark, appear, draw) = (m.mark, m.appear.value().clamp(0., 1.), m.draw.value().clamp(0., 1.));
        let leaving = m.leaving.as_ref().map(|(k, c)| (*k, c.value().clamp(0., 1.)));

        let rest = mix(theme.muted_foreground.opacity(EDGE_ALPHA), theme.muted_foreground, hover);
        let edge = mix(rest, theme.primary, tone);
        let fill = mix(theme.background, theme.primary, tone);
        let side = BOX * scale;
        let off = self.disabled;
        let mark_size = MARK * scale;
        let marks: Vec<(Mark, f32, f32)> = leaving.into_iter().map(|(k, exit)| (k, exit, 1.)).chain((mark != Mark::None).then_some((mark, appear, draw))).collect();
        // A tick is a mark, not words: the page when it reaches 3:1 on the fill.
        let ink = crate::theme::mark_on(&theme, theme.primary);

        let toggle: Toggle = {
            let (on_change, checked) = (self.on_change.clone(), self.checked);
            Rc::new(move |window, cx| {
                if let Some(f) = &on_change {
                    f(!checked, window, cx);
                }
            })
        };
        let (press, release, enter, leave, key_motion) = (motion.clone(), motion.clone(), motion.clone(), motion.clone(), motion.clone());
        let (click, key_toggle, click_focus) = (toggle.clone(), toggle, focus.clone());
        let ring = keyboard && focus.is_focused(window) && !off;

        let control = div()
            .id(ElementId::NamedChild(Arc::new(self.id.clone()), "box".into()))
            .debug_selector(|| "checkbox-box".into())
            .relative()
            .flex_none()
            .size(px(BOX))
            .flex()
            .items_center()
            .justify_center()
            .track_focus(&focus.tab_stop(!off))
            .when(!off, |d| {
                d.on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    let reduce = cx.reduce_motion();
                    press.update(cx, |m, cx| {
                        m.keyboard = false;
                        if !reduce {
                            m.scale.animate(PRESS_SCALE, Curve::Spring(Spring::PRESS), 0., false);
                        }
                        cx.notify();
                    });
                })
                .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                    release.update(cx, |m, cx| {
                        m.scale.animate(1., Curve::Spring(Spring::PRESS), 0., false);
                        cx.notify();
                    });
                })
                .on_hover(move |on, _, cx| {
                    let (reduce, entering) = (cx.reduce_motion(), *on);
                    let target = if entering { 1. } else { 0. };
                    let m = if entering { &enter } else { &leave };
                    m.update(cx, |m, cx| {
                        m.hover.animate(target, TINT, 0., reduce);
                        if !entering {
                            m.scale.animate(1., Curve::Spring(Spring::PRESS), 0., false);
                        }
                        cx.notify();
                    });
                })
                .on_click(move |_, window, cx| {
                    crate::trace::motion("checkbox", "click handled");
                    click_focus.focus(window, cx);
                    click(window, cx);
                })
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if matches!(event.keystroke.key.as_str(), "space" | "enter") {
                        cx.stop_propagation();
                        key_motion.update(cx, |m, _| m.keyboard = true);
                        key_toggle(window, cx);
                    }
                })
            })
            .when(off, |d| d.opacity(DISABLED))
            .child(
                div()
                    .debug_selector(|| "checkbox-face".into())
                    .absolute()
                    .size(px(side))
                    .rounded(px(CORNER * scale))
                    .border(px(EDGE * scale))
                    .border_color(edge)
                    .bg(fill)
                    .flex()
                    .items_center()
                    .justify_center()
                    .children(marks.into_iter().map(|(kind, shown, drawn)| {
                        let size = mark_size * (0.5 + 0.5 * shown);
                        div().debug_selector(|| "checkbox-mark".into()).absolute().size(px(size)).child(
                            canvas(|_, _, _| {}, move |bounds, _, window, _| stroke(bounds, kind.points(), drawn, ink.opacity(ink.a * shown), window))
                                .size_full(),
                        )
                    })),
            )
            .when(ring, |d| {
                let reach = 4.;
                d.child(
                    div()
                        .debug_selector(|| "checkbox-ring".into())
                        .absolute()
                        .top(px(-reach))
                        .left(px(-reach))
                        .size(px(BOX + 2. * reach))
                        .rounded(px(CORNER + reach))
                        .border(px(2.))
                        .border_color(theme.foreground.opacity(RING_ALPHA)),
                )
            });

        div()
            .id(self.id)
            .flex()
            .flex_none()
            .items_center()
            .gap(px(GAP))
            .when(off, |d| d.cursor_not_allowed())
            .when(!off, |d| d.cursor_pointer())
            .child(control)
            .children(self.label.map(|label| {
                div()
                    .text_size(TextSize::Sm.font_size())
                    .line_height(px(20.))
                    .text_color(theme.foreground)
                    .when(off, |d| d.opacity(DISABLED))
                    .child(label)
            }))
    }
}
