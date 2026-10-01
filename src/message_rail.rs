//! beui's PreviewRail as the conversation's rail (`components/motion/preview-rail.tsx`, used by `message-scroller` with
//! `navigation="rail"`): a column of short ticks at the list's edge, one for each message you sent. The tick under the
//! pointer, or the one for the message in view, is full length; its neighbours are 68%, 44% and 25% of it, so the
//! column swells like a dock, each tick on `Spring::LAYOUT`. The pointer on a tick shows a card beside it with the
//! message's start and the start of the answer, which rises 4px and fades in over 180ms; a press scrolls to that message.
use std::rc::Rc;

use gpui_kit::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window,
    div, prelude::FluentBuilder,
};

use crate::{
    motion::{Channel, Curve, Spring, ease},
    scale::px,
    theme::{ActiveTheme, radius},
    typography::TextSize,
};

/// One message on the rail: its words, and the start of what came back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RailItem {
    pub label: SharedString,
    pub description: Option<SharedString>,
}

/// The longest a message's label and its answer's start are, as on the web.
pub const LABEL_CHARS: usize = 56;
pub const DESCRIPTION_CHARS: usize = 88;

/// `text` cut at a word near `limit` characters, with an ellipsis, as the web does: at the last space past 65% of the
/// limit, else at the limit.
pub fn excerpt(text: &str, limit: usize) -> String {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() <= limit {
        return text;
    }
    let cut: String = text.chars().take(limit).collect();
    let boundary = cut.rfind(' ').filter(|&b| cut[..b].chars().count() as f32 > limit as f32 * 0.65).unwrap_or(cut.len());
    format!("{}…", cut[..boundary].trim())
}

/// How long tick `index` is, from 0 to 1, with `lit` the tick that is full length: 1, then .68, .44, and .25 for the rest.
pub fn tick_scale(index: usize, lit: Option<usize>) -> f32 {
    match lit.map(|l| l.abs_diff(index)) {
        Some(0) => 1.,
        Some(1) => 0.68,
        Some(2) => 0.44,
        _ => 0.25,
    }
}

/// How tall each tick's box is: 14 as on the web, less when so many messages would not fit in `room`.
pub fn item_size(count: usize, room: f32) -> f32 {
    if count == 0 { 14. } else { (room / count as f32).clamp(4., 14.) }
}

struct RailMotion {
    hovered: Option<usize>,
    ticks: Vec<Channel>,
    /// 0 to 1: the card coming in for the tick last hovered.
    card: Channel,
    shown: Option<usize>,
}

type Handler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct MessageRail {
    id: ElementId,
    items: Vec<RailItem>,
    active: usize,
    on_select: Option<Handler>,
}

impl MessageRail {
    pub fn new(id: impl Into<ElementId>, items: Vec<RailItem>, active: usize) -> Self {
        Self { id: id.into(), items, active, on_select: None }
    }

    /// Runs with the message's number (from 0) when its tick is pressed.
    pub fn on_select(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for MessageRail {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduce = cx.reduce_motion();
        let theme = cx.theme().clone();
        let count = self.items.len();
        let motion = window.use_keyed_state(self.id.clone(), cx, |_, _| RailMotion { hovered: None, ticks: Vec::new(), card: Channel::new(0.), shown: None });
        let active = self.active.min(count.saturating_sub(1));
        let (hovered, lit, scales, card, shown) = motion.update(cx, |m, _| {
            let lit = m.hovered.or(Some(active)).filter(|_| count > 0);
            m.ticks.resize_with(count, || Channel::new(0.25));
            for (i, tick) in m.ticks.iter_mut().enumerate() {
                let want = tick_scale(i, lit);
                if tick.target() != want {
                    tick.animate(want, Curve::Spring(Spring::LAYOUT), 0., reduce);
                }
            }
            let moving = m.ticks.iter().any(Channel::is_running) || m.card.is_running();
            (m.hovered, lit, m.ticks.iter().map(Channel::value).collect::<Vec<_>>(), m.card.value(), (m.shown, moving))
        });
        let (shown, moving) = shown;
        if moving {
            window.request_animation_frame();
        }
        let size = item_size(count, f32::from(window.viewport_size().height) / crate::scale::zoom() * 0.6);
        let on_select = self.on_select;
        let ticks = self.items.iter().enumerate().map(|(i, _)| {
            let (enter, leave, press) = (motion.clone(), motion.clone(), on_select.clone());
            let highlighted = lit == Some(i);
            div()
                .id(ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), format!("tick-{i}").into()))
                .debug_selector(move || format!("rail-tick-{i}"))
                .flex()
                .flex_none()
                .items_center()
                .justify_end()
                .w(px(28.))
                .h(px(size))
                .cursor_pointer()
                .on_hover(move |on, _, cx| {
                    let reduce = cx.reduce_motion();
                    let (target, other) = if *on { (Some(i), enter.clone()) } else { (None, leave.clone()) };
                    other.update(cx, |m, cx| {
                        if *on {
                            // A new tick under the pointer brings its card in afresh.
                            if m.shown != Some(i) {
                                m.card = Channel::new(0.);
                            }
                            m.shown = Some(i);
                            m.card.animate(1., Curve::Ease(0.18, ease::OUT), 0., reduce);
                        } else if m.hovered == Some(i) {
                            m.card = Channel::new(0.);
                            m.shown = None;
                        }
                        if *on || m.hovered == Some(i) {
                            m.hovered = target;
                        }
                        cx.notify();
                    });
                })
                .when_some(press, |d, press| d.on_click(move |_, window, cx| press(i, window, cx)))
                .child(div().h(px(1.5)).w(px(16. * scales.get(i).copied().unwrap_or(0.25))).rounded_full().bg(if highlighted { theme.foreground } else { theme.muted_foreground }))
        });
        // The card: beside the hovered tick, to its left, rising and fading in.
        let card_el = shown.filter(|_| hovered.is_some()).and_then(|i| self.items.get(i).map(|item| (i, item.clone()))).map(|(i, item)| {
            div()
                .absolute()
                .right(px(32.))
                .top(px(size * i as f32 + size / 2. - 40.))
                .w(px(256.))
                .h(px(80.))
                .opacity(card)
                .mt(px(4. * (1. - card)))
                .overflow_hidden()
                .rounded(radius::xl())
                .bg(theme.popover)
                .shadow(crate::theme::popover_shadow(&theme))
                .p(px(12.))
                .flex()
                .flex_col()
                .gap(px(4.))
                .debug_selector(|| "rail-card".into())
                .child(div().truncate().text_size(TextSize::Xs.font_size()).font_weight(gpui_kit::FontWeight::MEDIUM).text_color(theme.foreground).child(item.label))
                .children(item.description.map(|d| div().line_clamp(2).text_size(TextSize::Xs.font_size()).text_color(theme.muted_foreground).child(d)))
        });
        div()
            .absolute()
            .top_0()
            .bottom_0()
            .right(px(4.))
            .w(px(28.))
            .flex()
            .flex_col()
            .justify_center()
            .debug_selector(|| "message-rail".into())
            // The column of ticks is the card's frame, so the card's top is measured from the first tick.
            .child(div().relative().flex().flex_col().children(ticks).children(card_el))
    }
}

#[cfg(test)]
mod tests;
