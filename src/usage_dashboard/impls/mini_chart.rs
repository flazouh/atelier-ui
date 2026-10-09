use gpui_kit::{AnyElement, Hsla, IntoElement, ParentElement, Styled, div, prelude::FluentBuilder};

use super::super::{
    consts::{MINI_PAST_ALPHA, MINI_RADIUS},
    helpers::mini_bars,
};
use crate::scale::px;

/// Bars of one colour, scaled to the tallest, with rounded top corners only; the last is the full colour. `height` and `gap`
/// are in design pixels: a session's row has a short one, its open view a tall one.
pub(super) fn mini_chart(
    values: &[f32],
    color: Hsla,
    width: f32,
    height: f32,
    gap: f32,
) -> AnyElement {
    let bars = mini_bars(values, height, MINI_RADIUS);
    let last = bars.len().saturating_sub(1);
    div()
        .flex()
        .items_end()
        .gap(px(gap))
        .w_full()
        .max_w(px(width))
        .h(px(height))
        .children(bars.into_iter().enumerate().map(|(at, bar)| {
            let fill = if at == last {
                color
            } else {
                color.opacity(MINI_PAST_ALPHA)
            };
            div()
                .flex_1()
                .h(px(bar.height))
                .bg(fill)
                .when(bar.top_radius > 0., |b| {
                    b.rounded_tl(px(bar.top_radius))
                        .rounded_tr(px(bar.top_radius))
                })
        }))
        .into_any_element()
}
