//! beui's Checkbox (`components/motion/checkbox.tsx`): a 20px box with a 2px edge and `rounded-md`, and an
//! optional label at `gap-3`. Unchecked, its edge is `muted-foreground` at 50% (full on hover) on the page's
//! fill; checked or partly checked, edge and fill are the primary and the mark is `primary-foreground`. Colours
//! change over 200ms. The mark is a 12px tick (or a dash for a partial choice) drawn with a 1.5px round
//! stroke: it fades and scales in from 50% over 160ms on `EASE_OUT` while the stroke draws itself, 300ms
//! for the tick (200ms for the dash) after a 40ms wait; on the way out it fades and shrinks over 160ms. A
//! press sinks the box to 92% on `SPRING_PRESS`. Keyboard focus draws a 2px ring, 2px outside the box.
//! Disabled is 60%. Under Reduce Motion nothing animates: the mark is there, whole.
//!
//! The web version also blurs the mark by 4px as it leaves; gpui has no blur, so it only fades and shrinks.
use std::{rc::Rc, sync::Arc};

use gpui_kit::{px, 
    App, Bounds, ElementId, FocusHandle, Hsla, InteractiveElement, IntoElement, KeyDownEvent, MouseButton, ParentElement,
    PathBuilder, Pixels, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, canvas, div,
    prelude::FluentBuilder, point, 
};

use crate::{
    motion::{Channel, Curve, Spring, ease},
    theme::{ActiveTheme, mix},
    typography::TextSize,
};

/// The box, its edge and corner: `h-5 w-5`, `border-2`, `rounded-md`.
const BOX: f32 = 20.;
const EDGE: f32 = 2.;
const CORNER: f32 = 6.;
/// The mark: a 12px svg on a 24-unit grid, stroked 3 units wide.
const MARK: f32 = 12.;
const STROKE: f32 = 3.;
/// The tick `M5 13l4 4L19 7` and the dash `M6 12h12`.
const TICK: [(f32, f32); 3] = [(5., 13.), (9., 17.), (19., 7.)];
const DASH: [(f32, f32); 2] = [(6., 12.), (18., 12.)];
/// The label sits `gap-3` from the box.
const GAP: f32 = 12.;
const PRESS_SCALE: f32 = 0.92;
/// The mark comes in and goes out over 160ms; the stroke draws for 300ms (200ms for the dash) after 40ms.
const FADE: f32 = 0.16;
const DRAW_TICK: f32 = 0.3;
const DRAW_DASH: f32 = 0.2;
const DRAW_DELAY: f32 = 0.04;
/// `transition-colors duration-200`, Tailwind's default curve.
const TINT: Curve = Curve::Ease(0.2, [0.4, 0., 0.2, 1.]);
const DISABLED: f32 = 0.6;
/// The unchecked edge is `border-muted-foreground/50`.
const EDGE_ALPHA: f32 = 0.5;
/// The focus ring is `--ring`, the strong border: the foreground at 12%.
const RING_ALPHA: f32 = 0.12;

pub type ChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type Toggle = Rc<dyn Fn(&mut Window, &mut App)>;

/// What the mark shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    None,
    Tick,
    Dash,
}

impl Mark {
    /// A partial choice wins over checked, as in the web version.
    pub fn of(checked: bool, indeterminate: bool) -> Self {
        if indeterminate {
            Mark::Dash
        } else if checked {
            Mark::Tick
        } else {
            Mark::None
        }
    }

    fn points(self) -> &'static [(f32, f32)] {
        match self {
            Mark::None => &[],
            Mark::Tick => &TICK,
            Mark::Dash => &DASH,
        }
    }
}

#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    checked: bool,
    indeterminate: bool,
    disabled: bool,
    label: Option<SharedString>,
    on_change: Option<ChangeHandler>,
    focus: Option<FocusHandle>,
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

/// The first `fraction` of a polyline, by length: the stroke drawing itself (`pathLength`).
pub fn prefix(points: &[(f32, f32)], fraction: f32) -> Vec<(f32, f32)> {
    let fraction = fraction.clamp(0., 1.);
    let length = |a: (f32, f32), b: (f32, f32)| ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
    let total: f32 = points.windows(2).map(|w| length(w[0], w[1])).sum();
    let mut left = total * fraction;
    let Some(&first) = points.first() else { return Vec::new() };
    let mut out = vec![first];
    for w in points.windows(2) {
        let run = length(w[0], w[1]);
        if left >= run {
            out.push(w[1]);
            left -= run;
        } else {
            let t = if run > 0. { left / run } else { 0. };
            out.push((w[0].0 + (w[1].0 - w[0].0) * t, w[0].1 + (w[1].1 - w[0].1) * t));
            break;
        }
    }
    out
}

struct Motion {
    focus: FocusHandle,
    /// The mark on show, how far it has come in, and how far its stroke has drawn.
    mark: Mark,
    appear: Channel,
    draw: Channel,
    /// The mark that is leaving, and how much of it is left.
    leaving: Option<(Mark, Channel)>,
    /// 1 when the box wears the primary, 0 when it wears the plain edge.
    tone: Channel,
    hover: Channel,
    scale: Channel,
    keyboard: bool,
}

impl Motion {
    fn new(cx: &mut App, mark: Mark) -> Self {
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

/// Paints the first `fraction` of `points` in `color`, 1.5px wide at 12px, with round ends and joints.
fn stroke(bounds: Bounds<Pixels>, points: &[(f32, f32)], fraction: f32, color: Hsla, window: &mut Window) {
    let drawn = prefix(points, fraction);
    if drawn.len() < 2 {
        return;
    }
    let unit = f32::from(bounds.size.width) / 24.;
    let at = |p: (f32, f32)| point(bounds.origin.x + px(p.0 * unit), bounds.origin.y + px(p.1 * unit));
    let mut path = PathBuilder::stroke(px(STROKE * unit));
    path.move_to(at(drawn[0]));
    for p in &drawn[1..] {
        path.line_to(at(*p));
    }
    if let Ok(path) = path.build() {
        window.paint_path(path, color);
    }
    // gpui strokes have butt ends: a dot at each end and joint makes them round.
    for p in &drawn {
        let mut dot = PathBuilder::fill();
        let (c, r) = (at(*p), STROKE * unit / 2.);
        for k in 0..16 {
            let a = k as f32 / 16. * std::f32::consts::TAU;
            let q = point(c.x + px(r * a.cos()), c.y + px(r * a.sin()));
            if k == 0 { dot.move_to(q) } else { dot.line_to(q) }
        }
        dot.close();
        if let Ok(dot) = dot.build() {
            window.paint_path(dot, color);
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

#[cfg(test)]
mod tests;
