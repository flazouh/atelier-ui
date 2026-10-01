//! Pull requests sorted by who owes the next move, after GitQuiet's working set: its four Courts, Needs
//! You, Waiting, Running and Settled, with its words (`CONTEXT.md`, `domain/sittings.ts`).
//!
//! atelier-ui does not decide which Court a pull request sits in; the app does, from its own reading of the
//! pull request, and hands it over as data. atelier-ui files them: Courts in reading order, an empty Court left
//! out, a pull request listed twice kept once in its most urgent Court, and inside each Court the newest
//! change first.

use std::{collections::HashSet, rc::Rc, sync::Arc};

use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::{
    focus::PressStop,
    icon::{Icon, IconName},
    pr::{Checks, ChecksSummary, PrChipData, ReviewState},
    spinner::Spinner,
    theme::{ActiveTheme, Theme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};

/// Who owes the next move.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Court {
    NeedsYou,
    Waiting,
    Running,
    Settled,
}

impl Court {
    /// Reading order, most urgent first.
    pub const ALL: [Court; 4] = [Self::NeedsYou, Self::Waiting, Self::Running, Self::Settled];

    pub fn name(self) -> &'static str {
        match self {
            Self::NeedsYou => "Needs You",
            Self::Waiting => "Waiting",
            Self::Running => "Running",
            Self::Settled => "Settled",
        }
    }

    pub fn means(self) -> &'static str {
        match self {
            Self::NeedsYou => "You can act on it now.",
            Self::Waiting => "Someone else has to act.",
            Self::Running => "A machine is still working. Nothing to do but wait.",
            Self::Settled => "Finished. Nothing left to do.",
        }
    }

    fn urgency(self) -> usize {
        Self::ALL.iter().position(|c| *c == self).unwrap_or(usize::MAX)
    }
}

/// One pull request in the working set.
#[derive(Clone, Debug, PartialEq)]
pub struct CourtItem {
    pub pr: PrChipData,
    pub author: SharedString,
    pub court: Court,
    /// Why it sits in its Court, in a few words: "Review asked of you", "Ready to merge".
    pub why: SharedString,
    pub checks: Checks,
    pub review: ReviewState,
    pub comments: usize,
    pub added: usize,
    pub removed: usize,
    /// How long since it changed, as the app words it: "2h ago".
    pub age: SharedString,
    /// When it changed, for ordering: larger is newer.
    pub changed_at: u64,
    /// The reader has not read it since it changed.
    pub unread: bool,
}

/// The working set filed into Courts: reading order, empty Courts left out, each pull request once in
/// its most urgent Court, the newest change first.
pub fn courts(items: Vec<CourtItem>) -> Vec<(Court, Vec<CourtItem>)> {
    let mut once: Vec<CourtItem> = Vec::new();
    for item in items {
        match once.iter_mut().find(|o| o.pr.repo == item.pr.repo && o.pr.number == item.pr.number) {
            Some(already) if item.court.urgency() < already.court.urgency() => *already = item,
            Some(_) => {}
            None => once.push(item),
        }
    }
    Court::ALL
        .into_iter()
        .filter_map(|court| {
            let mut rows: Vec<CourtItem> = once.iter().filter(|i| i.court == court).cloned().collect();
            rows.sort_by_key(|r| std::cmp::Reverse(r.changed_at));
            (!rows.is_empty()).then_some((court, rows))
        })
        .collect()
}

/// A row's checks as GitQuiet counts them, "11 of 18": passed of all, while some run or fail. All passed
/// shows only the mark, and no checks show nothing.
pub fn checks_text(checks: Checks) -> Option<SharedString> {
    let total = checks.passed + checks.failed + checks.running;
    (total > 0 && checks.passed < total).then(|| format!("{} of {total}", checks.passed).into())
}

type OpenHandler = Rc<dyn Fn(&PrChipData, &mut Window, &mut App)>;

/// The working set, filed into Courts, one row per pull request: its author, its state mark, `#N`, the
/// The least room the title keeps: the other columns give way before it does, so a narrow pane still reads.
const TITLE_LEAST: f32 = 200.;

/// title (in weight while unread), its repository, why it sits there, its review and checks, how much was
/// said, its size, and its age. Each Court heads its rows with its name and count, and folds.
#[derive(IntoElement)]
pub struct CourtList {
    id: ElementId,
    items: Vec<CourtItem>,
    on_open: Option<OpenHandler>,
}

impl CourtList {
    pub fn new(id: impl Into<ElementId>, items: Vec<CourtItem>) -> Self {
        Self { id: id.into(), items, on_open: None }
    }

    pub fn on_open(mut self, f: impl Fn(&PrChipData, &mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(f));
        self
    }
}

impl Court {
    fn icon(self) -> IconName {
        match self {
            Self::NeedsYou => IconName::PriorityHigh,
            Self::Waiting => IconName::Schedule,
            Self::Running => IconName::Progress,
            Self::Settled => IconName::Check,
        }
    }
}

fn initial(name: &SharedString, theme: &Theme) -> impl IntoElement {
    let initial: String = name.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default();
    div()
        .flex()
        .flex_none()
        .size(px(18.))
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(theme.card_strong)
        .text_size(px(10.))
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.muted_foreground)
        .child(initial)
}

