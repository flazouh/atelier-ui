use std::{sync::Arc, time::Instant};

use gpui_kit::{
    AnyElement, App, AppContext, Context, Div, ElementId, Entity, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    icon::{Icon, IconName},
    pr::{ChecksSummary, PrChipData, PrFacts, PrReviewer, PrStanding, PrState, ReviewState},
    pr_card::link_actions,
    sidebar_model::since,
    spinner::Spinner,
    theme::{ActiveTheme, Theme, popover_shadow, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};
use super::structs::{PrCardStore, PrCards, PrFailing, PrFile, PrGlance, PrGlanceCard, PrKey, PrParts, PrSession};
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

/// `+120 −34` in the diff colours.
pub(crate) fn size_text(added: u32, removed: u32, theme: &Theme) -> Div {
    div()
        .flex()
        .flex_none()
        .gap(px(4.))
        .font_family(MONO_FONT_FAMILY)
        .child(div().text_color(theme.diff_color(true)).child(format!("+{added}")))
        .child(div().text_color(theme.diff_color(false)).child(format!("\u{2212}{removed}")))
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

fn age(facts: &PrFacts, state: PrState, now: u64) -> String {
    let since = match since(now, facts.updated_at).as_str() {
        "now" => "just now".to_string(),
        age => format!("{age} ago"),
    };
    match state {
        PrState::Merged => format!("merged {since}"),
        PrState::Closed => format!("closed {since}"),
        PrState::Open | PrState::Draft => format!("updated {since}"),
    }
}

/// The author, how long since it changed, and how much was said: `alex · updated 3h ago · 4`.
fn byline(facts: &PrFacts, state: PrState, theme: &Theme) -> Div {
    let dot = || div().flex_none().child("·");
    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .text_color(theme.muted_foreground)
        .child(div().min_w_0().truncate().font_weight(FontWeight::MEDIUM).child(facts.author.clone()))
        .when(facts.updated_at > 0, |d| d.child(dot()).child(div().flex_none().child(age(facts, state, now_secs()))))
        .when(facts.comments > 0, |d| {
            d.child(dot()).child(
                div().flex().flex_none().items_center().gap(px(3.)).child(Icon::new(IconName::ChatBubble).size(px(11.))).child(facts.comments.to_string()),
            )
        })
}

/// The checks and the review on the left, the size and its bar on the right.
fn standing_row(id: &ElementId, facts: &PrFacts, live: bool, theme: &Theme) -> Div {
    let summary = facts.checks.map(|c| c.summary()).unwrap_or(ChecksSummary::None);
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
                        d.child(Spinner::new(child_id(id, "checks")).size(px(11.)).color(theme.muted_foreground))
                    })
                    .child(summary.text()),
            )
        })
        .when(facts.review != ReviewState::None, |d| {
            d.child(div().flex_none().font_weight(FontWeight::MEDIUM).text_color(facts.review.color(theme)).child(facts.review.text()))
        })
        .when(live, |d| d.child(div().flex_none().text_color(theme.muted_foreground).child("· live")))
        .child(div().flex_1())
        .child(size_text(facts.added, facts.removed, theme))
        .child(size_bar(facts.added, facts.removed, theme))
}

fn branches(facts: &PrFacts, theme: &Theme) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .text_color(theme.muted_foreground)
        .child(Icon::new(IconName::GitBranch).size(px(11.)))
        .child(div().min_w_0().truncate().font_family(MONO_FONT_FAMILY).text_color(theme.foreground.opacity(0.8)).child(facts.head.clone()))
        .child(div().flex_none().child("→"))
        .child(div().flex_none().font_family(MONO_FONT_FAMILY).text_color(theme.foreground.opacity(0.8)).child(facts.base.clone()))
        .when(facts.conflicting, |d| d.child(div().flex_none().font_weight(FontWeight::MEDIUM).text_color(theme.warning).child("conflicts")))
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
                .bg(theme.card_strong)
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(9.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.muted_foreground)
                .child(initial),
        )
        .child(div().text_color(theme.foreground.opacity(0.8)).child(reviewer.who.clone()))
        .child(div().text_color(reviewer.verdict.color(theme)).child(reviewer.verdict.text()))
}

