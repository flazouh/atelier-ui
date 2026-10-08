use std::rc::Rc;

use gpui_kit::{
    AnyElement, App, Bounds, ElementId, FocusHandle, Global, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Pixels, RenderOnce, Size, Styled, Window, anchored, deferred, div,
    point, prelude::FluentBuilder, relative,
};

use super::helpers::{cover, placement, watch};
use super::types::{Align, CloseHandler, HIDDEN_AFTER_FRAMES, Hang, MARGIN, PRIORITY, Side};
use crate::placement::measure;
use crate::scale::px;

/// Which popover opened last.
#[derive(Default)]
pub(super) struct Registry {
    pub(super) owner: Option<String>,
    seq: u64,
    /// For each open popover, the frames the window has drawn and the frame it was last drawn in. The element
    /// state is dropped once a popover is not drawn for a frame, so this is where the gap survives.
    pub(super) frames: std::collections::HashMap<String, Frames>,
    /// Frames drawn while a popover is open, counted by [`watch`]: how old a trigger's bounds are.
    pub(super) frame: u64,
    /// Each switchable popover's trigger, as last drawn, and the frame it was drawn in.
    pub(super) triggers: std::collections::HashMap<String, (Bounds<Pixels>, u64)>,
}

pub(super) struct Frames {
    /// Frames drawn since it opened, counted by [`watch`].
    pub(super) watched: u64,
    pub(super) drawn: u64,
    pub(super) close: Option<CloseHandler>,
}

impl Frames {
    /// One more frame drawn by the window. True when the popover has missed [`HIDDEN_AFTER_FRAMES`] of them.
    pub(super) fn tick(&mut self) -> bool {
        self.watched += 1;
        self.watched.saturating_sub(self.drawn) >= HIDDEN_AFTER_FRAMES
    }

    /// The popover was drawn in the frame just counted.
    pub(super) fn drew(&mut self) {
        self.drawn = self.watched;
    }
}

impl Global for Registry {}

/// What one popover remembers between frames.
struct PopState {
    was_open: bool,
    seq: u64,
    /// The trigger's bounds and the window's size when it opened.
    opened_at: Option<(Bounds<Pixels>, Size<Pixels>)>,
    /// Whether the window was the active one when it opened: losing that closes it.
    active_at_open: bool,
    /// Where a hung popover's point was in the last layout.
    marker: Option<Bounds<Pixels>>,
    /// While open with the owner keeping focus: hears Escape wherever focus is.
    pub(super) escape: Option<gpui_kit::Subscription>,
    pub(super) panel: FocusHandle,
}

#[derive(IntoElement)]
pub struct Popover {
    pub(super) id: ElementId,
    pub(super) open: bool,
    shown: Option<bool>,
    pub(super) anchor: Option<Bounds<Pixels>>,
    pub(super) side: Side,
    pub(super) align: Align,
    pub(super) gap: Option<f32>,
    pub(super) height: f32,
    pub(super) width: Option<Pixels>,
    min_width: Option<Pixels>,
    pub(super) on_close: Option<CloseHandler>,
    pub(super) return_focus: Option<FocusHandle>,
    panel_focus: Option<FocusHandle>,
    pub(super) child: Option<AnyElement>,
    pub(super) hang: Option<Hang>,
    pub(super) keep_focus: bool,
    pub(super) hole: bool,
    pub(super) switchable: bool,
}

impl Popover {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            open: false,
            shown: None,
            anchor: None,
            side: Side::Auto,
            align: Align::Start,
            gap: None,
            height: 240.,
            width: None,
            min_width: None,
            on_close: None,
            return_focus: None,
            panel_focus: None,
            child: None,
            hang: None,
            keep_focus: false,
            hole: false,
            switchable: false,
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Whether the panel is drawn. It is drawn while open; an owner that fades or slides the panel out
    /// keeps it drawn until the motion ends, with no backdrop.
    pub fn shown(mut self, shown: bool) -> Self {
        self.shown = Some(shown);
        self
    }

    /// The trigger's bounds in the window, from `crate::placement::measure`.
    pub fn anchor(mut self, bounds: Option<Bounds<Pixels>>) -> Self {
        self.anchor = bounds;
        self
    }

    pub fn side(mut self, side: Side) -> Self {
        self.side = side;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Space between the trigger and the panel.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = Some(gap);
        self
    }

