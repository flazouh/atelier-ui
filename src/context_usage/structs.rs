use std::rc::Rc;

use gpui_kit::{
    App, ClickEvent, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement,
    Styled, Window, div, prelude::FluentBuilder, relative,
};

use crate::{
    icon::{Icon, IconName},
    scale::px,
    theme::{ActiveTheme, dropdown_edge, popover_shadow},
    typography::{FONT_FAMILY, MONO_FONT_FAMILY},
};
use crate::context_meter::{fraction, ink, level};
use super::helpers::{header, precise, shares, swatch};
use super::types::{BAR, ContextPart, FREE, SEAM, SWATCH, WHOLE, WIDTH};

#[derive(IntoElement)]
pub struct ContextUsage {
    used: u64,
    window: u64,
    parts: Vec<ContextPart>,
    on_close: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
}

impl ContextUsage {
    /// `used` tokens of a window of `window`.
    pub fn new(used: u64, window: u64) -> Self {
        Self { used, window, parts: Vec::new(), on_close: None }
    }

    /// What fills the window, in the order the rows list it. With none, one row tells all that is in use.
    pub fn parts(mut self, parts: Vec<ContextPart>) -> Self {
        self.parts = parts;
        self
    }

    /// Runs when the cross is pressed. Without it the panel has no cross.
    pub fn on_close(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for ContextUsage {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        // An agent that cannot break the window down gets two rows instead: what is in use and what is left.
        let bare = self.parts.is_empty();
        let parts = if bare { vec![ContextPart::new(WHOLE, self.used)] } else { self.parts };
        let free = bare.then(|| ContextPart::new(FREE, self.window.saturating_sub(self.used)));
        let (share, numbers) = header(self.used, self.window);
        // A bar that is one part wears the ring's colour, so it turns amber and red as the ring does.
        let whole = ink(level(fraction(self.used, self.window)), &theme);
        let tint = |i: usize| if bare && i == 0 { whole } else { swatch(&theme, i) };

        let bar = parts.iter().zip(shares(&parts, self.window)).enumerate().fold(
            div()
                .debug_selector(|| "context-usage-bar".into())
                .flex()
                .w_full()
                .h(px(BAR))
                .rounded_full()
                .overflow_hidden()
                .bg(theme.card_strong),
            |bar, (i, (_, share))| {
                bar.child(
                    div().flex_none().h_full().w(relative(share)).child(
                        div().size_full().mr(px(SEAM)).rounded_full().bg(tint(i)).debug_selector(move || format!("context-usage-segment-{i}")),
                    ),
                )
            },
        );

        let free_at = parts.len();
        let rows = parts.into_iter().chain(free).enumerate().map(|(i, part)| {
            // The free row wears the colour of the bar's empty track.
            let colour = if bare && i == free_at { theme.card_strong } else { tint(i) };
            div()
                .debug_selector(move || format!("context-usage-part-{i}"))
                .flex()
                .items_center()
                .gap(px(8.))
                .child(div().flex_none().size(px(SWATCH)).rounded(px(2.)).bg(colour))
                .child(div().flex_1().min_w_0().truncate().text_color(theme.muted_foreground).child(part.label))
                .child(div().flex_none().font_family(MONO_FONT_FAMILY).text_color(theme.foreground).child(precise(part.tokens)))
        });

        let cross = self.on_close.map(|close| {
            div()
                .id("context-usage-close")
                .debug_selector(|| "context-usage-close".into())
                .flex()
                .flex_none()
                .items_center()
                .justify_center()
                .size(px(20.))
                .rounded(px(6.))
                .cursor_pointer()
                .text_color(theme.muted_foreground)
                .hover(|d| d.bg(theme.card_strong).text_color(theme.foreground))
                .on_click(move |event, window, cx| close(event, window, cx))
                .child(Icon::new(IconName::Close).size(px(14.)))
        });

        div()
            .debug_selector(|| "context-usage".into())
            .flex()
            .flex_col()
            .gap(px(12.))
            .w(px(WIDTH))
            .p(px(16.))
            .rounded(px(12.))
            .border_1()
            .border_color(dropdown_edge(&theme))
            .bg(theme.popover)
            .shadow(popover_shadow(&theme))
            .font_family(FONT_FAMILY)
            .text_size(px(13.))
            .line_height(px(18.))
            .text_color(theme.foreground)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(div().font_weight(FontWeight::MEDIUM).child("Context Usage"))
                            .when_some(cross, |d, cross| d.child(cross)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .text_size(px(12.))
                            .text_color(theme.muted_foreground)
                            .child(div().child(share))
                            .child(div().font_family(MONO_FONT_FAMILY).child(numbers)),
                    ),
            )
            .child(bar)
            .child(div().flex().flex_col().gap(px(6.)).children(rows))
    }
}