/// A box a little darker than the card, for the failing check and the linked session.
fn inset(theme: &Theme) -> Div {
    div().flex().flex_col().gap(px(2.)).px(px(8.)).py(px(6.)).rounded(radius::md()).bg(theme.card_strong)
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

/// The menu the three dots open, in place of the card's body: each part, ticked while it shows.
fn parts_menu(card: &PrGlanceCard, parts: PrParts, cx: &mut Context<PrGlanceCard>) -> Div {
    let theme = cx.theme().clone();
    let row = |id: ElementId| {
        div().id(id).flex().items_center().gap(px(8.)).h(px(26.)).px(px(6.)).mx(px(-6.)).rounded(radius::md()).cursor_pointer().hover(|s| s.bg(theme.muted_hover()))
    };
    let store = card.store.clone();
    div()
        .flex()
        .flex_col()
        .child(div().pb(px(4.)).text_color(theme.muted_foreground).child("Show in this card"))
        .children(PrPart::ALL.into_iter().map(|part| {
            let store = store.clone();
            let on = parts.shows(part);
            row(child_id(&card.id, part.key()))
                .on_click(move |_, _, cx| {
                    store.update(cx, |s, cx| {
                        let mut parts = s.parts();
                        parts.set(part, !on);
                        s.set_parts(parts, cx);
                    })
                })
                .child(div().size(px(14.)).flex().items_center().justify_center().when(on, |d| d.child(Icon::new(IconName::Check).size(px(13.)).color(theme.foreground))))
                .child(div().text_color(theme.foreground.opacity(0.9)).child(part.label()))
        }))
        .child(div().h(px(1.)).my(px(4.)).bg(theme.divider))
        .child(
            row(child_id(&card.id, "settings"))
                .on_click(cx.listener(|card, _, window, cx| {
                    card.menu = false;
                    let (pr, handler) = (card.pr.clone(), card.store.read(cx).on_action.clone());
                    if let Some(handler) = handler {
                        handler(PrAction::Settings, &pr, window, cx);
                    }
                }))
                .child(div().size(px(14.)).flex().items_center().justify_center().child(Icon::new(IconName::Settings).size(px(13.)).color(theme.muted_foreground)))
                .child(div().text_color(theme.foreground.opacity(0.9)).child("Card settings…")),
        )
}

fn child_id(id: &ElementId, name: &'static str) -> ElementId {
    ElementId::NamedChild(Arc::new(id.clone()), name.into())
}

/// The three dots: they open the parts menu in place of the body.
fn more_button(card: &PrGlanceCard, theme: &Theme, cx: &mut Context<PrGlanceCard>) -> impl IntoElement {
    div()
        .id(child_id(&card.id, "more"))
        .flex()
        .flex_none()
        .size(px(20.))
        .items_center()
        .justify_center()
        .rounded(radius::md())
        .cursor_pointer()
        .text_color(theme.muted_foreground)
        .when(card.menu, |d| d.bg(theme.muted_hover()).text_color(theme.foreground))
        .hover(|s| s.bg(theme.muted_hover()).text_color(theme.foreground))
        .on_click(cx.listener(|card, _, _, cx| {
            card.menu = !card.menu;
            cx.notify();
        }))
        .child(Icon::new(IconName::MoreHoriz).size(px(14.)))
}

/// The failing check, and the first line of its log that says why, or that the log is being read.
fn failing_box(id: &ElementId, failing: &PrFailing, this: &Entity<PrGlanceCard>, theme: &Theme) -> Div {
    let name = div()
        .flex()
        .items_center()
        .gap(px(6.))
        .text_color(theme.danger)
        .font_weight(FontWeight::MEDIUM)
        .child(Icon::new(IconName::Close).size(px(12.)))
        .child(div().flex_1().min_w_0().truncate().child(failing.name.clone()))
        .when(failing.url.is_some(), |d| {
            d.child(
                div()
                    .id(child_id(id, "log"))
                    .flex_none()
                    .cursor_pointer()
                    .font_weight(FontWeight::NORMAL)
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
    inset(theme).child(name).child(line)
}

fn reviewers(facts: &PrFacts, theme: &Theme) -> Div {
    div().flex().flex_wrap().items_center().gap_x(px(12.)).gap_y(px(4.)).children(facts.reviewers.iter().map(|r| avatar(r, theme)))
}

fn merge_standing(standing: &PrStanding, theme: &Theme) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .child(div().flex_none().font_weight(FontWeight::MEDIUM).text_color(standing.tone.color(theme)).child(standing.word.clone()))
        .child(div().min_w_0().truncate().text_color(theme.muted_foreground).child(standing.detail.clone()))
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
fn session_box(id: &ElementId, session: &PrSession, this: &Entity<PrGlanceCard>, theme: &Theme) -> impl IntoElement {
    div()
        .id(child_id(id, "session"))
        .flex()
        .items_center()
        .gap(px(6.))
        .px(px(8.))
        .py(px(5.))
        .rounded(radius::md())
        .bg(theme.card_strong)
        .cursor_pointer()
        .hover(|d| d.bg(theme.muted_hover()))
        .on_click(act(this, PrAction::OpenSession))
        .child(Icon::new(IconName::Bot).size(px(12.)).color(theme.muted_foreground))
        .child(div().flex_none().text_color(theme.muted_foreground).child("From"))
        .child(div().flex_1().min_w_0().truncate().font_weight(FontWeight::MEDIUM).child(session.title.clone()))
        .child(div().flex_none().text_color(if session.running { theme.primary } else { theme.muted_foreground }).child(session.status.clone()))
}

/// Ask the agent to fix while a check fails, Merge for an open pull request (Confirm merge once pressed), and
/// Approve.
fn write_buttons(card: &PrGlanceCard, failing: bool, this: &Entity<PrGlanceCard>, cx: &mut Context<PrGlanceCard>) -> Div {
    let id = &card.id;
    let confirming = card.confirming.is_some_and(|at| at.elapsed() < CONFIRM_FOR);
    let ask = failing.then(|| small_button(id, "ask", "Ask the agent to fix", ButtonVariant::Tinted).on_click(act(this, PrAction::AskToFix)));
    let merge = (card.pr.state == PrState::Open).then(|| {
        let (label, variant) = if confirming { ("Confirm merge", ButtonVariant::Invert) } else { ("Merge", ButtonVariant::Tinted) };
        small_button(id, "merge", label, variant).on_click(cx.listener(press_merge))
    });
    let approve = small_button(id, "approve", "Approve", ButtonVariant::Ghost).on_click(act(this, PrAction::Approve));
    div().flex().items_center().gap(px(4.)).children(ask).children(merge).child(approve)
}

fn doing_line(doing: &PrDoing, theme: &Theme) -> Div {
    let (text, color) = match doing {
        PrDoing::Working(t) => (t, theme.muted_foreground),
        PrDoing::Done(t) => (t, theme.success),
        PrDoing::Failed(t) => (t, theme.danger),
    };
    div().text_color(color).child(text.clone())
}

/// The card: where it lives and its state, the whole title, then each part the reader shows that has
/// something to say, and the actions.
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

    let header = div()
        .flex()
        .items_center()
        .gap(px(6.))
        .text_color(theme.muted_foreground)
        .child(Icon::new(pr.state.icon()).size(px(12.)).color(pr.state.color(&theme)))
        .child(div().flex_1().min_w_0().truncate().child(format!("{} {}", pr.repo, pr.label())))
        .child(div().flex_none().font_weight(FontWeight::MEDIUM).text_color(pr.state.color(&theme)).child(pr.state.label()))
        .child(more_button(card, &theme, cx));
    let body = div()
        .w(px(CARD_WIDTH))
        .flex()
        .flex_col()
        .gap(px(6.))
        .p(px(12.))
        .pb(px(8.))
        .rounded(radius::xl())
        .bg(theme.card)
        .shadow(popover_shadow(&theme))
        .text_size(TextSize::Xs.font_size())
        .line_height(TextSize::Xs.line_height())
        .child(header);
    if card.menu {
        return body.child(parts_menu(card, parts, cx));
    }

    let failing = glance.failing.as_ref().filter(|_| shows(PrPart::Failing) && open);
    let busy = matches!(glance.doing, Some(PrDoing::Working(_)));
    let writes = (shows(PrPart::Actions) && open && !busy).then(|| write_buttons(card, failing.is_some(), &this, cx));
    let open_here = card.on_open.clone().map(|open| {
        let pr = pr.clone();
        small_button(&id, "open-here", "Open", ButtonVariant::Ghost).on_click(move |_, window, cx| open(&pr, window, cx))
    });
    let links: Vec<AnyElement> = link_actions(&id, &pr.url, &card.actions, &theme, window, cx).into();
    let title = div()
        .text_size(TextSize::Sm.font_size())
        .line_height(TextSize::Sm.line_height())
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.foreground.opacity(0.92))
        .child(pr.title.clone());

    body.child(title)
        .when_some(facts.as_ref().filter(|f| shows(PrPart::Branches) && !f.head.is_empty()), |d, f| d.child(branches(f, &theme)))
        .when_some(facts.as_ref(), |d, f| d.child(byline(f, pr.state, &theme)).child(standing_row(&id, f, shows(PrPart::Live) && open, &theme)))
        .when_some(failing, |d, f| d.child(failing_box(&id, f, &this, &theme)))
        .when_some(facts.as_ref().filter(|f| shows(PrPart::Reviewers) && !f.reviewers.is_empty()), |d, f| d.child(reviewers(f, &theme)))
        .when_some(facts.as_ref().and_then(|f| f.standing.as_ref()).filter(|_| shows(PrPart::Merge)), |d, s| d.child(merge_standing(s, &theme)))
        .when_some(glance.files.as_deref().filter(|f| shows(PrPart::Files) && !f.is_empty()), |d, f| d.child(files_list(f, &theme)))
        .when_some(glance.session.as_ref().filter(|_| shows(PrPart::Sessions)), |d, s| d.child(session_box(&id, s, &this, &theme)))
        .child(div().h(px(1.)).mt(px(2.)).bg(theme.divider))
        .child(div().flex().items_center().gap(px(4.)).ml(px(-8.)).children(open_here).children(writes).child(div().flex_1()).children(links))
        .when_some(glance.doing.as_ref(), |d, doing| d.child(doing_line(doing, &theme)))
}
