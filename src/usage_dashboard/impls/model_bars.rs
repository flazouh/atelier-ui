use gpui_kit::{AnyElement, FontWeight, InteractiveElement, IntoElement, ParentElement, Styled, div, relative};

use super::{super::structs::UsageDashboard, panel::panel};
use crate::{scale::px, theme::Theme, typography::MONO_FONT_FAMILY};

/// The models: a name, its tokens, and a bar to the share of the biggest; the total at the foot.
pub(super) fn model_bars(d: &UsageDashboard, theme: &Theme) -> AnyElement {
    let most = d.models.iter().map(|m| m.tokens).max().unwrap_or(0).max(1) as f32;
    let rows = d.models.iter().enumerate().map(|(at, model)| {
        let share = model.tokens as f32 / most;
        let selector = format!("usage-model-{at}");
        div()
            .debug_selector(move || selector.clone())
            .mb(px(15.))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .mb(px(6.))
                    .text_size(px(13.))
                    .child(div().min_w_0().truncate().child(model.name.clone()))
                    .child(
                        div()
                            .font_family(MONO_FONT_FAMILY)
                            .text_size(px(12.))
                            .text_color(theme.muted_foreground)
                            .child(model.label.clone()),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .h(px(4.))
                    .rounded_full()
                    .bg(theme.foreground.opacity(0.1))
                    .child(
                        div()
                            .h_full()
                            .w(relative(share.clamp(0., 1.)))
                            .rounded_full()
                            .bg(theme.series(model.series.hue, model.series.shade)),
                    ),
            )
    });
    panel(theme)
        .flex_1()
        .child(
            div()
                .flex()
                .items_baseline()
                .justify_between()
                .mb(px(14.))
                .child(div().text_size(px(14.)).font_weight(FontWeight::MEDIUM).child("By model"))
                .child(div().text_size(px(12.)).text_color(theme.muted_foreground).child(d.range.label())),
        )
        .children(rows)
        .child(div().flex_1())
        .children(d.total.clone().map(|total| {
            div()
                .flex()
                .items_baseline()
                .justify_between()
                .text_size(px(13.))
                .child(div().text_color(theme.muted_foreground).child("Total · est. cost"))
                .child(div().font_family(MONO_FONT_FAMILY).child(total))
        }))
        .into_any_element()
}
