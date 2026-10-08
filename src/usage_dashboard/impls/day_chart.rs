use gpui_kit::{
    AnyElement, App, FontWeight, Hsla, InteractiveElement, IntoElement, ParentElement, Styled, Window, div,
    prelude::FluentBuilder, StatefulInteractiveElement,
};

use super::{
    super::{
        consts::{AXIS_WIDTH, BAR_GAP, DOT, PAST_ALPHA, PLOT_HEIGHT, TOP_RADIUS},
        helpers::{axis, bar_segments, legend, tokens_label},
        structs::{BarSegment, UsageDashboard, UsageDay},
    },
    key::key,
    panel::panel,
};
use crate::{scale::px, theme::Theme};

/// The stacked chart of the days: a legend, the line for the highlighted day, the grid, the bars and the day numbers.
pub(super) fn day_chart(d: &UsageDashboard, theme: &Theme, window: &mut Window, cx: &mut App) -> AnyElement {
    let last = d.days.len().saturating_sub(1);
    let hovered = window.use_keyed_state(key(&d.id, "hover-day".into()), cx, |_, _| None::<usize>);
    let lit = hovered.read(cx).filter(|at| *at < d.days.len()).unwrap_or(last);
    let scale = axis(d.days.iter().map(UsageDay::total).max().unwrap_or(0));
    let per_token = PLOT_HEIGHT / scale.top as f32;
    let detail = d.days.get(lit).map(|day| day.detail.clone()).unwrap_or_default();

    let legend = div().flex().items_center().gap(px(12.)).text_size(px(12.)).text_color(theme.muted_foreground).children(
        legend(&d.sources).into_iter().map(|entry| {
            div()
                .flex()
                .items_center()
                .gap(px(5.))
                .child(div().flex().gap(px(2.)).children(entry.shades.iter().map(|shade| {
                    div().size(px(DOT)).rounded_full().bg(theme.series(entry.hue, *shade))
                })))
                .child(entry.name)
        }),
    );

    let grid = scale.ticks.iter().enumerate().map(|(at, tick)| {
        let from_bottom = PLOT_HEIGHT * (at + 1) as f32 / scale.ticks.len() as f32;
        div()
            .absolute()
            .left_0()
            .right_0()
            .bottom(px(from_bottom))
            .h(px(1.))
            .bg(theme.foreground.opacity(0.06))
            .child(
                div()
                    .absolute()
                    .left(px(-AXIS_WIDTH))
                    .bottom(px(-7.))
                    .text_size(px(11.))
                    .text_color(theme.muted_foreground)
                    .child(tokens_label(*tick)),
            )
    });

    let columns = d.days.iter().enumerate().map(|(at, day)| {
        let on = at == lit;
        let segments = bar_segments(&day.parts, per_token, TOP_RADIUS);
        let hover = hovered.clone();
        div()
            .id(key(&d.id, format!("day-{at}")))
            .debug_selector(move || format!("usage-day-{at}"))
            .relative()
            .flex_1()
            .min_w_0()
            .h(px(PLOT_HEIGHT))
            .flex()
            .flex_col()
            .justify_end()
            .on_hover(move |entered, _, cx| {
                hover.update(cx, |state, cx| {
                    let next = entered.then_some(at);
                    if *state != next {
                        *state = next;
                        cx.notify();
                    }
                })
            })
            .when(on, |c| {
                c.child(
                    div()
                        .absolute()
                        .left(px(-BAR_GAP / 2.))
                        .right(px(-BAR_GAP / 2.))
                        .top(px(-6.))
                        .bottom_0()
                        .rounded(crate::theme::radius::lg())
                        .bg(theme.foreground.opacity(0.04)),
                )
            })
            .child(stack(&segments, on, at, theme))
    });

    let labels = d.days.iter().enumerate().map(|(at, day)| {
        div()
            .flex_1()
            .min_w_0()
            .flex()
            .justify_center()
            .text_size(px(11.))
            .text_color(if at == lit { theme.foreground } else { theme.muted_foreground })
            .child(day.label.clone())
    });

    panel(theme)
        .w(gpui_kit::relative(0.655))
        .flex_none()
        .child(
            div()
                .flex()
                .items_baseline()
                .justify_between()
                .child(div().text_size(px(14.)).font_weight(FontWeight::MEDIUM).child("Tokens per day"))
                .child(legend),
        )
        .child(div().mt(px(10.)).h(px(18.)).text_size(px(13.)).child(detail))
        .child(
            div()
                .relative()
                .mt(px(10.))
                .ml(px(AXIS_WIDTH))
                .h(px(PLOT_HEIGHT))
                .child(div().absolute().inset_0().children(grid))
                .child(div().absolute().inset_0().flex().gap(px(BAR_GAP)).children(columns)),
        )
        .child(div().flex().gap(px(BAR_GAP)).mt(px(8.)).ml(px(AXIS_WIDTH)).children(labels))
        .into_any_element()
}

/// One bar: its segments from the top down, square and touching, the top one with rounded top corners.
fn stack(segments: &[BarSegment], lit: bool, at: usize, theme: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .children(segments.iter().rev().enumerate().map(|(depth, segment)| {
            let fill: Hsla = theme.series(segment.series.hue, segment.series.shade);
            let fill = if lit { fill } else { fill.opacity(PAST_ALPHA) };
            let selector = format!("usage-seg-{at}-{depth}");
            div()
                .debug_selector(move || selector.clone())
                .w_full()
                .h(px(segment.height))
                .bg(fill)
                .when(segment.top_radius > 0., |s| s.rounded_tl(px(segment.top_radius)).rounded_tr(px(segment.top_radius)))
        }))
        .into_any_element()
}