impl RenderOnce for CourtList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let folded = window.use_keyed_state(self.id.clone(), cx, |_, _| HashSet::<u8>::new());
        let folded_now = folded.read(cx).clone();
        let child = |name: String| ElementId::NamedChild(Arc::new(self.id.clone()), name.into());
        let on_open = self.on_open.clone();

        let row = |item: CourtItem, window: &mut Window, cx: &mut App| {
            let open = on_open.clone();
            let pr = item.pr.clone();
            let checks = item.checks.summary();
            let checks_el = match checks {
                ChecksSummary::None => div().into_any_element(),
                ChecksSummary::Running => div()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .text_color(theme.warning)
                    .child(Spinner::new(child(format!("spin-{}-{}", item.pr.repo, item.pr.number))).size(px(12.)).color(theme.warning))
                    .children(checks_text(item.checks))
                    .into_any_element(),
                ChecksSummary::Failing(_) => div()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .text_color(theme.danger)
                    .child(Icon::new(IconName::Error).size(px(12.)))
                    .children(checks_text(item.checks))
                    .into_any_element(),
                ChecksSummary::Passed(_) => Icon::new(IconName::CheckCircle).size(px(12.)).color(theme.success).into_any_element(),
            };
            div()
                .id(child(format!("row-{}-{}", item.pr.repo, item.pr.number)))
                .flex()
                .items_center()
                .gap(px(10.))
                .h(px(30.))
                .px(px(12.))
                .rounded(radius::md())
                .cursor_pointer()
                .text_size(TextSize::Xs.font_size())
                .hover(|s| s.bg(theme.muted_hover()))
                .when_some(open, |d, open| {
                    d.press_stop(gpui_kit::ElementId::Name(format!("row-focus-{}-{}", item.pr.repo, item.pr.number).into()), crate::theme::radius::md(), window, cx)
                        .on_click(move |_, window, cx| open(&pr, window, cx))
                })
                .child(initial(&item.author, &theme))
                .child(Icon::new(item.pr.state.icon()).size(px(14.)).color(item.pr.state.color(&theme)))
                .child(div().flex_none().w(px(58.)).font_family(MONO_FONT_FAMILY).text_size(px(11.)).text_color(muted).child(item.pr.label()))
                .child(
                    div()
                        .flex_1()
                        .min_w(px(TITLE_LEAST))
                        .debug_selector(|| "court-title".into())
                        .truncate()
                        .text_color(theme.foreground.opacity(0.9))
                        .when(item.unread, |d| d.font_weight(FontWeight::SEMIBOLD))
                        .child(item.pr.title.clone()),
                )
                .child(div().min_w_0().w(px(120.)).truncate().text_color(muted).child(item.pr.repo.clone()))
                .child(div().min_w_0().w(px(132.)).truncate().text_color(muted).child(item.why.clone()))
                .child(div().min_w_0().w(px(104.)).truncate().text_color(item.review.color(&theme)).child(item.review.text()))
                .child(div().min_w_0().w(px(72.)).child(checks_el))
                .child(
                    div()
                        .flex()
                        .min_w_0()
                        .w(px(40.))
                        .items_center()
                        .gap(px(4.))
                        .text_color(muted)
                        .when(item.comments > 0, |d| d.child(Icon::new(IconName::ChatBubble).size(px(12.))).child(item.comments.to_string())),
                )
                .child(
                    div()
                        .flex()
                        .min_w_0()
                        .w(px(88.))
                        .justify_end()
                        .gap(px(4.))
                        .font_family(MONO_FONT_FAMILY)
                        .text_size(px(11.))
                        .child(div().text_color(theme.diff_color(true)).child(format!("+{}", item.added)))
                        .child(div().text_color(theme.diff_color(false)).child(format!("\u{2212}{}", item.removed))),
                )
                .child(div().flex_none().w(px(56.)).text_color(muted).text_right().child(item.age.clone()))
        };

        div().flex().flex_col().gap(px(8.)).children(courts(self.items).into_iter().map(|(court, rows)| {
            let key = court as u8;
            let is_folded = folded_now.contains(&key);
            let toggle = folded.clone();
            let color = if court == Court::NeedsYou { theme.warning } else { theme.foreground.opacity(0.9) };
            let count = rows.len();
            div()
                .flex()
                .flex_col()
                .p(px(4.))
                .rounded(radius::lg())
                .bg(theme.card)
                .child(
                    div()
                        .id(child(format!("court-{key}")))
                        .press_stop(child(format!("court-focus-{key}")), crate::theme::radius::md(), window, cx)
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .h(px(30.))
                        .px(px(8.))
                        .rounded(radius::md())
                        .cursor_pointer()
                        .text_size(TextSize::Xs.font_size())
                        .hover(|s| s.bg(theme.muted_hover()))
                        .on_click(move |_, _, cx| toggle.update(cx, |set, cx| {
                            if !set.remove(&key) {
                                set.insert(key);
                            }
                            cx.notify();
                        }))
                        .child(Icon::new(if is_folded { IconName::ChevronRight } else { court.icon() }).size(px(14.)).color(color))
                        .child(div().font_weight(FontWeight::SEMIBOLD).text_color(color).child(court.name()))
                        .child(div().text_color(muted).child(count.to_string())),
                )
                .when(!is_folded, |d| d.children(rows.into_iter().map(|item| row(item, window, cx))))
        }))
    }
}

#[cfg(test)]
mod tests;
