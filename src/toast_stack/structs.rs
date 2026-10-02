use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use gpui_kit::{
    Bounds, Context, ElementId, EventEmitter, FontWeight, Hsla, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Pixels, Render, SharedString, StatefulInteractiveElement,
    Styled, Task, Window, anchored, deferred, div, point, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    icon::{Icon, IconName},
    layout_motion::shifted,
    motion::{Channel, Curve, Spring, duration, ease, now},
    placement::measure,
    theme::ActiveTheme,
    typography::TextSize,
};
use super::types::{
    BORDER, DEFAULT_DURATION, EDGE_BOTTOM, EDGE_TOP, EDGE_X, ENTER_RISE, EXIT, EXIT_SLIDE, GAP,
    ICON_BOX, ICON_GLYPH, PAD, PILL, RADIUS, SWAP, SWAP_RISE, ToastEvent, ToastPosition,
    ToastStatus,
};
use super::helpers::{drawn, elastic, layer, lets_go, stack_width};

/// A toast, as plain data.
#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub id: SharedString,
    pub title: SharedString,
    pub description: Option<SharedString>,
    pub status: ToastStatus,
    /// The label of the action button, if it has one.
    pub action: Option<SharedString>,
    /// How long it stays: `None` is the stack's default and `Some(ZERO)` is until dismissed.
    pub duration: Option<Duration>,
    pub dismissible: bool,
}

impl Toast {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self { id: SharedString::default(), title: title.into(), description: None, status: ToastStatus::Neutral, action: None, duration: None, dismissible: true }
    }

    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = id.into();
        self
    }

    pub fn description(mut self, words: impl Into<SharedString>) -> Self {
        self.description = Some(words.into());
        self
    }

    pub fn status(mut self, status: ToastStatus) -> Self {
        self.status = status;
        self
    }

    pub fn action(mut self, label: impl Into<SharedString>) -> Self {
        self.action = Some(label.into());
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Stays until it is dismissed.
    pub fn sticky(self) -> Self {
        self.duration(Duration::ZERO)
    }

    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }
}

/// What [`ToastStack::update`] changes. A field left `None` stays as it was. A new duration also restarts the time.
#[derive(Clone, Debug, Default)]
pub struct ToastPatch {
    pub title: Option<SharedString>,
    pub description: Option<Option<SharedString>>,
    pub status: Option<ToastStatus>,
    pub duration: Option<Duration>,
}

pub(super) struct Item {
    pub(super) toast: Toast,
    created: Instant,
    timer: Option<Task<()>>,
    pub(super) enter: Channel,
    /// The toast as it was before its last change, and how far the swap has run.
    pub(super) swap: Option<(Toast, Channel)>,
    /// The sideways offset of a drag, springing back to zero.
    offset: Channel,
}

pub(super) struct Leaving {
    pub(super) toast: Toast,
    pub(super) bounds: Bounds<Pixels>,
    from_x: f32,
    pub(super) opacity: f32,
    run: Channel,
}

pub(super) struct Drag {
    pub(super) id: SharedString,
    start: f32,
    pub(super) last: (f32, Instant),
    pub(super) speed: f32,
}

pub struct ToastStack {
    pub(super) id: ElementId,
    pub(super) items: Vec<Item>,
    pub(super) leaving: Vec<Leaving>,
    pub(super) position: ToastPosition,
    pub(super) default_duration: Duration,
    pub(super) limit: Option<usize>,
    pub(super) max_visible: usize,
    seed: usize,
    pub(super) bounds: HashMap<SharedString, Bounds<Pixels>>,
    pub(super) drag: Option<Drag>,
    epoch: Instant,
}

impl EventEmitter<ToastEvent> for ToastStack {}

