//! Merging a pull request, as a pure model: what the repository allows, what holds a merge up and in
//! which order, and what the merge button offers. The app hands the facts over from the forge; nothing
//! here asks anything. The words follow GitQuiet's (`src/ui/Ask.tsx`, `src/ui/Merge.tsx`): a merge that
//! cannot happen says why rather than hiding, and the button says what a press would do.
//!
//! Merging has no keyboard shortcut, on purpose: a merge is hard to undo, and a key would make an
//! accidental one easy. The merge button is still reached and driven from the keyboard (Tab to it,
//! then its menu with Enter, Space or Down), so no action needs the pointer.

use gpui_kit::SharedString;

/// The three ways a forge puts a branch into another.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MergeMethod {
    Merge,
    Squash,
    Rebase,
}

impl MergeMethod {
    pub const ALL: [MergeMethod; 3] = [MergeMethod::Merge, MergeMethod::Squash, MergeMethod::Rebase];

    /// On the button.
    pub fn word(self) -> &'static str {
        match self {
            MergeMethod::Merge => "Merge",
            MergeMethod::Squash => "Squash and merge",
            MergeMethod::Rebase => "Rebase and merge",
        }
    }

    /// In the menu, where each is told apart by what it leaves behind.
    pub fn menu_word(self) -> &'static str {
        match self {
            MergeMethod::Merge => "Create a merge commit",
            MergeMethod::Squash => "Squash and merge",
            MergeMethod::Rebase => "Rebase and merge",
        }
    }
}

/// The two ways a branch is caught up with its base while the pull request stays open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdateWay {
    Merge,
    Rebase,
}

impl UpdateWay {
    pub fn word(self) -> &'static str {
        match self {
            UpdateWay::Merge => "Update with merge commit",
            UpdateWay::Rebase => "Update with rebase",
        }
    }
}

/// What the reader may do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rights {
    Merge,
    /// An administrator, who may merge past the rules that failed.
    Bypass,
    Cannot,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PullState {
    #[default]
    Open,
    Merged,
    Closed,
}

/// Where the required review stands.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ReviewNeed {
    #[default]
    Met,
    Missing,
    /// Changes asked, by these reviewers.
    ChangesAsked(Vec<SharedString>),
}

/// A repository's merge queue, and this pull request's place in it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Queue {
    pub queued: bool,
    /// The first in line is 1.
    pub position: Option<usize>,
}

/// Everything the app knows about merging one pull request.
#[derive(Clone, Debug, PartialEq)]
pub struct MergeFacts {
    pub state: PullState,
    pub draft: bool,
    /// The files that conflict; empty when the branch merges cleanly.
    pub conflicts: Vec<SharedString>,
    /// When the branch is behind its base: the ways it may be caught up, the default first.
    pub behind: Option<Vec<UpdateWay>>,
    /// Required checks that failed, and that are still running.
    pub checks_failing: usize,
    pub checks_running: usize,
    pub review: ReviewNeed,
    /// The repository's merge queue, when it has one.
    pub queue: Option<Queue>,
    /// The ways the repository allows, and its default among them.
    pub methods: Vec<MergeMethod>,
    pub default_method: MergeMethod,
    /// Merge when ready: `None` where the repository does not allow it, else whether it is on.
    pub auto_merge: Option<bool>,
    /// Whether the head branch goes after a merge, as the repository sets it.
    pub delete_branch: bool,
    pub rights: Rights,
}

impl Default for MergeFacts {
    fn default() -> Self {
        Self {
            state: PullState::Open,
            draft: false,
            conflicts: Vec::new(),
            behind: None,
            checks_failing: 0,
            checks_running: 0,
            review: ReviewNeed::Met,
            queue: None,
            methods: MergeMethod::ALL.to_vec(),
            default_method: MergeMethod::Merge,
            auto_merge: None,
            delete_branch: false,
            rights: Rights::Merge,
        }
    }
}

/// One thing that holds a merge up.
#[derive(Clone, Debug, PartialEq)]
pub enum Blocker {
    Draft,
    Conflicts(Vec<SharedString>),
    Behind(Vec<UpdateWay>),
    ChecksFailing(usize),
    ChecksRunning(usize),
    ReviewMissing,
    ChangesAsked(Vec<SharedString>),
    /// Nothing lands straight: it goes through the queue.
    Queue(Queue),
    /// The reader has no write access; someone who does can still merge.
    NoRights,
}

fn checks(n: usize) -> String {
    if n == 1 { "1 required check".into() } else { format!("{n} required checks") }
}

fn names(people: &[SharedString]) -> String {
    match people {
        [] => "a reviewer".into(),
        [one] => one.to_string(),
        [rest @ .., last] => format!("{} and {last}", rest.iter().map(|p| p.as_ref()).collect::<Vec<_>>().join(", ")),
    }
}

impl Blocker {
    /// Its row's title.
    pub fn name(&self) -> String {
        match self {
            Blocker::Draft => "This pull request is still a draft".into(),
            Blocker::Conflicts(_) => "This branch has conflicts that must be resolved".into(),
            Blocker::Behind(_) => "This branch is out of date with the base branch".into(),
            Blocker::ChecksFailing(n) => format!("{} failing", checks(*n)),
            Blocker::ChecksRunning(n) => format!("{} still running", checks(*n)),
            Blocker::ReviewMissing => "Review required".into(),
            Blocker::ChangesAsked(people) => format!("Changes asked by {}", names(people)),
            Blocker::Queue(_) => "This repository merges through a queue".into(),
            Blocker::NoRights => "You cannot merge into this repository".into(),
        }
    }

