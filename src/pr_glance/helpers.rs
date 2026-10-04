use std::{sync::Arc, time::Instant};

use gpui_kit::{
    AnyElement, App, AppContext, Context, Div, ElementId, Entity, FontWeight, HighlightStyle, Hsla, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, StyledText, Styled, Window, black, div, prelude::FluentBuilder, white,
};

use crate::scale::px;
use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    icon::{Icon, IconName},
    pr::{Checks, ChecksSummary, PrChipData, PrFacts, PrReviewer, PrStanding, PrState, ReviewState},
    pr_card::link_actions,
    sidebar_model::since,
    spinner::Spinner,
    theme::{ActiveTheme, Theme, mix, popover_shadow, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};
use super::structs::{PrCardStore, PrCards, PrFailing, PrFile, PrGlance, PrGlanceCard, PrKey, PrSession};
use super::types::{CARD_WIDTH, CONFIRM_FOR, PrAction, PrDoing, PrPart, SIZE_SQUARES, TOP_FILES};

/// The app's one card store, made on first use.
pub fn pr_cards(cx: &mut App) -> Entity<PrCardStore> {
    if let Some(cards) = cx.try_global::<PrCards>() {
        return cards.0.clone();
    }
    let store = cx.new(|_| PrCardStore::default());
    cx.set_global(PrCards(store.clone()));
    store
}

pub fn key_of(pr: &PrChipData) -> PrKey {
    (pr.repo.clone(), pr.number)
}

/// The biggest `TOP_FILES` of `files` by lines changed, the first listed first among equals.
pub fn top_files<T>(mut files: Vec<T>, size: impl Fn(&T) -> u32) -> Vec<T> {
    files.sort_by_key(|f| std::cmp::Reverse(size(f)));
    files.truncate(TOP_FILES);
    files
}

/// How many of the size bar's squares are additions: none for an empty change, and at least one for each side
/// that has any lines.
pub(crate) fn added_squares(added: u32, removed: u32) -> u32 {
    let total = added + removed;
    if total == 0 {
        return 0;
    }
    let share = ((added as f32 / total as f32) * SIZE_SQUARES as f32).round() as u32;
    let least = u32::from(added > 0);
    let most = SIZE_SQUARES - u32::from(removed > 0);
    share.clamp(least, most)
}

/// `path` cut from the left to `most` characters, so the file's own name shows.
pub(super) fn short_path(path: &str, most: usize) -> String {
    let count = path.chars().count();
    if count <= most {
        return path.to_string();
    }
    let tail: String = path.chars().skip(count - (most - 1)).collect();
    let tail = tail.find('/').map_or(tail.as_str(), |at| &tail[at..]);
    format!("…{tail}")
}

/// Two words in their own colours as one text, which lays out and draws as one line.
fn two_tone(first: String, first_color: Hsla, second: String, second_color: Hsla) -> StyledText {
    let split = first.len();
    let text = format!("{first} {second}");
    let tone = |color| HighlightStyle { color: Some(color), ..Default::default() };
    StyledText::new(text.clone()).with_highlights([(0..split, tone(first_color)), (split..text.len(), tone(second_color))])
}

/// `+120 −34` in the diff colours.
pub(crate) fn size_text(added: u32, removed: u32, theme: &Theme) -> Div {
    let text = two_tone(format!("+{added}"), theme.diff_color(true), format!("\u{2212}{removed}"), theme.diff_color(false));
    div().flex_none().font_family(MONO_FONT_FAMILY).child(text)
}

fn size_bar(added: u32, removed: u32, theme: &Theme) -> Div {
    let green = added_squares(added, removed);
    div().flex().flex_none().gap(px(2.)).children((0..SIZE_SQUARES).map(|i| {
        let color = match i {
            _ if added + removed == 0 => theme.divider,
            i if i < green => theme.diff_color(true),
            _ => theme.diff_color(false),
        };
        div().size(px(7.)).rounded(px(1.5)).bg(color)
    }))
}

fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// `2h ago`, or `just now`.
pub(super) fn age(updated_at: u64, now: u64) -> String {
    match since(now, updated_at).as_str() {
        "now" => "just now".to_string(),
        age => format!("{age} ago"),
    }
}

/// `18/20`: the checks that passed out of every check, and how they stand. `None` without checks.
pub(super) fn checks_count(checks: Checks) -> Option<(String, ChecksSummary)> {
    let total = checks.passed + checks.failed + checks.running;
    (total > 0).then(|| (format!("{}/{total}", checks.passed), checks.summary()))
}

fn child_id(id: &ElementId, name: &'static str) -> ElementId {
    ElementId::NamedChild(Arc::new(id.clone()), name.into())
}

fn open_in_app(card: &PrGlanceCard) -> Option<impl Fn(&gpui_kit::ClickEvent, &mut Window, &mut App) + 'static> {
    let (open, pr) = (card.on_open.clone()?, card.pr.clone());
    Some(move |_: &gpui_kit::ClickEvent, window: &mut Window, cx: &mut App| open(&pr, window, cx))
}

/// Text that reads on `fill`: dark on a bright fill, white on a dark one.
pub(super) fn ink_on(fill: Hsla) -> Hsla {
    let c = fill.to_rgb();
    let luminance = 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
    if luminance > 0.4 { mix(fill, black(), 0.72) } else { white() }
}

/// The state filled in its colour and how long since the pull request changed: `Open 2h ago`. A press opens the
/// pull request in the app.
fn state_pill(card: &PrGlanceCard, updated_at: u64, theme: &Theme) -> impl IntoElement {
    let state = card.pr.state;
    let fill = state.color(theme);
    let ink = ink_on(fill);
    div()
        .id(child_id(&card.id, "state"))
        .flex()
        .flex_none()
        .items_center()
        .gap(px(4.))
        .h(px(20.))
        .px(px(7.))
        .rounded(radius::md())
        .bg(fill)
        .text_color(ink)
        .when_some(open_in_app(card), |d, open| d.cursor_pointer().hover(|s| s.bg(mix(fill, white(), 0.12))).on_click(open))
        .child(Icon::new(state.icon()).size(px(12.)).color(ink))
        .child(div().font_weight(FontWeight::SEMIBOLD).child(state.label()))
        .when(updated_at > 0, |d| d.child(div().opacity(0.8).child(age(updated_at, now_secs()))))
}

/// A section of the card: its name and what sums it up on one line, then the rest under it.
fn section(name: &'static str, sum: impl IntoElement, theme: &Theme) -> Div {
    div().flex().flex_col().gap(px(4.)).px(px(10.)).py(px(7.)).rounded(radius::lg()).bg(theme.card_strong).child(
        div()
            .flex()
            .items_center()
            .gap(px(6.))
            .child(div().flex_1().font_weight(FontWeight::MEDIUM).text_color(theme.muted_foreground).child(name))
            .child(sum),
    )
}

/// `✓ 20/20` in the checks' colour, with a spinner while some run.
fn checks_sum(id: &ElementId, text: String, summary: &ChecksSummary, theme: &Theme) -> Div {
    let color = summary.color(theme);
    let mark = match summary {
        ChecksSummary::Running => Spinner::new(child_id(id, "checks")).size(px(11.)).color(color).into_any_element(),
        ChecksSummary::Failing(_) => Icon::new(IconName::Close).size(px(12.)).color(color).into_any_element(),
        _ => Icon::new(IconName::Check).size(px(12.)).color(color).into_any_element(),
    };
    div().flex().flex_none().items_center().gap(px(4.)).text_color(color).font_family(MONO_FONT_FAMILY).font_weight(FontWeight::MEDIUM).child(mark).child(text)
}

