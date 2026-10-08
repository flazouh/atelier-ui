use std::rc::Rc;

use gpui_kit::{
    AnyElement, App, ElementId, FocusHandle, FontWeight, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder,
};

use super::helpers::colors;
use super::types::{HEIGHT, PILL, Press, RADIUS, SHELL};
use crate::scale::px;
use crate::{
    focus::ring_shadow,
    icon::{Icon, IconName},
    motion::{Animated, FrameClock},
    number::Digits,
    placement::measure,
    spinner::Spinner,
    theme::ActiveTheme,
    typography::FONT_FAMILY,
};

struct State {
    pub(super) content: Option<(f32, f32)>,
    pub(super) width: Animated,
    pub(super) height: Animated,
    sized: bool,
    pub(super) clock: FrameClock,
    focus: Option<FocusHandle>,
}

/// The shell: its content, and the least size of the pill.
#[derive(IntoElement)]
pub struct Island {
    pub(super) id: ElementId,
    min: (f32, f32),
    pub(super) child: Option<AnyElement>,
    pub(super) on_press: Option<Press>,
    selector: Option<&'static str>,
}

impl Island {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            min: PILL,
            child: None,
            on_press: None,
            selector: None,
        }
    }

    /// The least width and height of the pill, in px.
    pub fn min_size(mut self, width: f32, height: f32) -> Self {
        self.min = (width, height);
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = Some(child.into_any_element());
        self
    }

    pub fn on_press(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_press = Some(Rc::new(f));
        self
    }

    pub fn debug_name(mut self, name: &'static str) -> Self {
        self.selector = Some(name);
        self
    }
}

impl RenderOnce for Island {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let min = self.min;
        let state = window.use_keyed_state(self.id.clone(), cx, move |_, _| State {
            content: None,
            width: Animated::new(SHELL, min.0),
            height: Animated::new(SHELL, min.1),
            sized: false,
            clock: FrameClock::default(),
            focus: None,
        });
        let (size, moving, focus) = state.update(cx, |s, cx| {
            let focus = s.focus.get_or_insert_with(|| cx.focus_handle()).clone();
            if let Some((w, h)) = s.content {
                let (w, h) = (w.max(min.0), h.max(min.1));
                if !s.sized || reduce {
                    s.width = Animated::new(SHELL, w);
                    s.height = Animated::new(SHELL, h);
                    s.sized = true;
                }
                s.width.set_target(w);
                s.height.set_target(h);
            }
            let dt = s.clock.tick();
            let mut moving = s.width.step(dt, reduce) | s.height.step(dt, reduce);
            moving |= s.content.is_none();
            if !moving {
                s.clock.rest();
            }
            ((s.width.value(), s.height.value()), moving, focus)
        });
        if moving {
            window.request_animation_frame();
        }
        let report = state.clone();
        let content = div()
            .relative()
            .flex_none()
            .child(measure(move |b, cx| {
                let next = (f32::from(b.size.width), f32::from(b.size.height));
                report.update(cx, |s, _| {
                    if s.content != Some(next) {
                        s.content = Some(next);
                    }
                })
            }))
            .children(self.child);
        let keyed = focus.is_focused(window) && window.last_input_was_keyboard();
        let press = self.on_press.clone();
        div()
            .id(self.id.clone())
            .relative()
            .flex()
            .flex_none()
            .items_start()
            .justify_center()
            .overflow_hidden()
            .w(px(size.0))
            .h(px(size.1))
            .rounded(px(RADIUS))
            .bg(colors(&theme).0)
            .text_color(colors(&theme).1)
            .font_family(FONT_FAMILY)
            .when(keyed, |d| d.shadow(ring_shadow(&theme, theme.background)))
            .when_some(self.selector, |d, name| {
                d.debug_selector(move || name.into())
            })
            .when_some(press, |d, press| {
                let key = press.clone();
                d.cursor_pointer()
                    .track_focus(&focus.tab_stop(true))
                    .on_click(move |_, window, cx| press(window, cx))
                    .on_key_down(move |event, window, cx| {
                        if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                            cx.stop_propagation();
                            key(window, cx);
                        }
                    })
            })
            .child(content)
    }
}

/// How many sessions are in each state that matters to the reader.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IslandCounts {
    /// The agent works.
    pub running: usize,
    /// A tool waits for a yes or a no, or the agent asked a question.
    pub needs: usize,
    /// A turn ended and the reader has not looked yet.
    pub done: usize,
}

impl IslandCounts {
    pub fn is_empty(self) -> bool {
        self.running + self.needs + self.done == 0
    }
}

/// The island of the running sessions.
#[derive(IntoElement)]
pub struct SessionsIsland {
    pub(super) id: ElementId,
    pub(super) counts: IslandCounts,
    pub(super) on_press: Option<Press>,
}

impl SessionsIsland {
    pub fn new(id: impl Into<ElementId>, counts: IslandCounts) -> Self {
        Self {
            id: id.into(),
            counts,
            on_press: None,
        }
    }

    /// A press on the pill: open the session that needs the reader most.
    pub fn on_press(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_press = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for SessionsIsland {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        if self.counts.is_empty() {
            return div().into_any_element();
        }
        let theme = cx.theme().clone();
        let c = self.counts;
        let part = |key: &'static str, mark: AnyElement, n: usize, words: &'static str| {
            div()
                .flex()
                .flex_none()
                .items_center()
                .gap(px(6.))
                .child(mark)
                .child(Digits::new((self.id.clone(), key), n.to_string(), px(12.)))
                .child(div().child(words))
        };
        let dot = |color: gpui_kit::Hsla| {
            div()
                .size(px(8.))
                .flex_none()
                .rounded_full()
                .bg(color)
                .into_any_element()
        };
        let content = div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .gap(px(12.))
            .px(px(14.))
            .py(px(6.))
            .text_size(px(12.))
            .line_height(px(16.))
            .font_weight(FontWeight::MEDIUM)
            .whitespace_nowrap()
            .when(c.needs > 0, |d| {
                d.child(part(
                    "needs",
                    Icon::new(IconName::PriorityHigh)
                        .size(px(12.))
                        .color(theme.warning_fill)
                        .into_any_element(),
                    c.needs,
                    "needs you",
                ))
            })
            .when(c.running > 0, |d| {
                d.child(part(
                    "running",
                    Spinner::new((self.id.clone(), "spin"))
                        .size(px(12.))
                        .color(theme.foreground)
                        .into_any_element(),
                    c.running,
                    "running",
                ))
            })
            .when(c.done > 0, |d| {
                d.child(part("done", dot(theme.accent), c.done, "done"))
            });
        let mut island = Island::new(self.id.clone())
            .min_size(0., HEIGHT)
            .debug_name("sessions-island")
            .child(content);
        if let Some(press) = self.on_press {
            island = island.on_press(move |window, cx| press(window, cx));
        }
        island.into_any_element()
    }
}