    /// The panel's height, or a guess at it, to choose the side. It does not size the panel.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn min_width(mut self, width: Pixels) -> Self {
        self.min_width = Some(width);
        self
    }

    /// Called when the popover asks to close: a press outside, Escape, Tab, or a move of the trigger.
    /// The owner clears its `open` flag.
    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }

    /// Where focus goes back to when the popover closes: the trigger.
    pub fn return_focus(mut self, handle: &FocusHandle) -> Self {
        self.return_focus = Some(handle.clone());
        self
    }

    /// The handle focus moves to when it opens, when the owner has a control to focus (a filter field).
    /// Without one the panel itself takes focus.
    pub fn panel_focus(mut self, handle: &FocusHandle) -> Self {
        self.panel_focus = Some(handle.clone());
        self
    }

    /// For a picker with no trigger: hangs the panel from a point of the parent's box. The owner draws the
    /// popover as a child of the box it means.
    pub fn hang(mut self, hang: Hang) -> Self {
        self.hang = Some(hang);
        self
    }

    /// The owner keeps focus while the popover is open, and its keys work the list: the popover does not
    /// take focus when it opens.
    pub fn keep_focus(mut self) -> Self {
        self.keep_focus = true;
        self
    }

    /// Leaves the trigger uncovered: the backdrop is four strips around it, so the trigger and what is in it
    /// (a text field) stay live while the popover is open. A press anywhere else still only closes it.
    pub fn hole(mut self) -> Self {
        self.hole = true;
        self
    }

    /// Its trigger stays live while another switchable popover is open, and that one leaves it uncovered: a
    /// press on it closes the open one and opens this one. For a row of pickers, such as a model and a mode.
    pub fn switchable(mut self) -> Self {
        self.switchable = true;
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = Some(child.into_any_element());
        self
    }
}

