//! beui's MessageBubble (`components/agents/message-bubble.tsx`), class for class:
//!
//! - Surface: borderless per the recipe. `solid` fills `foreground` (text flips to `background`); `soft`
//!   and `tint` fill `card`; `borderless` (beui's `outline`) fills `card_strong` in place of a border;
//!   `danger` fills `danger` at 10% (text `danger`); `ghost` draws no surface and drops the padding,
//!   stretching to the full row width.
//! - Shape: `rounded-2xl`, `px-3.5 py-2.5`, `text-sm leading-6`, capped at 82% width, at least 36px wide.
//! - Alignment: `start` or `end`; the row stacks the bubble on that edge of its column.
//! - Grouped corners: [`message_bubble_group`] stacks consecutive bubbles `gap-1.5` (compact) or
//!   `gap-3` (default) apart, so a speaker's turns read as one column instead of separate cards.
//! - Expandable content: [`MessageBubbleCollapsible`] clips long prose to a line count behind a bottom
//!   fade, with a "Show more/less" pill whose chevron turns on `SPRING_SWAP`.
//! - Entrance: the shared chat [`Entrance`]: the whole bubble fades and rises 6px, like every other
//!   chat item. Under Reduce Motion it only fades.

use std::sync::Arc;

use gpui_kit::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, relative,
};
use crate::scale::px;

