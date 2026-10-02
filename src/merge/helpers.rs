use gpui_kit::SharedString;

use super::structs::{ButtonState, Choice, MergeFacts, Queue};
use super::types::{Action, Blocker, MergeMethod, PullState, ReviewNeed, Rights};

pub(super) fn checks(n: usize) -> String {
    if n == 1 { "1 required check".into() } else { format!("{n} required checks") }
}

pub(super) fn names(people: &[SharedString]) -> String {
    match people {
        [] => "a reviewer".into(),
        [one] => one.to_string(),
        [rest @ .., last] => format!("{} and {last}", rest.iter().map(|p| p.as_ref()).collect::<Vec<_>>().join(", ")),
    }
}

/// What holds the merge up, in the order it must be dealt with: a draft, conflicts, a branch behind
/// its base, required checks failing then running, the review, a queue in force, and last a reader
/// who cannot merge, which only says so: someone else can.
pub fn blockers(facts: &MergeFacts) -> Vec<Blocker> {
    let mut out = Vec::new();
    if facts.draft {
        out.push(Blocker::Draft);
    }
    if !facts.conflicts.is_empty() {
        out.push(Blocker::Conflicts(facts.conflicts.clone()));
    }
    if let Some(ways) = &facts.behind {
        out.push(Blocker::Behind(ways.clone()));
    }
    if facts.checks_failing > 0 {
        out.push(Blocker::ChecksFailing(facts.checks_failing));
    }
    if facts.checks_running > 0 {
        out.push(Blocker::ChecksRunning(facts.checks_running));
    }
    match &facts.review {
        ReviewNeed::Met => {}
        ReviewNeed::Missing => out.push(Blocker::ReviewMissing),
        ReviewNeed::ChangesAsked(people) => out.push(Blocker::ChangesAsked(people.clone())),
    }
    if let Some(queue) = facts.queue {
        out.push(Blocker::Queue(queue));
    }
    if facts.rights == Rights::Cannot {
        out.push(Blocker::NoRights);
    }
    out
}

/// The methods the menu offers: those the repository allows, its default first.
pub fn offered(facts: &MergeFacts) -> Vec<MergeMethod> {
    let mut out: Vec<MergeMethod> = facts.methods.iter().copied().filter(|m| *m == facts.default_method).collect();
    out.extend(MergeMethod::ALL.into_iter().filter(|m| facts.methods.contains(m) && *m != facts.default_method));
    out
}

/// The method on the button: the one the reader used last in this repository while it is still
/// allowed, else the repository's default, else the first allowed.
pub fn pick_method(facts: &MergeFacts, remembered: Option<MergeMethod>) -> MergeMethod {
    remembered
        .filter(|m| facts.methods.contains(m))
        .or_else(|| offered(facts).first().copied())
        .unwrap_or(facts.default_method)
}

/// The reader's choice to start from: the method as [`pick_method`] finds it, merge when ready off,
/// and the repository's branch setting.
pub fn first_choice(facts: &MergeFacts, remembered: Option<MergeMethod>) -> Choice {
    Choice { method: pick_method(facts, remembered), auto: false, delete_branch: facts.delete_branch }
}

/// What the merge button offers for `facts` and the reader's `choice`; `None` once the pull request
/// is merged or closed, which has no merge button.
pub fn button(facts: &MergeFacts, choice: &Choice) -> Option<ButtonState> {
    if facts.state != PullState::Open {
        return None;
    }
    let method = choice.method;
    if facts.rights == Rights::Cannot {
        return Some(ButtonState::refused(method.word(), "You cannot merge into this repository"));
    }
    if facts.auto_merge == Some(true) {
        return Some(ButtonState::doing(Action::CancelMergeWhenReady, false));
    }
    let all = blockers(facts);
    let queue = facts.queue;
    if queue.is_some_and(|q| q.queued) {
        return Some(ButtonState::doing(Action::RemoveFromQueue, false));
    }
    let holding = all.iter().find(|b| !matches!(b, Blocker::Queue(_)));
    let Some(first) = holding else {
        let action = if queue.is_some() { Action::AddToQueue } else { Action::Merge(method) };
        return Some(ButtonState::doing(action, true));
    };
    if let Some(action) = first.action() {
        return Some(ButtonState::doing(action, false));
    }
    let waitable = all.iter().filter(|b| !matches!(b, Blocker::Queue(_))).all(Blocker::can_wait);
    if waitable && facts.auto_merge.is_some() && (choice.auto || queue.is_some()) {
        return Some(ButtonState::doing(Action::MergeWhenReady(method), false));
    }
    let bypassable = all.iter().filter(|b| !matches!(b, Blocker::Queue(_))).all(Blocker::bypassable);
    if facts.rights == Rights::Bypass && bypassable && queue.is_none() {
        return Some(ButtonState::doing(Action::BypassAndMerge(method), false));
    }
    let label = if queue.is_some() { Action::AddToQueue.word() } else { method.word().to_string() };
    Some(ButtonState::refused(label, first.name()))
}

/// The one line that says where the merge stands.
pub fn standing(facts: &MergeFacts) -> String {
    match facts.state {
        PullState::Merged => return "Merged".into(),
        PullState::Closed => return "Closed".into(),
        PullState::Open => {}
    }
    if facts.auto_merge == Some(true) {
        return "Merges when ready".into();
    }
    if let Some(Queue { queued: true, position }) = facts.queue {
        return match position {
            Some(n) => format!("In the merge queue, number {n}"),
            None => "In the merge queue".into(),
        };
    }
    let all: Vec<Blocker> = blockers(facts).into_iter().filter(|b| !matches!(b, Blocker::Queue(_) | Blocker::NoRights)).collect();
    match all.as_slice() {
        [] if facts.rights == Rights::Cannot => "Ready to merge, by someone with write access".into(),
        [] => "Ready to merge".into(),
        [Blocker::ChecksRunning(n)] => format!("{n} {} still running", if *n == 1 { "check" } else { "checks" }),
        [first, ..] => format!("Blocked: {}", first.short()),
    }
}