/// The failing check, and the first line of its log that says why, or that the log is being read.
fn failing_rows(id: &ElementId, failing: &PrFailing, this: &Entity<PrGlanceCard>, theme: &Theme) -> [Div; 2] {
    let name = div()
        .flex()
        .items_center()
        .gap(px(6.))
        .child(div().flex_1().min_w_0().truncate().font_weight(FontWeight::MEDIUM).text_color(theme.danger).child(failing.name.clone()))
        .when(failing.url.is_some(), |d| {
            d.child(
                div()
                    .id(child_id(id, "log"))
                    .flex_none()
                    .cursor_pointer()
                    .text_color(theme.info)
                    .hover(|s| s.underline())
                    .on_click(act(this, PrAction::OpenLog))
                    .child("Log"),
            )
        });
    let line = match &failing.line {
        Some(line) => div().font_family(MONO_FONT_FAMILY).text_size(px(11.)).truncate().text_color(theme.foreground.opacity(0.8)).child(line.clone()),
        None => div().text_color(theme.muted_foreground).child("Reading the log…"),
    };
    [name, line]
}

/// The review's verdict in its colour, and how many comments.
fn review_sum(facts: &PrFacts, theme: &Theme) -> Div {
    div()
        .flex()
        .flex_none()
        .items_center()
        .gap(px(8.))
        .when(facts.review != ReviewState::None, |d| {
            d.child(div().font_weight(FontWeight::MEDIUM).text_color(facts.review.color(theme)).child(facts.review.text()))
        })
        .when(facts.comments > 0, |d| {
            d.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(3.))
                    .text_color(theme.muted_foreground)
                    .child(Icon::new(IconName::ChatBubble).size(px(11.)))
                    .child(facts.comments.to_string()),
            )
        })
}

fn avatar(reviewer: &PrReviewer, theme: &Theme) -> Div {
    let initial = reviewer.who.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default();
    div()
        .flex()
        .flex_none()
        .items_center()
        .gap(px(5.))
        .child(
            div()
                .size(px(18.))
                .rounded_full()
                .border_1()
                .border_color(reviewer.verdict.color(theme))
                .bg(theme.card)
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(9.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.muted_foreground)
                .child(initial),
        )
        .child(two_tone(reviewer.who.to_string(), theme.foreground.opacity(0.8), reviewer.verdict.text().to_string(), reviewer.verdict.color(theme)))
}

fn reviewers(facts: &PrFacts, theme: &Theme) -> Div {
    div().flex().flex_wrap().items_center().gap_x(px(12.)).gap_y(px(4.)).children(facts.reviewers.iter().map(|r| avatar(r, theme)))
}

fn size_sum(facts: &PrFacts, theme: &Theme) -> Div {
    div().flex().flex_none().items_center().gap(px(8.)).child(size_text(facts.added, facts.removed, theme)).child(size_bar(facts.added, facts.removed, theme))
}

fn files_list(files: &[PrFile], theme: &Theme) -> Div {
    div().flex().flex_col().gap(px(2.)).children(files.iter().map(|f| {
        div()
            .flex()
            .items_center()
            .gap(px(6.))
            .font_family(MONO_FONT_FAMILY)
            .text_size(px(11.))
            .child(div().flex_1().min_w_0().truncate().text_color(theme.foreground.opacity(0.8)).child(short_path(&f.path, 44)))
            .child(size_text(f.added, f.removed, theme))
    }))
}

/// The session the pull request came from; a press opens it.
fn session_section(id: &ElementId, session: &PrSession, this: &Entity<PrGlanceCard>, theme: &Theme) -> impl IntoElement {
    let status = div().flex_none().text_color(if session.running { theme.primary } else { theme.muted_foreground }).child(session.status.clone());
    section("Session", status, theme)
        .id(child_id(id, "session"))
        .cursor_pointer()
        .hover(|d| d.bg(theme.muted_hover()))
        .on_click(act(this, PrAction::OpenSession))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .child(Icon::new(IconName::Bot).size(px(12.)).color(theme.muted_foreground))
                .child(div().flex_1().min_w_0().truncate().font_weight(FontWeight::MEDIUM).child(session.title.clone())),
        )
}

