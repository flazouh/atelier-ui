use gpui_kit::SharedString;

use super::structs::Queue;
use super::helpers::{checks, names};

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
    pub(super) fn short(&self) -> String {
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