impl ToastStack {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            leaving: Vec::new(),
            position: ToastPosition::BottomRight,
            default_duration: DEFAULT_DURATION,
            limit: None,
            max_visible: 4,
            seed: 0,
            bounds: HashMap::new(),
            drag: None,
            epoch: now(),
        }
    }

    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    pub fn default_duration(mut self, duration: Duration) -> Self {
        self.default_duration = duration;
        self
    }

    /// At most this many toasts are kept: a new one pushes the oldest out.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn max_visible(mut self, max: usize) -> Self {
        self.max_visible = max;
        self
    }

    pub fn set_position(&mut self, position: ToastPosition, cx: &mut Context<Self>) {
        self.position = position;
        cx.notify();
    }

    pub fn toasts(&self) -> Vec<&Toast> {
        self.items.iter().map(|i| &i.toast).collect()
    }

    pub fn get(&self, id: &str) -> Option<&Toast> {
        self.items.iter().find(|i| i.toast.id.as_ref() == id).map(|i| &i.toast)
    }

    /// Adds a toast and returns its id (made up when the toast has none).
    pub fn show(&mut self, mut toast: Toast, cx: &mut Context<Self>) -> SharedString {
        if toast.id.is_empty() {
            self.seed += 1;
            toast.id = format!("toast-{}", self.seed).into();
        }
        let id = toast.id.clone();
        let reduce = cx.reduce_motion();
        let mut enter = Channel::new(if reduce { 1. } else { 0. });
        enter.animate(1., Curve::Spring(Spring::STACK), 0., reduce);
        self.items.push(Item { toast, created: now(), timer: None, enter, swap: None, offset: Channel::new(0.) });
        self.arm(&id, cx);
        if let Some(limit) = self.limit {
            while self.items.len() > limit {
                let oldest = self.items[0].toast.id.clone();
                self.dismiss(&oldest, cx);
            }
        }
        cx.notify();
        id
    }

    /// Changes a toast in place. A change of status, title or description swaps the old ones out.
    pub fn update(&mut self, id: &str, patch: ToastPatch, cx: &mut Context<Self>) {
        let reduce = cx.reduce_motion();
        let Some(item) = self.items.iter_mut().find(|i| i.toast.id.as_ref() == id) else { return };
        let before = item.toast.clone();
        if let Some(title) = patch.title {
            item.toast.title = title;
        }
        if let Some(description) = patch.description {
            item.toast.description = description;
        }
        if let Some(status) = patch.status {
            item.toast.status = status;
        }
        let changed = (before.title.clone(), before.description.clone(), before.status)
            != (item.toast.title.clone(), item.toast.description.clone(), item.toast.status);
        if changed && !reduce {
            let mut run = Channel::new(0.);
            run.animate(1., Curve::Ease(SWAP, ease::OUT), 0., false);
            item.swap = Some((before, run));
        }
        let restart = patch.duration.is_some();
        if let Some(duration) = patch.duration {
            item.toast.duration = Some(duration);
            item.created = now();
        }
        let id: SharedString = id.to_string().into();
        if restart {
            self.arm(&id, cx);
        }
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        let ids: Vec<SharedString> = self.items.iter().map(|i| i.toast.id.clone()).collect();
        for id in ids {
            self.dismiss(&id, cx);
        }
    }

    /// Takes a toast away. It leaves by a slide and a fade, drawn where it was.
    pub fn dismiss(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(at) = self.items.iter().position(|i| i.toast.id.as_ref() == id) else { return };
        let item = self.items.remove(at);
        let reduce = cx.reduce_motion();
        if let (Some(bounds), false) = (self.bounds.get(id).copied(), reduce) {
            let mut run = Channel::new(0.);
            run.animate(1., Curve::Ease(EXIT, ease::OUT), 0., false);
            self.leaving.push(Leaving { toast: item.toast, bounds, from_x: item.offset.value(), opacity: item.enter.value().clamp(0., 1.), run });
        }
        self.bounds.remove(id);
        if self.drag.as_ref().is_some_and(|d| d.id.as_ref() == id) {
            self.drag = None;
        }
        cx.notify();
    }

    /// Starts the clock that takes the toast away, unless it is meant to stay.
    fn arm(&mut self, id: &SharedString, cx: &mut Context<Self>) {
        let default = self.default_duration;
        let Some(item) = self.items.iter_mut().find(|i| &i.toast.id == id) else { return };
        let total = item.toast.duration.unwrap_or(default);
        if total.is_zero() {
            item.timer = None;
            return;
        }
        let remaining = total.saturating_sub(now().saturating_duration_since(item.created));
        let (weak, id) = (cx.entity().downgrade(), id.clone());
        item.timer = Some(cx.spawn(async move |_, cx| {
            cx.background_executor().timer(remaining).await;
            weak.update(cx, |stack, cx| stack.dismiss(&id, cx)).ok();
        }));
    }

    fn grab(&mut self, id: &SharedString, x: f32) {
        self.drag = Some(Drag { id: id.clone(), start: x, last: (x, now()), speed: 0. });
    }

    fn drag_to(&mut self, x: f32, cx: &mut Context<Self>) {
        let Some(drag) = self.drag.as_mut() else { return };
        let at = now();
        let dt = at.duration_since(drag.last.1).as_secs_f32();
        if dt > 0.004 {
            drag.speed = (x - drag.last.0) / dt;
            drag.last = (x, at);
        }
        let offset = elastic(x - drag.start);
        let id = drag.id.clone();
        if let Some(item) = self.items.iter_mut().find(|i| i.toast.id == id) {
            item.offset = Channel::new(offset);
        }
        cx.notify();
    }

    fn release(&mut self, x: f32, cx: &mut Context<Self>) {
        let Some(drag) = self.drag.take() else { return };
        if lets_go(x - drag.start, drag.speed) {
            self.dismiss(&drag.id, cx);
        } else if let Some(item) = self.items.iter_mut().find(|i| i.toast.id == drag.id) {
            item.offset.animate(0., Curve::Spring(Spring::STACK), 0., false);
        }
        cx.notify();
    }
}