fn act(this: &Entity<PrGlanceCard>, action: PrAction) -> impl Fn(&gpui_kit::ClickEvent, &mut Window, &mut App) + 'static {
    let this = this.downgrade();
    move |_, window, cx| {
        let Some(card) = this.upgrade() else { return };
        let (pr, handler) = {
            let card = card.read(cx);
            (card.pr.clone(), card.store.read(cx).on_action.clone())
        };
        if let Some(handler) = handler {
            handler(action, &pr, window, cx);
        }
    }
}

fn press_merge(card: &mut PrGlanceCard, _: &gpui_kit::ClickEvent, window: &mut Window, cx: &mut Context<PrGlanceCard>) {
    let confirmed = card.confirming.is_some_and(|at| at.elapsed() < CONFIRM_FOR);
    if !confirmed {
        card.confirming = Some(Instant::now());
        card.unconfirm = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(CONFIRM_FOR).await;
            _ = this.update(cx, |card, cx| {
                card.confirming = None;
                cx.notify();
            });
        });
        cx.notify();
        return;
    }
    card.confirming = None;
    let (pr, handler) = (card.pr.clone(), card.store.read(cx).on_action.clone());
    cx.notify();
    if let Some(handler) = handler {
        handler(PrAction::Merge, &pr, window, cx);
    }
}

fn small_button(id: &ElementId, name: &'static str, label: &'static str, variant: ButtonVariant) -> Button {
    Button::new(child_id(id, name)).label(label).variant(variant).size(ButtonSize::Sm)
}

/// Whether the pull request can merge, beside the buttons it decides: `● Ready to merge`.
fn merge_standing(standing: &PrStanding, theme: &Theme) -> Div {
    let color = standing.tone.color(theme);
    div()
        .flex()
        .flex_1()
        .min_w_0()
        .items_center()
        .gap(px(6.))
        .child(div().flex_none().size(px(6.)).rounded_full().bg(color))
        .child(div().flex_none().font_weight(FontWeight::MEDIUM).text_color(color).child(standing.word.clone()))
        .child(div().min_w_0().truncate().text_color(theme.muted_foreground).child(standing.detail.clone()))
}

/// Approve, and Merge for an open pull request (Confirm merge once pressed).
fn write_buttons(card: &PrGlanceCard, this: &Entity<PrGlanceCard>, cx: &mut Context<PrGlanceCard>) -> Div {
    let id = &card.id;
    let confirming = card.confirming.is_some_and(|at| at.elapsed() < CONFIRM_FOR);
    let approve = small_button(id, "approve", "Approve", ButtonVariant::Ghost).on_click(act(this, PrAction::Approve));
    let merge = (card.pr.state == PrState::Open).then(|| {
        let (label, variant) = if confirming { ("Confirm merge", ButtonVariant::Invert) } else { ("Merge", ButtonVariant::Tinted) };
        small_button(id, "merge", label, variant).on_click(cx.listener(press_merge))
    });
    div().flex().flex_none().items_center().gap(px(4.)).child(approve).children(merge)
}

fn doing_line(doing: &PrDoing, theme: &Theme) -> Div {
    let (text, color) = match doing {
        PrDoing::Working(t) => (t, theme.muted_foreground),
        PrDoing::Done(t) => (t, theme.success),
        PrDoing::Failed(t) => (t, theme.danger),
    };
    div().text_color(color).child(text.clone())
}

