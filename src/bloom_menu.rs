//! beui's BloomMenu (`components/motion/bloom-menu.tsx`): a "Create +" button that opens into a menu by growing, from its
//! own middle outward, into a 420px panel: a header ("Create" and a close cross) over a grid of three columns of
//! choices. It is the same box the whole way, so the button and the panel are one shape at two sizes, both centred on the
//! button, and it closes the same way back.
//!
//! Motion: the box grows on `{ stiffness: 300, damping: 32, mass: 0.9 }`, a folder opening with a touch of overshoot. The
//! panel's words come in after 120ms, over 200ms. The grid opens as an iris: a small box at its centre (45% of the height and
//! 34% of the width off each edge) that grows to the whole grid in 450ms after 80ms. Each choice comes up on
//! `{ stiffness: 440, damping: 34 }` after `100ms + 70ms` for each step of its distance from the grid's centre, so the four
//! corners arrive together. The button gives to 0.97 while it is pressed. Under Reduce Motion it all jumps.
//!
//! What gpui cannot draw is left out: the blur on the choices as they arrive, and the scale of their labels (their icons
//! do scale).
use gpui_kit::{
    Context, ElementId, EventEmitter, FocusHandle, Focusable, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Render, SharedString, StatefulInteractiveElement, Styled, Subscription, Window, deferred, div, prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::{
    icon::{Icon, IconName},
    motion::{Channel, Curve, Spring, ease},
    theme::{ActiveTheme, Theme},
    typography::TextSize,
};

/// The button: `h-11 w-36`, `rounded-2xl`.
const TRIGGER_W: f32 = 144.;
const TRIGGER_H: f32 = 44.;
const RADIUS: f32 = 16.;
/// The panel: the header, `px-4 py-3` round a line of words, and rows of cells of `px-3 py-6` round an icon and a label.
const PANEL_W: f32 = 420.;
const HEADER: f32 = 45.;
const CELL_W_PAD: f32 = 12.;
const CELL_H: f32 = 96.;
const COLUMNS: usize = 3;
/// The words arrive after this long, over this long; the iris after its own delay, over its own time.
const WORDS_DELAY: f32 = 0.12;
const WORDS_TIME: f32 = 0.2;
const IRIS_DELAY: f32 = 0.08;
const IRIS_TIME: f32 = 0.45;
/// The iris starts as this share of the grid's height and width cut off each side.
const IRIS_TOP: f32 = 0.45;
const IRIS_SIDE: f32 = 0.34;
/// A choice arrives after this long, and this much longer for each step from the centre.
const ITEM_DELAY: f32 = 0.1;
const ITEM_STEP: f32 = 0.07;
const ITEM_FROM: f32 = 0.85;
const PRESSED: f32 = 0.97;

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

/// The web's six choices.
pub fn default_items() -> Vec<BloomItem> {
    vec![
        BloomItem::new("Doc", IconName::Description),
        BloomItem::new("Board", IconName::Dashboard),
        BloomItem::new("Table", IconName::TableChart),
        BloomItem::new("Folder", IconName::Folder),
        BloomItem::new("Reminder", IconName::Notifications),
        BloomItem::new("Link", IconName::Link),
    ]
}

pub enum BloomEvent {
    /// A choice was made; the menu shuts.
    Select(SharedString),
    /// The menu opened or shut.
    Toggled(bool),
}

/// How many rows the grid has for `count` choices.
pub fn rows(count: usize) -> usize {
    count.div_ceil(COLUMNS).max(1)
}

/// The panel's size: 420 wide, and the header and the rows tall, with the border.
pub fn panel_size(count: usize) -> (f32, f32) {
    (PANEL_W, HEADER + grid_height(count) + 2.)
}

/// The grid's height: the rows, and the 1px line under each row but the last.
pub fn grid_height(count: usize) -> f32 {
    CELL_H * rows(count) as f32 + (rows(count) - 1) as f32
}

/// How far choice `index` is from the grid's centre, in cells: the four corners of a 3x2 grid are equally far.
pub fn distance(index: usize, count: usize) -> f32 {
    let (col, row) = ((index % COLUMNS) as f32, (index / COLUMNS) as f32);
    let (mid_col, mid_row) = ((COLUMNS - 1) as f32 / 2., (rows(count) - 1) as f32 / 2.);
    ((col - mid_col).powi(2) + (row - mid_row).powi(2)).sqrt()
}

/// The seconds before choice `index` starts to arrive.
pub fn delay(index: usize, count: usize) -> f32 {
    ITEM_DELAY + distance(index, count) * ITEM_STEP
}

/// The box `morph` of the way from the button to the panel, as its width and height; both share one centre.
pub fn box_size(morph: f32, count: usize) -> (f32, f32) {
    let (pw, ph) = panel_size(count);
    (TRIGGER_W + (pw - TRIGGER_W) * morph, TRIGGER_H + (ph - TRIGGER_H) * morph)
}

/// What the iris still cuts off the grid at `progress` (0 to 1): the top and bottom, and the two sides, as shares.
pub fn iris_cut(progress: f32) -> (f32, f32) {
    (IRIS_TOP * (1. - progress), IRIS_SIDE * (1. - progress))
}

pub struct BloomMenu {
    id: ElementId,
    items: Vec<BloomItem>,
    open: bool,
    morph: Channel,
    words: Channel,
    iris: Channel,
    arrive: Vec<Channel>,
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

fn hairline(theme: &Theme) -> gpui_kit::Hsla {
    theme.foreground.opacity(0.08)
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

#[cfg(test)]
mod tests;