impl Render for ToastStack {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let this = cx.entity().downgrade();
        self.leaving.retain(|l| l.run.is_running());
        for item in &mut self.items {
            if item.swap.as_ref().is_some_and(|(_, run)| !run.is_running()) {
                item.swap = None;
            }
        }
        let spin = (now().saturating_duration_since(self.epoch).as_secs_f32() % duration::SPIN.as_secs_f32()) / duration::SPIN.as_secs_f32();
        let animating = !self.leaving.is_empty()
            || self.items.iter().any(|i| i.enter.is_running() || i.offset.is_running() || i.swap.is_some() || i.toast.status == ToastStatus::Loading);
        if animating {
            window.request_animation_frame();
        }
        let viewport = window.viewport_size();
        let width = stack_width(f32::from(viewport.width));
        let list: Vec<(SharedString, Toast)> = self.items.iter().map(|i| (i.toast.id.clone(), i.toast.clone())).collect();
        let shown = drawn(&list, self.max_visible, self.position);

        let cards = shown.into_iter().map(|(id, toast)| {
            let item = self.items.iter().find(|i| i.toast.id == id).expect("a drawn toast is kept");
            let enter = item.enter.value();
            let offset = item.offset.value();
            let swap = item.swap.as_ref().map(|(before, run)| (before.clone(), run.value().clamp(0., 1.)));
            let details = toast.description.is_some() || toast.action.is_some();
            let can_dismiss = toast.dismissible;
            let element_id = |tag: &str| ElementId::NamedChild(Arc::new(self.id.clone()), format!("{tag}-{id}").into());

            // The icon: the new glyph in place, the old one rising out.
            let swap_t = swap.as_ref().map_or(1., |(_, t)| *t);
            let mut icon_box = div().relative().flex_none().size(px(ICON_BOX)).when(details, |d| d.mt(px(2.)));
            if let Some((before, _)) = &swap {
                icon_box = icon_box.child(layer(&theme, before, spin, -SWAP_RISE * swap_t, 1. - swap_t));
            }
            icon_box = icon_box.child(layer(&theme, &toast, spin, SWAP_RISE * (1. - swap_t), swap_t));

            // The words: the new ones in the flow, the old ones laid over them.
            let words = |t: &Toast, rise: f32, opacity: f32| {
                div()
                    .relative()
                    .top(px(rise))
                    .opacity(opacity)
                    .child(div().truncate().text_size(TextSize::Sm.font_size()).line_height(px(20.)).font_weight(FontWeight::MEDIUM).text_color(theme.foreground).child(t.title.clone()))
                    .when_some(t.description.clone(), |d, text| {
                        d.child(div().mt(px(2.)).line_clamp(2).text_size(TextSize::Xs.font_size()).line_height(px(16.)).text_color(theme.muted_foreground).child(text))
                    })
            };
            let mut content = div().relative().child(words(&toast, SWAP_RISE * (1. - swap_t), swap_t));
            if let Some((before, _)) = &swap {
                content = content.child(div().absolute().top_0().left_0().w_full().child(words(before, -SWAP_RISE * swap_t, 1. - swap_t)));
            }
            let action = toast.action.clone().map(|label| {
                let (this, id) = (this.clone(), id.clone());
                div()
                    .id(element_id("action"))
                    .debug_selector({
                        let id = id.clone();
                        move || format!("toast-action-{id}")
                    })
                    .mt(px(8.))
                    .h(px(PILL))
                    .px(px(12.))
                    .flex()
                    .items_center()
                    .w_auto()
                    .rounded_full()
                    .bg(theme.foreground.opacity(0.08))
                    .text_size(TextSize::Xs.font_size())
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground)
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.foreground.opacity(0.12)))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_click(move |_, _, cx| {
                        cx.stop_propagation();
                        this.update(cx, |_, cx| cx.emit(ToastEvent::Action(id.clone()))).ok();
                    })
                    .child(label)
            });
            let close = can_dismiss.then(|| {
                let (this, id) = (this.clone(), id.clone());
                div()
                    .id(element_id("close"))
                    .debug_selector({
                        let id = id.clone();
                        move || format!("toast-close-{id}")
                    })
                    .flex_none()
                    .size(px(PILL))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .text_color(theme.muted_foreground)
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.foreground.opacity(0.08)).text_color(theme.foreground))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_click(move |_, _, cx| {
                        cx.stop_propagation();
                        this.update(cx, |s, cx| s.dismiss(&id, cx)).ok();
                    })
                    .child(Icon::new(IconName::Close).size(px(ICON_GLYPH)))
            });
            let surface = div()
                .relative()
                .overflow_hidden()
                .rounded(px(RADIUS))
                .border_1()
                .border_color(theme.foreground.opacity(BORDER))
                .bg(theme.card)
                .p(px(PAD))
                .shadow(vec![gpui_kit::BoxShadow {
                    color: Hsla { a: (theme.shadow.a * 2.5).min(1.), ..theme.shadow },
                    offset: gpui_kit::point(px(0.), px(25.)),
                    blur_radius: px(50.),
                    spread_radius: px(-12.),
                    inset: false,
                }])
                .child(
                    div()
                        .flex()
                        .gap(px(PAD))
                        .when(details, |d| d.items_start())
                        .when(!details, |d| d.items_center())
                        .child(icon_box)
                        .child(div().flex_1().min_w_0().flex().flex_col().items_start().child(div().w_full().child(content)).children(action))
                        .children(close),
                );
            let (this_down, id_down) = (this.clone(), id.clone());
            let (bounds_this, bounds_id) = (this.clone(), id.clone());
            let moving = div()
                .id(element_id("toast"))
                .debug_selector({
                    let id = id.clone();
                    move || format!("toast-{id}")
                })
                .relative()
                .left(px(offset))
                .top(px(ENTER_RISE * (1. - enter.min(1.))))
                .opacity(enter.clamp(0., 1.))
                .when(can_dismiss && !reduce, |d| {
                    d.on_mouse_down(MouseButton::Left, move |event, _, cx| {
                        let x = f32::from(event.position.x);
                        this_down.update(cx, |s, _| s.grab(&id_down, x)).ok();
                    })
                })
                .child(surface);
            shifted(
                element_id("slot"),
                div()
                    .relative()
                    .w_full()
                    .child(measure(move |b, cx| {
                        bounds_this
                            .update(cx, |s, _| {
                                s.bounds.insert(bounds_id.clone(), b);
                            })
                            .ok();
                    }))
                    .child(moving),
            )
            .spring(Spring::STACK)
            .into_any_element()
        });

        let origin = point(px(0.), px(0.));
        let ghosts = self.leaving.iter().map(|l| {
            let t = l.run.value().clamp(0., 1.);
            let (icon_glyph, disc) = l.toast.status.tones(&theme, theme.card);
            div()
                .absolute()
                .left(l.bounds.origin.x - origin.x + px(l.from_x + EXIT_SLIDE * t))
                .top(l.bounds.origin.y - origin.y)
                .w(l.bounds.size.width)
                .h(l.bounds.size.height)
                .opacity(l.opacity * (1. - t))
                .rounded(px(RADIUS))
                .border_1()
                .border_color(theme.foreground.opacity(BORDER))
                .bg(theme.card)
                .p(px(PAD))
                .flex()
                .gap(px(PAD))
                .items_center()
                .child(div().flex_none().size(px(ICON_BOX)).flex().items_center().justify_center().rounded_full().bg(disc).child(Icon::new(l.toast.status.icon()).size(px(ICON_GLYPH)).color(icon_glyph)))
                .child(div().flex_1().min_w_0().truncate().text_size(TextSize::Sm.font_size()).font_weight(FontWeight::MEDIUM).text_color(theme.foreground).child(l.toast.title.clone()))
        });

        let column = div().flex().gap(px(GAP)).w(px(width)).flex_col().children(cards);
        let place = |d: gpui_kit::Div| match self.position {
            ToastPosition::TopLeft => d.left(px(EDGE_X)).top(px(EDGE_TOP)),
            ToastPosition::TopCenter => d.left(px((f32::from(viewport.width) - width) / 2.)).top(px(EDGE_TOP)),
            ToastPosition::TopRight => d.right(px(EDGE_X)).top(px(EDGE_TOP)),
            ToastPosition::BottomLeft => d.left(px(EDGE_X)).bottom(px(EDGE_BOTTOM)),
            ToastPosition::BottomCenter => d.left(px((f32::from(viewport.width) - width) / 2.)).bottom(px(EDGE_BOTTOM)),
            ToastPosition::BottomRight => d.right(px(EDGE_X)).bottom(px(EDGE_BOTTOM)),
        };
        let dragging = self.drag.is_some();
        let (move_this, up_this) = (this.clone(), this.clone());
        // The stack is fixed to the window, not to the box it is put in: it is drawn in a layer over everything, laid
        // out in the window's own coordinates. The layer has no hit box of its own, so what is under it stays live.
        let layer = div()
            .id(self.id.clone())
            .relative()
            .w(viewport.width)
            .h(viewport.height)
            .child(place(div().absolute()).child(column))
            .children(ghosts)
            .when(dragging, |d| {
                d.child(
                    div()
                        .absolute()
                        .inset_0()
                        .occlude()
                        .on_mouse_move(move |event, _, cx| {
                            let x = f32::from(event.position.x);
                            move_this.update(cx, |s, cx| s.drag_to(x, cx)).ok();
                        })
                        .on_mouse_up(MouseButton::Left, move |event, _, cx| {
                            let x = f32::from(event.position.x);
                            up_this.update(cx, |s, cx| s.release(x, cx)).ok();
                        }),
                )
            });
        div().absolute().size_0().child(deferred(anchored().position(point(px(0.), px(0.))).child(layer)).with_priority(crate::popover::PRIORITY + 2))
    }
}
