use std::sync::Arc;

use gpui_kit::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, relative,
};

use crate::scale::px;
use crate::{
    entrance::Entrance,
    focus::PressStop,
    icon::{Icon, IconName},
    motion::{Channel, Curve, Spring},
    theme::{ActiveTheme, radius},
    typography::TextSize,
};
use super::types::{LINE, MessageBubbleAlign, MessageBubbleVariant, PAD_X, PAD_Y};
use super::helpers::{content_color, surface_fill};

#[derive(IntoElement)]
pub struct MessageBubble {
    id: ElementId,
    pub(super) variant: MessageBubbleVariant,
    align: MessageBubbleAlign,
    animate_in: bool,
    pub(super) content: AnyElement,
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
                d.min_w(px(36.)).max_w(relative(0.82)).px(px(PAD_X)).py(px(PAD_Y)).rounded(radius::xl())
            })
            .when(ghost, |d| d.w_full())
            .text_size(TextSize::Sm.font_size())
            .line_height(px(LINE))
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

struct CollapsibleMotion {
    open: bool,
    pub(super) chevron: Channel,
}

#[derive(IntoElement)]
pub struct MessageBubbleCollapsible {
    id: ElementId,
    collapsed_lines: u8,
    default_open: bool,
    /// The surface color to fade into at the bottom edge when collapsed. Defaults to `card`, which
    /// matches a soft or tint bubble; pass the bubble's own fill for a solid, danger, or ghost one.
    fade_into: Option<gpui_kit::Hsla>,
    pub(super) content: AnyElement,
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
        let total_h = self.collapsed_lines as f32 * LINE;
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
            .mt(px(4.))
            .flex()
            .h(px(24.))
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
