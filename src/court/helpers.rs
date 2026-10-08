use gpui_kit::{FontWeight, IntoElement, ParentElement, SharedString, Styled, div};

use super::structs::CourtItem;
use super::types::Court;
use crate::scale::px;
use crate::{pr::Checks, theme::Theme};

/// The working set filed into Courts: reading order, empty Courts left out, each pull request once in
/// its most urgent Court, the newest change first.
pub fn courts(items: Vec<CourtItem>) -> Vec<(Court, Vec<CourtItem>)> {
    let mut once: Vec<CourtItem> = Vec::new();
    for item in items {
        match once
            .iter_mut()
            .find(|o| o.pr.repo == item.pr.repo && o.pr.number == item.pr.number)
        {
            Some(already) if item.court.urgency() < already.court.urgency() => *already = item,
            Some(_) => {}
            None => once.push(item),
        }
    }
    Court::ALL
        .into_iter()
        .filter_map(|court| {
            let mut rows: Vec<CourtItem> =
                once.iter().filter(|i| i.court == court).cloned().collect();
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

pub(super) fn initial(name: &SharedString, theme: &Theme) -> impl IntoElement {
    let initial: String = name
        .chars()
        .next()
        .map(|c| c.to_uppercase().collect())
        .unwrap_or_default();
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
