use std::{collections::HashMap, sync::Arc, time::Instant};

use gpui_kit::{
    AnyElement, App, Div, ElementId, Entity, FontWeight, InteractiveElement, ParentElement, Stateful,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    focus::PressStop,
    icon::Icon,
    icon::IconName,
    pr::{ChecksSummary, PrChipData, PrFacts, ReviewState},
    pr_card::{LinkActions, link_actions},
    pr_refs::pr_refs,
    sidebar_model::since,
    spinner::Spinner,
    theme::{Theme, mix, popover_shadow, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};
use super::structs::Warmth;
use super::types::{CARD_WIDTH, OPEN_DELAY, PILL_HEIGHT, PILL_TITLE_WIDTH, PrOpenHandler, SCHEME, SIZE_SQUARES, WARM_FOR};

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

/// How many of the size bar's squares are additions: none for an empty change, and at least one for each side
/// that has any lines.
pub(super) fn added_squares(added: u32, removed: u32) -> u32 {
    let total = added + removed;
    if total == 0 {
        return 0;
    }
    let share = ((added as f32 / total as f32) * SIZE_SQUARES as f32).round() as u32;
    let least = u32::from(added > 0);
    let most = SIZE_SQUARES - u32::from(removed > 0);
    share.clamp(least, most)
}

/// Whether a card opens at once: one is open, or one closed a moment ago.
pub(super) fn warm(warmth: Option<&Warmth>, now: Instant) -> bool {
    warmth.is_some_and(|w| w.open || w.changed.is_some_and(|at| now.duration_since(at) < WARM_FOR))
}

pub(super) fn open_delay(cx: &App) -> std::time::Duration {
    if warm(cx.try_global::<Warmth>(), Instant::now()) { std::time::Duration::ZERO } else { OPEN_DELAY }
}

/// `+120 −34` in the diff colours.
fn size_text(added: u32, removed: u32, theme: &Theme) -> Div {
    div()
        .flex()
        .flex_none()
        .gap(px(4.))
        .font_family(MONO_FONT_FAMILY)
        .child(div().text_color(theme.diff_color(true)).child(format!("+{added}")))
        .child(div().text_color(theme.diff_color(false)).child(format!("\u{2212}{removed}")))
}

/// The pill: the state's mark, `#3344`, the title cut to fit, and the size once known.
pub(super) fn pill(id: &ElementId, pr: &PrChipData, on_open: Option<PrOpenHandler>, theme: &Theme, window: &mut Window, cx: &mut App) -> Stateful<Div> {
    let child = |name: &'static str| ElementId::NamedChild(Arc::new(id.clone()), name.into());
    let open_pr = pr.clone();
    div()
        .id(child("pill"))
        .flex()
        .items_center()
        .gap(px(5.))
        .h(px(PILL_HEIGHT))
        .px(px(6.))
        .rounded(radius::md())
        .bg(theme.card_strong)
        .hover(|s| s.bg(mix(theme.card_strong, theme.foreground, 0.06)))
        .cursor_pointer()
        .text_size(TextSize::Xs.font_size())
        .line_height(px(PILL_HEIGHT))
        .when_some(on_open, |d, open| {
            d.press_stop((id.clone(), "chip-focus"), radius::md(), window, cx).on_click(move |_, window, cx| open(&open_pr, window, cx))
        })
        .child(Icon::new(pr.state.icon()).size(px(12.)).color(pr.state.color(theme)))
        .child(
            div()
                .flex_none()
                .font_family(MONO_FONT_FAMILY)
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.foreground.opacity(0.9))
                .child(pr.label()),
        )
        .child(div().min_w_0().max_w(px(PILL_TITLE_WIDTH)).truncate().text_color(theme.foreground.opacity(0.75)).child(pr.short_title().to_string()))
        .when_some(pr.facts.as_ref(), |d, facts| d.child(size_text(facts.added, facts.removed, theme)))
}

