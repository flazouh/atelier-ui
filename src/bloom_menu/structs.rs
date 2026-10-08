use gpui_kit::{
    Context, ElementId, EventEmitter, FocusHandle, Focusable, FontWeight, InteractiveElement,
    IntoElement, MouseButton, ParentElement, Render, SharedString, StatefulInteractiveElement,
    Styled, Subscription, Window, deferred, div, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    icon::{Icon, IconName},
    motion::{Channel, Curve, Spring, ease},
    theme::ActiveTheme,
    typography::TextSize,
};
use super::types::{
    BloomEvent, CELL_H, CELL_W_PAD, COLUMNS, HEADER, IRIS_DELAY, IRIS_TIME, ITEM_FROM, PRESSED,
    RADIUS, TRIGGER_H, TRIGGER_W, WORDS_DELAY, WORDS_TIME,
};
use super::helpers::{box_size, delay, grid_height, hairline, iris_cut, panel_size, rows};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BloomItem {
    pub label: SharedString,
    pub icon: IconName,
}

impl BloomItem {
    pub fn new(label: impl Into<SharedString>, icon: IconName) -> Self {
        Self { label: label.into(), icon }
    }
}

pub struct BloomMenu {
    id: ElementId,
    items: Vec<BloomItem>,
    pub(super) open: bool,
    pub(super) morph: Channel,
    pub(super) words: Channel,
    pub(super) iris: Channel,
    pub(super) arrive: Vec<Channel>,
    tap: Channel,
    focus: FocusHandle,
    _escape: Option<Subscription>,
}

impl EventEmitter<BloomEvent> for BloomMenu {}

