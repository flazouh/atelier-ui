use std::{collections::HashMap, sync::Arc, time::Instant};

use gpui_kit::{
    Anchor, Animation, AnimationExt, App, ElementId, Global, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, Styled, Window,
    base::{
        HoverCard,
        text::{
            InlineElement, InlineRenderContext, MarkdownNode, MarkdownParseContext, MarkdownPlugin,
            markdown_ast,
        },
    },
    div,
};

use super::helpers::{chip_number, open_delay, pill};
use super::types::{CLOSE_DELAY, PILL_BASELINE, PrOpenHandler};
use crate::scale::px;
use crate::{
    motion::{cubic_bezier, duration, ease},
    pr::PrChipData,
    pr_glance::{PrGlanceCard, pr_cards},
    theme::ActiveTheme,
};

#[derive(IntoElement)]
pub struct PrChip {
    id: ElementId,
    pub(super) pr: PrChipData,
    on_open: Option<PrOpenHandler>,
}

impl PrChip {
    pub fn new(id: impl Into<ElementId>, pr: PrChipData) -> Self {
        Self {
            id: id.into(),
            pr,
            on_open: None,
        }
    }

    pub fn on_open(
        mut self,
        handler: impl Fn(&PrChipData, &mut Window, &mut App) + Send + Sync + 'static,
    ) -> Self {
        self.on_open = Some(Arc::new(handler));
        self
    }

    fn on_open_handler(mut self, handler: Option<PrOpenHandler>) -> Self {
        self.on_open = handler;
        self
    }
}

/// Whether a chip's card is open, and when one last opened or closed, across every chip: the pointer going
/// from chip to chip opens the next card at once. A chip that leaves the screen with its card open leaves
/// `open` set until another card closes.
#[derive(Default)]
pub(super) struct Warmth {
    pub(super) open: bool,
    pub(super) changed: Option<Instant>,
}

impl Global for Warmth {}

impl RenderOnce for PrChip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let child =
            |name: &'static str| ElementId::NamedChild(Arc::new(self.id.clone()), name.into());
        let glance = window.use_keyed_state(child("glance"), cx, {
            let (id, pr, on_open) = (self.id.clone(), self.pr.clone(), self.on_open.clone());
            move |_, cx| PrGlanceCard::new(id, pr, on_open, cx)
        });
        glance.update(cx, |card, cx| card.set_pr(self.pr.clone(), cx));
        let delay = open_delay(cx);
        // A card that takes over from another swaps in place; only the first fades in.
        let animate = !cx.reduce_motion() && !delay.is_zero();
        let trigger = pill(&self.id, &self.pr, self.on_open.clone(), &theme, window, cx);
        let hover = child("hover");
        // Where the pill was last drawn: in the window's top half the card opens below it, else above.
        let high = window.use_keyed_state(child("high"), cx, |_, _| false);
        let anchor = if *high.read(cx) {
            Anchor::TopLeft
        } else {
            Anchor::BottomLeft
        };
        let (id, pr) = (self.id, self.pr);
        let card = HoverCard::new(hover)
            .anchor(anchor)
            .open_delay(delay)
            .close_delay(CLOSE_DELAY)
            .on_open_change(move |open, _, cx| {
                *cx.default_global::<Warmth>() = Warmth {
                    open: *open,
                    changed: Some(Instant::now()),
                };
                let handler = pr_cards(cx).read(cx).on_open.clone();
                if let Some(handler) = handler {
                    handler(&pr, *open, cx);
                }
            })
            .trigger(trigger)
            .content(move |_, _, _| {
                let body = div().my(px(6.)).child(glance.clone());
                let body = if animate {
                    body.with_animation(
                        ElementId::NamedChild(Arc::new(id.clone()), "enter".into()),
                        Animation::new(duration::REVEAL)
                            .with_easing(|t| cubic_bezier(ease::OUT, t)),
                        |card, t| card.opacity(t).relative().top(px(2. * (1. - t))),
                    )
                    .into_any_element()
                } else {
                    body.into_any_element()
                };
                div()
                    .id(ElementId::NamedChild(Arc::new(id.clone()), "card".into()))
                    .child(body)
            });
        div()
            .on_children_prepainted(move |bounds, window, cx| {
                let Some(pill) = bounds.first() else { return };
                let is_high = pill.top() < window.viewport_size().height / 2.;
                high.update(cx, |high, _| *high = is_high);
            })
            .child(card)
    }
}

/// Draws the links [`link_prs`](crate::pr_chip::link_prs) wrote as [`PrChip`]s.
pub struct PrChips {
    /// Keeps each chip's hover and copy state apart from other messages'.
    pub id: ElementId,
    pub chips: Arc<HashMap<u64, PrChipData>>,
    pub on_open: Option<PrOpenHandler>,
}

impl MarkdownPlugin for PrChips {
    fn name(&self) -> &str {
        "pr-chip"
    }

    fn parse(
        &self,
        node: &markdown_ast::Node,
        _cx: &MarkdownParseContext<'_>,
    ) -> Option<MarkdownNode> {
        let markdown_ast::Node::Link(link) = node else {
            return None;
        };
        let number = chip_number(&link.url)?;
        Some(MarkdownNode::new("pr-chip", number).text(format!("#{number}")))
    }

    fn render_inline(
        &self,
        node: &MarkdownNode,
        _context: &InlineRenderContext,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Option<InlineElement> {
        let number = *node.data::<u64>()?;
        let pr = self.chips.get(&number)?.clone();
        let id = ElementId::NamedChild(Arc::new(self.id.clone()), format!("pr-{number}").into());
        Some(
            InlineElement::new(PrChip::new(id, pr).on_open_handler(self.on_open.clone()))
                .with_baseline(px(PILL_BASELINE)),
        )
    }
}