/// The card: the state pill, where the pull request lives and its links, the whole title, then a section for its
/// checks, its review, its changes and its session, each when it has something to say, and at the foot whether
/// it can merge beside Approve and Merge.
pub(super) fn card(card: &mut PrGlanceCard, window: &mut Window, cx: &mut Context<PrGlanceCard>) -> Div {
    let theme = cx.theme().clone();
    let this = cx.entity();
    let id = card.id.clone();
    let parts = card.store.read(cx).parts();
    let glance: PrGlance = card.store.read(cx).glance(&key_of(&card.pr)).cloned().unwrap_or_default();
    let pr = card.pr.clone();
    let facts = glance.facts.clone().or_else(|| pr.facts.clone());
    let shows = |part: PrPart| parts.shows(part);
    let open = pr.state == PrState::Open || pr.state == PrState::Draft;
    let busy = matches!(glance.doing, Some(PrDoing::Working(_)));
    let can_write = shows(PrPart::Actions) && open && !busy;

    let links: Vec<AnyElement> = link_actions(&id, &pr.url, &card.actions, &theme, window, cx).into();
    let header = div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(state_pill(card, facts.as_ref().map_or(0, |f| f.updated_at), &theme))
        .child(div().flex_1().min_w_0().truncate().text_color(theme.muted_foreground).child(format!("{} {}", pr.repo, pr.label())))
        .child(div().flex().flex_none().mr(px(-6.)).children(links));
    let title = div()
        .id(child_id(&id, "title"))
        .text_size(TextSize::Sm.font_size())
        .line_height(TextSize::Sm.line_height())
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.foreground.opacity(0.92))
        .when_some(open_in_app(card), |d, open| d.cursor_pointer().hover(|s| s.underline()).on_click(open))
        .child(pr.title.clone());

    let failing = glance.failing.as_ref().filter(|_| shows(PrPart::Failing) && open);
    let checks = facts.as_ref().and_then(|f| f.checks).and_then(checks_count).map(|(text, summary)| {
        let ask = (failing.is_some() && can_write).then(|| {
            div().flex().pt(px(2.)).child(small_button(&id, "ask", "Ask the agent to fix", ButtonVariant::Tinted).on_click(act(&this, PrAction::AskToFix)))
        });
        section("Checks", checks_sum(&id, text, &summary, &theme), &theme)
            .children(failing.into_iter().flat_map(|f| failing_rows(&id, f, &this, &theme)))
            .children(ask)
    });
    let review = facts.as_ref().filter(|f| shows(PrPart::Reviewers) && (!f.reviewers.is_empty() || f.review != ReviewState::None || f.comments > 0)).map(|f| {
        section("Review", review_sum(f, &theme), &theme).when(!f.reviewers.is_empty(), |d| d.child(reviewers(f, &theme)))
    });
    let changes = facts.as_ref().map(|f| {
        section("Changes", size_sum(f, &theme), &theme)
            .when_some(glance.files.as_deref().filter(|files| shows(PrPart::Files) && !files.is_empty()), |d, files| d.child(files_list(files, &theme)))
    });
    let session = glance.session.as_ref().filter(|_| shows(PrPart::Sessions)).map(|s| session_section(&id, s, &this, &theme));
    let standing = facts.as_ref().and_then(|f| f.standing.as_ref()).filter(|_| shows(PrPart::Merge) && open).map(|s| merge_standing(s, &theme));
    let writes = can_write.then(|| write_buttons(card, &this, cx));
    let foot = (standing.is_some() || writes.is_some()).then(|| {
        div().flex().items_center().gap(px(8.)).pt(px(2.)).min_h(px(26.)).child(standing.unwrap_or_else(|| div().flex_1())).children(writes)
    });

    div()
        .w(px(CARD_WIDTH))
        .flex()
        .flex_col()
        .gap(px(6.))
        .p(px(10.))
        .rounded(radius::xl())
        .bg(theme.card)
        .shadow(popover_shadow(&theme))
        .text_size(TextSize::Xs.font_size())
        .line_height(TextSize::Xs.line_height())
        .child(header)
        .child(div().px(px(2.)).pb(px(2.)).child(title))
        .children(checks)
        .children(review)
        .children(changes)
        .children(session)
        .children(foot)
        .when_some(glance.doing.as_ref(), |d, doing| d.child(div().px(px(2.)).child(doing_line(doing, &theme))))
}
