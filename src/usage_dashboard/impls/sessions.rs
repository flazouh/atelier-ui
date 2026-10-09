use gpui_kit::{
    AnyElement, App, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};

use super::{
    super::{
        consts::{
            CHEVRON_WIDTH, COST_WIDTH, MINI_GAP, MINI_HEIGHT, MINI_WIDTH, ROW_BARS_GAP,
            ROW_BARS_HEIGHT, SPARK_WIDTH, SPLIT_WIDTH, TOKENS_WIDTH,
        },
        structs::{UsageDashboard, UsageSession},
    },
    key::key,
    mini_chart::mini_chart,
    panel::panel,
};
use crate::{
    focus::PressStop,
    icon::{Icon, IconName},
    scale::px,
    theme::{Theme, radius},
    typography::MONO_FONT_FAMILY,
};

/// The sessions: a head, and a row for each; the open one shows its days and where its tokens went.
pub(super) fn sessions(
    d: &UsageDashboard,
    theme: &Theme,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let muted = theme.muted_foreground;
    let head = div()
        .flex()
        .items_center()
        .px(px(12.))
        .pb(px(6.))
        .text_size(px(12.))
        .text_color(muted)
        .child(div().flex_1().pl(px(0.)).child("Session"))
        .child(div().w(px(SPARK_WIDTH)).flex_none().child(d.range.label()))
        .child(
            div()
                .w(px(TOKENS_WIDTH))
                .flex_none()
                .flex()
                .justify_end()
                .child("Tokens"),
        )
        .child(
            div()
                .w(px(COST_WIDTH))
                .flex_none()
                .flex()
                .justify_end()
                .child("Cost"),
        )
        .child(div().w(px(CHEVRON_WIDTH)).flex_none());
    let mut list = div().flex().flex_col();
    for session in &d.sessions {
        let open = d.expanded.as_ref() == Some(&session.id);
        list = list.child(row(d, session, open, theme, window, cx));
        if open {
            list = list.child(detail(session, theme));
        }
    }
    panel(theme)
        .pb(px(12.))
        .child(
            div()
                .flex()
                .items_baseline()
                .justify_between()
                .mb(px(10.))
                .child(
                    div()
                        .text_size(px(14.))
                        .font_weight(FontWeight::MEDIUM)
                        .child("Sessions"),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(muted)
                        .child("press a row for its days"),
                ),
        )
        .child(head)
        .child(list)
        .into_any_element()
}

fn row(
    d: &UsageDashboard,
    session: &UsageSession,
    open: bool,
    theme: &Theme,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let handler = d.on_expand.clone();
    let next = if open { None } else { Some(session.id.clone()) };
    let selector = format!("usage-session-{}", session.id);
    let color = theme.series(session.series.hue, session.series.shade);
    div()
        .id(key(&d.id, format!("session-{}", session.id)))
        .debug_selector(move || selector.clone())
        .flex()
        .items_center()
        .px(px(12.))
        .py(px(11.))
        .rounded(radius::xl())
        .when(open, |r| r.bg(theme.card_strong).rounded_b(px(0.)))
        .cursor_pointer()
        .press_stop(
            key(&d.id, format!("session-press-{}", session.id)),
            radius::xl(),
            window,
            cx,
        )
        .on_click(move |_, window, cx| {
            if let Some(handler) = &handler {
                handler(next.clone(), window, cx);
            }
        })
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .text_size(px(13.))
                        .font_weight(FontWeight::MEDIUM)
                        .truncate()
                        .child(session.title.clone()),
                )
                .child(
                    div()
                        .mt(px(2.))
                        .text_size(px(12.))
                        .text_color(theme.muted_foreground)
                        .truncate()
                        .child(session.meta.clone()),
                ),
        )
        .child(
            div()
                .flex_none()
                .w(px(SPARK_WIDTH))
                .pr(px(12.))
                .child(mini_chart(
                    &session.days,
                    color,
                    SPARK_WIDTH,
                    ROW_BARS_HEIGHT,
                    ROW_BARS_GAP,
                )),
        )
        .child(mono(session.tokens.clone(), TOKENS_WIDTH))
        .child(mono(session.cost.clone(), COST_WIDTH))
        .child(
            div()
                .w(px(CHEVRON_WIDTH))
                .flex_none()
                .flex()
                .justify_end()
                .text_color(theme.faint())
                .child(
                    Icon::new(if open {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .size(px(14.)),
                ),
        )
        .into_any_element()
}

/// A number that changes, in the mono face, at the right of its column.
fn mono(text: gpui_kit::SharedString, width: f32) -> impl IntoElement {
    div()
        .w(px(width))
        .flex_none()
        .flex()
        .justify_end()
        .font_family(MONO_FONT_FAMILY)
        .text_size(px(13.))
        .child(text)
}

/// The open session: its days as bars, and where its tokens went.
fn detail(session: &UsageSession, theme: &Theme) -> AnyElement {
    let muted = theme.muted_foreground;
    let color = theme.series(session.series.hue, session.series.shade);
    let split = session.split.iter().map(|(label, value)| {
        div()
            .flex()
            .justify_between()
            .mb(px(6.))
            .child(div().child(label.clone()))
            .child(div().font_family(MONO_FONT_FAMILY).child(value.clone()))
    });
    div()
        .flex()
        .gap(px(24.))
        .px(px(12.))
        .pt(px(6.))
        .pb(px(16.))
        .mb(px(4.))
        .rounded_b(radius::xl())
        .bg(theme.card_strong)
        .debug_selector(|| "usage-session-detail".into())
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .mb(px(6.))
                        .text_size(px(12.))
                        .text_color(muted)
                        .child("Tokens per day · this session"),
                )
                .child(mini_chart(
                    &session.days,
                    color,
                    MINI_WIDTH,
                    MINI_HEIGHT,
                    MINI_GAP,
                )),
        )
        .child(
            div()
                .flex_none()
                .w(px(SPLIT_WIDTH))
                .text_size(px(13.))
                .child(
                    div()
                        .mb(px(8.))
                        .text_size(px(12.))
                        .text_color(muted)
                        .child("Where the tokens went"),
                )
                .children(split)
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .text_color(muted)
                        .child(div().child(session.footnote.clone()))
                        .child(
                            div()
                                .font_family(MONO_FONT_FAMILY)
                                .child(session.cost.clone()),
                        ),
                ),
        )
        .into_any_element()
}