    /// The sentence under its name.
    pub fn explanation(&self) -> String {
        match self {
            Blocker::Draft => "Mark it ready for review before it can be merged.".into(),
            Blocker::Conflicts(files) => match files.len() {
                1 => "1 file conflicts with the base branch.".into(),
                n => format!("{n} files conflict with the base branch."),
            },
            Blocker::Behind(_) => "Update the branch before it can be merged.".into(),
            Blocker::ChecksFailing(_) => "Fix the failing checks, or run them again.".into(),
            Blocker::ChecksRunning(_) => "It can merge once they pass.".into(),
            Blocker::ReviewMissing => "At least one approving review is required.".into(),
            Blocker::ChangesAsked(_) => "The changes asked for must be made and the review given again.".into(),
            Blocker::Queue(q) if q.queued => match q.position {
                Some(n) => format!("It is number {n} in line, and merges when its turn comes."),
                None => "It is in line, and merges when its turn comes.".into(),
            },
            Blocker::Queue(_) => "Adding it tests it against what is ahead, then merges it.".into(),
            Blocker::NoRights => "Someone with write access can merge it.".into(),
        }
    }

    /// The press that answers it here, where there is one.
    pub fn action(&self) -> Option<Action> {
        match self {
            Blocker::Draft => Some(Action::ReadyForReview),
            Blocker::Behind(ways) => Some(Action::UpdateBranch(ways.first().copied().unwrap_or(UpdateWay::Merge))),
            _ => None,
        }
    }

    /// Whether merge when ready can wait it out: checks and reviews pass by themselves in time.
    pub fn can_wait(&self) -> bool {
        matches!(self, Blocker::ChecksFailing(_) | Blocker::ChecksRunning(_) | Blocker::ReviewMissing | Blocker::ChangesAsked(_))
    }

    /// Whether an administrator may merge past it: the repository's rules, not a conflict or a draft.
    pub fn bypassable(&self) -> bool {
        matches!(
            self,
            Blocker::Behind(_) | Blocker::ChecksFailing(_) | Blocker::ChecksRunning(_) | Blocker::ReviewMissing | Blocker::ChangesAsked(_)
        )
    }

    /// For the standing line: "Blocked: {short}".
    fn short(&self) -> String {
        match self {
            Blocker::Draft => "still a draft".into(),
            Blocker::Conflicts(_) => "conflicts".into(),
            Blocker::Behind(_) => "out of date".into(),
            Blocker::ChecksFailing(n) => format!("{n} {} failing", if *n == 1 { "check" } else { "checks" }),
            Blocker::ChecksRunning(n) => format!("{n} {} still running", if *n == 1 { "check" } else { "checks" }),
            Blocker::ReviewMissing => "review required".into(),
            Blocker::ChangesAsked(people) => format!("changes asked by {}", names(people)),
            Blocker::Queue(_) => "the queue".into(),
            Blocker::NoRights => "no write access".into(),
        }
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

/// A press the merge controls can ask the app for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    ReadyForReview,
    UpdateBranch(UpdateWay),
    Merge(MergeMethod),
    /// Past the rules that failed, as an administrator may.
    BypassAndMerge(MergeMethod),
    MergeWhenReady(MergeMethod),
    CancelMergeWhenReady,
    AddToQueue,
    RemoveFromQueue,
    DeleteBranch,
    Revert,
}

impl Action {
    pub fn word(&self) -> String {
        match self {
            Action::ReadyForReview => "Ready for review".into(),
            Action::UpdateBranch(_) => "Update branch".into(),
            Action::Merge(method) => method.word().into(),
            Action::BypassAndMerge(_) => "Bypass rules and merge".into(),
            Action::MergeWhenReady(_) => "Merge when ready".into(),
            Action::CancelMergeWhenReady => "Cancel merge when ready".into(),
            Action::AddToQueue => "Add to merge queue".into(),
            Action::RemoveFromQueue => "Remove from the queue".into(),
            Action::DeleteBranch => "Delete branch".into(),
            Action::Revert => "Revert".into(),
        }
    }
}

/// What the reader chose in the button's menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choice {
    pub method: MergeMethod,
    /// Merge when ready rather than now, where the repository allows it.
    pub auto: bool,
    pub delete_branch: bool,
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

/// The main part of the merge button.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ButtonState {
    pub label: String,
    /// What a press does; `None` draws it disabled.
    pub action: Option<Action>,
    /// Why it is disabled, for its tooltip.
    pub reason: Option<String>,
    /// Nothing holds it up: the small ready mark beside it shows.
    pub ready: bool,
}

impl ButtonState {
    fn doing(action: Action, ready: bool) -> Self {
        Self { label: action.word(), action: Some(action), reason: None, ready }
    }

    fn refused(label: impl Into<String>, reason: impl Into<String>) -> Self {
        Self { label: label.into(), action: None, reason: Some(reason.into()), ready: false }
    }
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

#[cfg(test)]
mod tests;