impl RenderOnce for Popover {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(
            ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), "popover".into()),
            cx,
            |_, cx| PopState {
                was_open: false,
                seq: 0,
                opened_at: None,
                active_at_open: false,
                marker: None,
                escape: None,
                panel: cx.focus_handle(),
            },
        );
        // Its own state's id, not its element id: two copies of one view, such as two agent panels, give
        // their popovers the same element ids.
        let key = state.entity_id().to_string();
        let viewport = window.viewport_size();
        let active = window.is_window_active();
        let shown = self.shown.unwrap_or(self.open);
        // A popover hung from its parent takes the trigger's place from the marker it draws.
        let anchor_now = if self.hang.is_some() {
            state.read(cx).marker
        } else {
            self.anchor
        };
        if let (true, None, Some(trigger)) = (self.switchable, self.hang, anchor_now) {
            let registry = cx.default_global::<Registry>();
            let frame = registry.frame;
            registry.triggers.insert(key.clone(), (trigger, frame));
        }
        if !self.open {
            cx.default_global::<Registry>().frames.remove(&key);
            state.update(cx, |s, _| {
                s.was_open = false;
                s.opened_at = None;
                s.escape = None;
            });
        }
        let close = self.on_close.clone();
        let back = self.return_focus.clone();
        let ask_close = {
            let close = close.clone();
            move |window: &mut Window, cx: &mut App| {
                if let Some(close) = &close {
                    close(window, cx);
                }
            }
        };
        if self.open {
            {
                let registry = cx.default_global::<Registry>();
                let frames = registry.frames.entry(key.clone()).or_insert(Frames {
                    watched: 0,
                    drawn: 0,
                    close: None,
                });
                frames.drew();
                frames.close = close.clone();
            }
            let opening = !state.read(cx).was_open;
            if opening {
                let seq = {
                    let registry = cx.default_global::<Registry>();
                    registry.seq += 1;
                    registry.owner = Some(key.clone());
                    registry.seq
                };
                let target = self
                    .panel_focus
                    .clone()
                    .unwrap_or_else(|| state.read(cx).panel.clone());
                state.update(cx, |s, _| {
                    s.was_open = true;
                    s.seq = seq;
                    s.opened_at = anchor_now.map(|a| (a, viewport));
                    s.active_at_open = active;
                });
                watch(window, key.clone());
                if self.keep_focus {
                    // The owner has focus, so its keys never pass through the panel: hear Escape from the
                    // app, after the owner's own handling.
                    let ask = ask_close.clone();
                    let window_handle = window.window_handle();
                    let subscription = cx.observe_keystrokes(move |event, window, cx| {
                        if event.keystroke.key == "escape"
                            && window.window_handle() == window_handle
                        {
                            ask(window, cx);
                        }
                    });
                    state.update(cx, |s, _| s.escape = Some(subscription));
                } else {
                    window.defer(cx, move |window, cx| target.focus(window, cx));
                }
            }
            // Reasons to close that only the frame can see.
            let opened_at = state.read(cx).opened_at;
            let moved = match (opened_at, anchor_now) {
                (Some((was, size)), Some(now)) => {
                    (was.origin.x - now.origin.x).abs() > px(1.)
                        || (was.origin.y - now.origin.y).abs() > px(1.)
                        || size != viewport
                }
                _ => false,
            };
            let superseded = cx.default_global::<Registry>().owner.as_deref() != Some(key.as_str());
            let inactive = state.read(cx).active_at_open && !active;
            if !opening && (moved || superseded || inactive) {
                state.update(cx, |s, _| {
                    s.was_open = false;
                });
                let ask = ask_close.clone();
                window.defer(cx, move |window, cx| ask(window, cx));
                return div().into_any_element();
            }
            // An open popover draws every frame, so a frame that never comes means it is hidden.
            window.request_animation_frame();
        }
        let marker = self.hang.map(|hang| {
            let state = state.clone();
            let at = div().absolute().size_0().child(measure(move |bounds, cx| {
                let moved = state.read(cx).marker != Some(bounds);
                if moved {
                    state.update(cx, |s, _| s.marker = Some(bounds));
                }
            }));
            match hang {
                Hang::Left(x, y) => at.left(px(x)).top(px(y)),
                Hang::Right(x, y) => at.right(px(x)).top(px(y)),
                Hang::Centre(y) => at.left(relative(0.5)).top(px(y)),
            }
        });
        let wrap = |d: gpui_kit::Div| {
            if self.hang.is_some() {
                d.absolute().inset_0()
            } else {
                d
            }
        };
        let (Some(anchor), Some(child), true) = (anchor_now, self.child, shown) else {
            // Hung, and not measured yet: draw the marker, and come back for the measure.
            if self.open && self.hang.is_some() {
                window.request_animation_frame();
            }
            return wrap(div()).children(marker).into_any_element();
        };
        let (_, corner, at) = placement(
            anchor,
            self.side,
            match self.hang {
                Some(Hang::Right(..)) => Align::End,
                Some(Hang::Centre(_)) => Align::Center,
                _ => self.align,
            },
            self.gap
                .unwrap_or(if self.hang.is_some() { 0. } else { 4. })
                * crate::scale::zoom(),
            self.height * crate::scale::zoom(),
            f32::from(viewport.height),
        );
        let panel_focus = self
            .panel_focus
            .clone()
            .unwrap_or_else(|| state.read(cx).panel.clone());
        let tracked = self.panel_focus.is_none() && !self.keep_focus;
        let dismiss = {
            let ask = ask_close.clone();
            let back = back.clone();
            move |move_on: bool, window: &mut Window, cx: &mut App| {
                ask(window, cx);
                if let Some(back) = back.clone() {
                    window.defer(cx, move |window, cx| {
                        back.focus(window, cx);
                        if move_on {
                            window.focus_next(cx);
                        }
                    });
                }
            }
        };
        let backdrop = self.open.then(|| {
            let press = {
                let ask = ask_close.clone();
                move |_: &gpui_kit::MouseDownEvent, window: &mut Window, cx: &mut App| {
                    cx.stop_propagation();
                    ask(window, cx);
                }
            };
            // A strip of the backdrop: it takes every press, and nothing under it hears one.
            let strip = |left: Pixels, top: Pixels, width: Pixels, height: Pixels| {
                div()
                    .absolute()
                    .left(left)
                    .top(top)
                    .w(width)
                    .h(height)
                    .occlude()
                    .cursor_default()
                    .on_mouse_down(MouseButton::Left, press.clone())
                    .on_mouse_down(MouseButton::Right, press.clone())
                    .on_mouse_down(MouseButton::Middle, press.clone())
                    .on_mouse_up(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            };
            let mut holes: Vec<Bounds<Pixels>> = self.hole.then_some(anchor).into_iter().collect();
            if self.switchable {
                // Other triggers drawn in the last two frames; an older one belongs to a view no longer drawn.
                let registry = cx.default_global::<Registry>();
                let fresh = registry.frame.saturating_sub(1);
                holes.extend(
                    registry
                        .triggers
                        .iter()
                        .filter(|(k, (_, at))| **k != key && *at >= fresh)
                        .map(|(_, (b, _))| *b),
                );
            }
            let whole = div()
                .relative()
                .w(viewport.width)
                .h(viewport.height)
                .children(
                    cover(viewport, &holes)
                        .into_iter()
                        .map(|r| strip(r.left(), r.top(), r.size.width, r.size.height)),
                );
            deferred(anchored().position(point(px(0.), px(0.))).child(whole))
                .with_priority(PRIORITY)
        });
        let panel = deferred(
            anchored()
                .position(at)
                .anchor(corner)
                .snap_to_window_with_margin(px(MARGIN))
                .child(
                    div()
                        .id(ElementId::NamedChild(
                            std::sync::Arc::new(self.id.clone()),
                            "panel".into(),
                        ))
                        .occlude()
                        .when_some(self.width, |d, w| d.w(w))
                        .when_some(self.min_width, |d, w| d.min_w(w))
                        .when(tracked, |d| d.track_focus(&panel_focus))
                        .key_context("BeuiPopover")
                        .on_key_down(
                            move |event, window, cx| match event.keystroke.key.as_str() {
                                "escape" => {
                                    cx.stop_propagation();
                                    dismiss(false, window, cx);
                                }
                                "tab" => {
                                    cx.stop_propagation();
                                    dismiss(true, window, cx);
                                }
                                _ => {}
                            },
                        )
                        .child(child),
                ),
        )
        .with_priority(PRIORITY + 1);
        wrap(div())
            .children(marker)
            .children(backdrop)
            .child(panel)
            .into_any_element()
    }
}
