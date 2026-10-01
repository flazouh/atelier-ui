//! A pull request named inline, as a small pill: its state mark and `#3344`. Pressing it reports it.
//!
//! Hovering opens a card by it, after gpui-base's `HoverCard` delay: `owner/repo`, the state, the title,
//! and two small ghost buttons, Copy link and Open in browser. The card is the app's one popover look,
//! Select's menu: a `card` fill, no border, and [`popover_shadow`]. The buttons live in that card, over
//! the text, not in the
//! pill: space reserved in the pill would leave a gap in the prose at rest, and buttons that grew the
//! pill on hover would push the words after it. The card fades and rises 4px in on
//! [`duration::REVEAL`].
//!
//! [`crate::agent_text::AgentText`] makes chips from agent text: [`link_prs`] rewrites each `#N` the app
//! knows into a link to `atelier-pr:N`, and [`PrChips`], a Markdown plugin, draws those links as chips.

use std::{collections::HashMap, sync::Arc};

use gpui_kit::{
    Anchor, Animation, AnimationExt, App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window,
    base::{
        HoverCard,
        text::{InlineElement, InlineRenderContext, MarkdownNode, MarkdownParseContext, MarkdownPlugin, markdown_ast},
    },
    div, prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::{
    focus::PressStop,
    icon::Icon,
    motion::{cubic_bezier, duration, ease},
    button::{Button, ButtonSize, ButtonVariant},
    copy_feedback::CopyFeedback,
    icon::IconName,
    pr::PrChipData,
    pr_card::LinkActions,
    pr_refs::pr_refs,
    theme::{ActiveTheme, mix, popover_shadow, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};

/// The link scheme [`link_prs`] writes and [`PrChips`] reads.
const SCHEME: &str = "atelier-pr:";

/// The pill's height, and where its text sits, so the pill lines up with the words around it.
const PILL_HEIGHT: f32 = 20.;
const PILL_BASELINE: f32 = 14.;

/// The hover card's widest, so the title wraps at a comfortable measure.
const CARD_MAX_WIDTH: f32 = 320.;

/// A callback for a pressed chip. It must be `Send + Sync` because Markdown renderers are.
pub type PrOpenHandler = Arc<dyn Fn(&PrChipData, &mut Window, &mut App) + Send + Sync>;

#[derive(IntoElement)]
pub struct PrChip {
    id: ElementId,
    pr: PrChipData,
    on_open: Option<PrOpenHandler>,
}

impl PrChip {
    pub fn new(id: impl Into<ElementId>, pr: PrChipData) -> Self {
        Self { id: id.into(), pr, on_open: None }
    }

    pub fn on_open(mut self, handler: impl Fn(&PrChipData, &mut Window, &mut App) + Send + Sync + 'static) -> Self {
        self.on_open = Some(Arc::new(handler));
        self
    }

    fn on_open_handler(mut self, handler: Option<PrOpenHandler>) -> Self {
        self.on_open = handler;
        self
    }
}

impl RenderOnce for PrChip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let reduce = cx.reduce_motion();
        let child = |name: &'static str| ElementId::NamedChild(Arc::new(self.id.clone()), name.into());
        let actions = window.use_keyed_state(child("actions"), cx, |_, _| LinkActions::default());
        let (pr, on_open) = (self.pr.clone(), self.on_open.clone());

        let pill = div()
            .id(child("pill"))
            .flex()
            .items_center()
            .gap(px(4.))
            .h(px(PILL_HEIGHT))
            .px(px(6.))
            .rounded_full()
            .bg(theme.card_strong)
            .hover(|s| s.bg(mix(theme.card_strong, theme.foreground, 0.06)))
            .cursor_pointer()
            .text_size(TextSize::Xs.font_size())
            .line_height(px(PILL_HEIGHT))
            .font_family(MONO_FONT_FAMILY)
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.foreground.opacity(0.9))
            .when_some(on_open, |d, open| d.press_stop((self.id.clone(), "chip-focus"), crate::theme::radius::md(), window, cx).on_click(move |_, window, cx| open(&pr, window, cx)))
            .child(Icon::new(self.pr.state.icon()).size(px(12.)).color(self.pr.state.color(&theme)))
            .child(self.pr.label());

        let id = self.id.clone();
        let pr = self.pr;
        HoverCard::new(child("hover")).anchor(Anchor::BottomLeft).trigger(pill).content(move |_, _, cx| {
            let theme = cx.theme().clone();
            let muted = theme.muted_foreground;
            let copied = actions.read(cx).copy.copied();
            let button = |name: &'static str, icon: IconName, label: &'static str| {
                Button::new(ElementId::NamedChild(Arc::new(id.clone()), name.into()))
                    .icon(icon)
                    .label(label)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
            };
            let (copy_url, open_url, copy_state) = (pr.url.to_string(), pr.url.to_string(), actions.clone());
            let copy = button("copy", if copied { IconName::Check } else { IconName::Copy }, if copied { "Copied" } else { "Copy link" })
                .on_click(move |_, _, cx| CopyFeedback::click(&copy_state, |s: &mut LinkActions| &mut s.copy, copy_url.clone(), cx));
            let open = button("open", IconName::OpenInNew, "Open in browser").on_click(move |_, _, cx| cx.open_url(&open_url));
            let card = div()
                .my(px(6.))
                .max_w(px(CARD_MAX_WIDTH))
                .flex()
                .flex_col()
                .gap(px(4.))
                .p(px(12.))
                .rounded(radius::xl())
                .bg(theme.card)
                .shadow(popover_shadow(&theme))
                .text_size(TextSize::Xs.font_size())
                .line_height(TextSize::Xs.line_height())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .text_color(muted)
                        .child(Icon::new(pr.state.icon()).size(px(12.)).color(pr.state.color(&theme)))
                        .child(div().flex_1().min_w_0().truncate().child(format!("{} {}", pr.repo, pr.label())))
                        .child(div().flex_none().font_weight(FontWeight::MEDIUM).text_color(pr.state.color(&theme)).child(pr.state.label())),
                )
                .child(
                    div()
                        .text_size(TextSize::Sm.font_size())
                        .line_height(TextSize::Sm.line_height())
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.foreground.opacity(0.9))
                        .child(pr.title.clone()),
                )
                .child(div().flex().gap(px(4.)).mt(px(6.)).ml(px(-8.)).child(copy).child(open));
            let body = if reduce {
                card.into_any_element()
            } else {
                card.with_animation(
                    ElementId::NamedChild(Arc::new(id.clone()), "enter".into()),
                    Animation::new(duration::REVEAL).with_easing(|t| cubic_bezier(ease::OUT, t)),
                    |card, t| card.opacity(t).relative().top(px(4. * (1. - t))),
                )
                .into_any_element()
            };
            div().id(ElementId::NamedChild(Arc::new(id.clone()), "card".into())).child(body)
        })
    }
}