/// The author, how long since it changed, and how much was said: `alex · updated 3h ago · 4`.
fn byline(facts: &PrFacts, now: u64, theme: &Theme) -> Div {
    let dot = || div().child("·");
    let age = match since(now, facts.updated_at).as_str() {
        "now" => "updated just now".to_string(),
        age => format!("updated {age} ago"),
    };
    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .text_color(theme.muted_foreground)
        .child(div().min_w_0().truncate().font_weight(FontWeight::MEDIUM).child(facts.author.clone()))
        .when(facts.updated_at > 0, |d| d.child(dot()).child(div().flex_none().child(age)))
        .when(facts.comments > 0, |d| {
            d.child(dot()).child(
                div().flex().flex_none().items_center().gap(px(3.)).child(Icon::new(IconName::ChatBubble).size(px(11.))).child(facts.comments.to_string()),
            )
        })
}

/// The checks and the review on the left, the size and its bar on the right.
fn standing(id: &ElementId, facts: &PrFacts, theme: &Theme) -> Div {
    let summary = facts.checks.map(|c| c.summary()).unwrap_or(ChecksSummary::None);
    let green = added_squares(facts.added, facts.removed);
    let squares = (0..SIZE_SQUARES).map(|i| {
        let color = match i {
            _ if facts.added + facts.removed == 0 => theme.divider,
            i if i < green => theme.diff_color(true),
            _ => theme.diff_color(false),
        };
        div().size(px(7.)).rounded(px(1.5)).bg(color)
    });
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .when(summary != ChecksSummary::None, |d| {
            d.child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(4.))
                    .text_color(summary.color(theme))
                    .when(summary == ChecksSummary::Running, |d| {
                        d.child(Spinner::new(ElementId::NamedChild(Arc::new(id.clone()), "checks".into())).size(px(11.)).color(theme.muted_foreground))
                    })
                    .child(summary.text()),
            )
        })
        .when(facts.review != ReviewState::None, |d| {
            d.child(div().flex_none().font_weight(FontWeight::MEDIUM).text_color(facts.review.color(theme)).child(facts.review.text()))
        })
        .child(div().flex_1())
        .child(size_text(facts.added, facts.removed, theme))
        .child(div().flex().flex_none().gap(px(2.)).children(squares))
}

/// The card: where it lives and its state, the whole title, then what the forge said of it, and the actions.
pub(super) fn card(
    id: &ElementId,
    pr: &PrChipData,
    on_open: Option<PrOpenHandler>,
    actions: &Entity<LinkActions>,
    theme: &Theme,
    window: &mut Window,
    cx: &mut App,
) -> Div {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let links: Vec<AnyElement> = link_actions(id, &pr.url, actions, theme, window, cx).into();
    let open = on_open.map(|open| {
        let pr = pr.clone();
        Button::new(ElementId::NamedChild(Arc::new(id.clone()), "open-here".into()))
            .label("Open")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .on_click(move |_, window, cx| open(&pr, window, cx))
    });
    div()
        .my(px(6.))
        .w(px(CARD_WIDTH))
        .flex()
        .flex_col()
        .gap(px(6.))
        .p(px(12.))
        .pb(px(8.))
        .rounded(radius::xl())
        .bg(theme.card)
        .shadow(popover_shadow(theme))
        .text_size(TextSize::Xs.font_size())
        .line_height(TextSize::Xs.line_height())
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .text_color(theme.muted_foreground)
                .child(Icon::new(pr.state.icon()).size(px(12.)).color(pr.state.color(theme)))
                .child(div().flex_1().min_w_0().truncate().child(format!("{} {}", pr.repo, pr.label())))
                .child(div().flex_none().font_weight(FontWeight::MEDIUM).text_color(pr.state.color(theme)).child(pr.state.label())),
        )
        .child(
            div()
                .text_size(TextSize::Sm.font_size())
                .line_height(TextSize::Sm.line_height())
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.foreground.opacity(0.92))
                .child(pr.title.clone()),
        )
        .when_some(pr.facts.as_ref(), |d, facts| d.child(byline(facts, now, theme)).child(standing(id, facts, theme)))
        .child(div().h(px(1.)).mt(px(2.)).bg(theme.divider))
        .child(div().flex().items_center().ml(px(-8.)).children(open).child(div().flex_1()).children(links))
}