use crate::{
    focus::PressStop,
    entrance::Entrance,
    icon::{Icon, IconName},
    motion::{Channel, Curve, Spring},
    theme::{ActiveTheme, Theme, radius},
    typography::TextSize,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MessageBubbleVariant {
    Solid,
    #[default]
    Soft,
    Tint,
    /// beui's `outline`, borderless: a `card_strong` fill stands in for the border.
    Borderless,
    Danger,
    Ghost,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MessageBubbleAlign {
    #[default]
    Start,
    End,
}

fn surface_fill(variant: MessageBubbleVariant, theme: &Theme) -> Option<gpui_kit::Hsla> {
    match variant {
        MessageBubbleVariant::Solid => Some(theme.foreground),
        MessageBubbleVariant::Soft | MessageBubbleVariant::Tint => Some(theme.card),
        MessageBubbleVariant::Borderless => Some(theme.card_strong),
        MessageBubbleVariant::Danger => Some(theme.danger.opacity(0.1)),
        MessageBubbleVariant::Ghost => None,
    }
}

fn content_color(variant: MessageBubbleVariant, theme: &Theme) -> gpui_kit::Hsla {
    match variant {
        MessageBubbleVariant::Solid => theme.background,
        MessageBubbleVariant::Danger => theme.danger,
        _ => theme.foreground,
    }
}

#[derive(IntoElement)]
pub struct MessageBubble {
    id: ElementId,
    variant: MessageBubbleVariant,
    align: MessageBubbleAlign,
    animate_in: bool,
    content: AnyElement,
}

impl MessageBubble {
    pub fn new(id: impl Into<ElementId>, content: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            variant: MessageBubbleVariant::default(),
            align: MessageBubbleAlign::default(),
            animate_in: false,
            content: content.into_any_element(),
        }
    }

    /// Plain text, for a bubble with no nested markup.
    pub fn text(id: impl Into<ElementId>, body: impl Into<gpui_kit::SharedString>) -> Self {
        Self::new(id, div().child(body.into()))
    }

    pub fn variant(mut self, variant: MessageBubbleVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn align(mut self, align: MessageBubbleAlign) -> Self {
        self.align = align;
        self
    }

    /// Plays the [`Entrance`] once, for a bubble the user just sent. Streaming updates to an
    /// already-mounted bubble never replay it, because the entrance state is kept by `id`.
    pub fn animate_in(mut self, animate_in: bool) -> Self {
        self.animate_in = animate_in;
        self
    }
}

impl RenderOnce for MessageBubble {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let variant = self.variant;
        let ghost = variant == MessageBubbleVariant::Ghost;
        let fill = surface_fill(variant, &theme);
        let text_color = content_color(variant, &theme);

        let card = div()
            .relative()
            .when(!ghost, |d| {
                d.min_w(px(36.)).max_w(relative(0.82)).px(px(14.)).py(px(10.)).rounded(radius::xxl())
            })
            .when(ghost, |d| d.w_full())
            .text_size(TextSize::Sm.font_size())
            .line_height(px(24.))
            .text_color(text_color)
            .when_some(fill, |d, color| d.bg(color))
            .child(self.content);

        let row = div()
            .flex()
            .w_full()
            .flex_col()
            .when(self.align == MessageBubbleAlign::End, |d| d.items_end())
            .when(self.align == MessageBubbleAlign::Start, |d| d.items_start())
            .child(card);
        Entrance::new(ElementId::NamedChild(Arc::new(self.id), "enter".into()), row).skip_initial(!self.animate_in)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MessageBubbleGroupSpacing {
    #[default]
    Compact,
    Default,
}

/// Stacks a speaker's bubbles into one grouped column.
pub fn message_bubble_group(spacing: MessageBubbleGroupSpacing) -> gpui_kit::Div {
    let gap = match spacing {
        MessageBubbleGroupSpacing::Compact => px(6.),
        MessageBubbleGroupSpacing::Default => px(12.),
    };
    div().flex().flex_col().w_full().gap(gap)
}

struct CollapsibleMotion {
    open: bool,
    chevron: Channel,
}

#[derive(IntoElement)]
pub struct MessageBubbleCollapsible {
    id: ElementId,
    collapsed_lines: u8,
    default_open: bool,
    /// The surface color to fade into at the bottom edge when collapsed. Defaults to `card`, which
    /// matches a soft or tint bubble; pass the bubble's own fill for a solid, danger, or ghost one.
    fade_into: Option<gpui_kit::Hsla>,
    content: AnyElement,
}

impl MessageBubbleCollapsible {
    pub fn new(id: impl Into<ElementId>, content: impl IntoElement) -> Self {
        Self { id: id.into(), collapsed_lines: 4, default_open: false, fade_into: None, content: content.into_any_element() }
    }

    /// How many lines show before the fade, from beui's `2 | 3 | 4 | 5 | 6`.
    pub fn collapsed_lines(mut self, lines: u8) -> Self {
        self.collapsed_lines = lines.clamp(2, 6);
        self
    }

    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    pub fn fade_into(mut self, color: impl Into<gpui_kit::Hsla>) -> Self {
        self.fade_into = Some(color.into());
        self
    }
}

impl RenderOnce for MessageBubbleCollapsible {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let default_open = self.default_open;
        let motion = window.use_keyed_state(self.id.clone(), cx, move |_, _| CollapsibleMotion {
            open: default_open,
            chevron: Channel::new(if default_open { 180. } else { 0. }),
        });
        let m = motion.read(cx);
        if m.chevron.is_running() {
            window.request_animation_frame();
        }
        let (open, chevron) = (m.open, m.chevron.value());
        let theme = cx.theme().clone();
        let fade_into = self.fade_into.unwrap_or(theme.card);
        let total_h = self.collapsed_lines as f32 * 24.;
        let fade_h = total_h * 0.32;

        let clipped = div()
            .relative()
            .w_full()
            .when(!open, |d| d.max_h(px(total_h)).overflow_hidden())
            .child(self.content)
            .when(!open, |d| {
                d.child(
                    div()
                        .absolute()
                        .bottom_0()
                        .left_0()
                        .right_0()
                        .h(px(fade_h))
                        .bg(gpui_kit::linear_gradient(
                            180.,
                            gpui_kit::linear_color_stop(fade_into.opacity(0.), 0.),
                            gpui_kit::linear_color_stop(fade_into, 1.),
                        )),
                )
            });

        let toggle = motion.clone();
        let pill = div()
            .id(ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), "toggle".into()))
            .mt(px(8.))
            .flex()
            .h(px(28.))
            .items_center()
            .gap(px(4.))
            .rounded_full()
            .px(px(8.))
            .text_size(TextSize::Xs.font_size())
            .font_weight(gpui_kit::FontWeight::MEDIUM)
            .text_color(theme.muted_foreground)
            .cursor_pointer()
            .hover(|s| s.bg(theme.card).text_color(theme.foreground))
            .press_stop((self.id.clone(), "more-focus"), crate::theme::radius::md(), window, cx)
            .on_click(move |_, _, cx| {
                let reduce = cx.reduce_motion();
                toggle.update(cx, |m, cx| {
                    m.open = !m.open;
                    m.chevron.animate(if m.open { 180. } else { 0. }, Curve::Spring(Spring::SWAP), 0., reduce);
                    cx.notify();
                })
            })
            .child(if open { "Show less" } else { "Show more" })
            .child(Icon::new(IconName::ChevronDown).size(px(14.)).turn(chevron / 360.));

        div().flex().flex_col().w_full().child(clipped).child(pill)
    }
}