/// Rewrites every `#N` that `resolve` knows into a Markdown link to `atelier-pr:N`, and returns the data
/// for each. Numbers it does not know stay plain text.
pub fn link_prs(markdown: &str, resolve: impl Fn(u64) -> Option<PrChipData>) -> (String, HashMap<u64, PrChipData>) {
    let mut chips = HashMap::new();
    let mut out = String::with_capacity(markdown.len());
    let mut at = 0;
    for (range, number) in pr_refs(markdown) {
        let known = chips.contains_key(&number) || resolve(number).map(|pr| chips.insert(number, pr)).is_some();
        if known {
            out.push_str(&markdown[at..range.start]);
            out.push_str(&format!("[#{number}]({SCHEME}{number})"));
            at = range.end;
        }
    }
    out.push_str(&markdown[at..]);
    (out, chips)
}

/// The number in a link that [`link_prs`] wrote.
pub fn chip_number(url: &str) -> Option<u64> {
    url.strip_prefix(SCHEME)?.parse().ok()
}

/// Draws the links [`link_prs`] wrote as [`PrChip`]s.
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

    fn parse(&self, node: &markdown_ast::Node, _cx: &MarkdownParseContext<'_>) -> Option<MarkdownNode> {
        let markdown_ast::Node::Link(link) = node else { return None };
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
        Some(InlineElement::new(PrChip::new(id, pr).on_open_handler(self.on_open.clone())).with_baseline(px(PILL_BASELINE)))
    }
}

#[cfg(test)]
mod tests;