impl Focusable for BloomMenu {
    fn focus_handle(&self, _: &gpui_kit::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl BloomMenu {
    pub fn new(id: impl Into<ElementId>, items: Vec<BloomItem>, cx: &mut Context<Self>) -> Self {
        Self {
            id: id.into(),
            arrive: vec![Channel::new(0.); items.len()],
            items,
            open: false,
            morph: Channel::new(0.),
            words: Channel::new(0.),
            iris: Channel::new(0.),
            tap: Channel::new(1.),
            focus: cx.focus_handle(),
            _escape: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.open == open {
            return;
        }
        self.open = open;
        let reduce = cx.reduce_motion();
        let count = self.items.len();
        if open {
            self.morph.animate(1., Curve::Spring(Spring::FOLDER), 0., reduce);
            self.words = Channel::new(0.);
            self.words.animate(1., Curve::Ease(WORDS_TIME, ease::OUT), WORDS_DELAY, reduce);
            self.iris = Channel::new(0.);
            self.iris.animate(1., Curve::Ease(IRIS_TIME, ease::OUT), IRIS_DELAY, reduce);
            for (i, channel) in self.arrive.iter_mut().enumerate() {
                *channel = Channel::new(0.);
                channel.animate(1., Curve::Spring(Spring::BLOOM_ITEM), delay(i, count), reduce);
            }
            // Escape shuts it wherever focus is, as the web listens on the window.
            self._escape = Some(cx.observe_keystrokes(move |this, event, _, cx| {
                if event.keystroke.key == "escape" {
                    this.set_open(false, cx);
                }
            }));
        } else {
            // The panel's words go at once and the button comes back as the box shrinks.
            self.morph.animate(0., Curve::Spring(Spring::FOLDER), 0., reduce);
            self._escape = None;
        }
        cx.emit(BloomEvent::Toggled(open));
        cx.notify();
    }

    fn choose(&mut self, label: SharedString, cx: &mut Context<Self>) {
        cx.emit(BloomEvent::Select(label));
        self.set_open(false, cx);
    }
}

impl Render for BloomMenu {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let this = cx.entity().downgrade();
        let count = self.items.len();
        let m = self.morph.value();
        let moving = self.morph.is_running() || self.words.is_running() || self.iris.is_running() || self.tap.is_running() || self.arrive.iter().any(|c| c.is_running());
        if moving {
            window.request_animation_frame();
        }
        let (pw, ph) = panel_size(count);
        let pressed = self.tap.value();
        let (w, h) = box_size(m, count);
        let (w, h) = if self.open || m > 0.001 { (w, h) } else { (w * pressed, h * pressed) };
        let (open_this, close_this, escape_this, out_this) = (this.clone(), this.clone(), this.clone(), this.clone());
        let words = self.words.value().clamp(0., 1.);
        let iris = self.iris.value().clamp(0., 1.);
        let (cut_v, cut_h) = iris_cut(iris);

        // The panel's inside, at its full size, centred in the box and cut by it.
        let grid_w = pw - 2.;
        let cell_w = grid_w / COLUMNS as f32;
        let grid_h = grid_height(count);
        let cells = self.items.iter().enumerate().map(|(i, item)| {
            let label = item.label.clone();
            let click = this.clone();
            let arrive = self.arrive[i].value().clamp(0., 1.2);
            let (col, row) = (i % COLUMNS, i / COLUMNS);
            div()
                .id((self.id.clone(), format!("cell-{i}")))
                .debug_selector(move || format!("bloom-cell-{i}"))
                .absolute()
                .left(px(col as f32 * cell_w))
                .top(px(row as f32 * (CELL_H + 1.)))
                .w(px(cell_w))
                .h(px(if row + 1 < rows(count) { CELL_H + 1. } else { CELL_H }))
                .flex()
                .items_center()
                .justify_center()
                .px(px(CELL_W_PAD))
                .text_color(theme.muted_foreground)
                .cursor_pointer()
                .hover(|s| s.text_color(theme.foreground))
                .when(col != COLUMNS - 1, |d| d.border_r_1().border_color(hairline(&theme)))
                .when(row == 0 && rows(count) > 1, |d| d.border_b_1().border_color(hairline(&theme)))
                .on_click(move |_, _, cx| {
                    let label = label.clone();
                    click.update(cx, |s, cx| s.choose(label, cx)).ok();
                })
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(8.))
                        .opacity(arrive.min(1.))
                        .child(Icon::new(item.icon).size(px(20. * (ITEM_FROM + (1. - ITEM_FROM) * arrive.min(1.)))))
                        .child(div().text_size(TextSize::Sm.font_size()).line_height(px(20.)).font_weight(FontWeight::MEDIUM).child(item.label.clone())),
                )
        });
        let panel_inside = div()
            .absolute()
            .left(px((w - 2. - (pw - 2.)) / 2.))
            .top(px((h - 2. - (ph - 2.)) / 2.))
            .w(px(pw - 2.))
            .h(px(ph - 2.))
            .opacity(words)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(HEADER))
                    .px(px(16.))
                    .border_b_1()
                    .border_color(hairline(&theme))
                    .child(div().text_size(TextSize::Sm.font_size()).font_weight(FontWeight::MEDIUM).text_color(theme.muted_foreground).child("Create"))
                    .child(
                        div()
                            .id((self.id.clone(), "close"))
                            .debug_selector(|| "bloom-close".into())
                            .cursor_pointer()
                            .text_color(theme.muted_foreground)
                            .hover(|s| s.text_color(theme.foreground))
                            .on_click(move |_, _, cx| {
                                close_this.update(cx, |s, cx| s.set_open(false, cx)).ok();
                            })
                            .child(Icon::new(IconName::Close).size(px(16.))),
                    ),
            )
            .child(
                // The iris: a window on the grid that opens from its middle.
                div()
                    .relative()
                    .w(px(grid_w))
                    .h(px(grid_h))
                    .child(
                        div()
                            .absolute()
                            .left(px(grid_w * cut_h))
                            .top(px(grid_h * cut_v))
                            .w(px(grid_w * (1. - 2. * cut_h)))
                            .h(px(grid_h * (1. - 2. * cut_v)))
                            .overflow_hidden()
                            .child(div().absolute().left(px(-grid_w * cut_h)).top(px(-grid_h * cut_v)).w(px(grid_w)).h(px(grid_h)).children(cells)),
                    ),
            );
        // The button's label, centred in the box.
        let label = div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .gap(px(8.))
            .whitespace_nowrap()
            .text_size(TextSize::Sm.font_size())
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.foreground)
            .child("Create")
            .child(Icon::new(IconName::Add).size(px(16.)));

        let shape = div()
            .id((self.id.clone(), "box"))
            .debug_selector(|| "bloom-box".into())
            .absolute()
            .left(px((TRIGGER_W - w) / 2.))
            .top(px((TRIGGER_H - h) / 2.))
            .w(px(w))
            .h(px(h))
            .rounded(px(RADIUS))
            .border_1()
            .border_color(hairline(&theme))
            .bg(theme.card)
            .overflow_hidden()
            .track_focus(&self.focus)
            .when(self.open, |d| d.child(panel_inside))
            .when(!self.open, |d| {
                d.cursor_pointer()
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        escape_this.update(cx, |s, cx| {
                            s.tap.animate(PRESSED, Curve::Spring(Spring::PRESS), 0., cx.reduce_motion());
                            cx.notify();
                        })
                        .ok();
                    })
                    .on_click(move |_, _, cx| {
                        open_this.update(cx, |s, cx| {
                            s.tap.animate(1., Curve::Spring(Spring::PRESS), 0., cx.reduce_motion());
                            s.set_open(true, cx);
                        })
                        .ok();
                    })
                    .child(label)
            })
            .on_mouse_down_out(move |_, _, cx| {
                out_this.update(cx, |s, cx| {
                    if s.open {
                        s.set_open(false, cx);
                    }
                })
                .ok();
            });

        // The spacer keeps the button's place; the box is laid over it and drawn last, over the page.
        div()
            .id(self.id.clone())
            .relative()
            .flex_none()
            .w(px(TRIGGER_W))
            .h(px(TRIGGER_H))
            .child(deferred(div().absolute().top_0().left_0().w(px(TRIGGER_W)).h(px(TRIGGER_H)).child(shape)).with_priority(crate::popover::PRIORITY))
    }
}
