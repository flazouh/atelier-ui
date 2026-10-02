use gpui_kit::SharedString;

use super::types::{Action, MergeMethod, PullState, ReviewNeed, Rights, UpdateWay};

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

/// What the reader chose in the button's menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choice {
    pub method: MergeMethod,
    /// Merge when ready rather than now, where the repository allows it.
    pub auto: bool,
    pub delete_branch: bool,
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
    pub(super) fn doing(action: Action, ready: bool) -> Self {
        Self { label: action.word(), action: Some(action), reason: None, ready }
    }

    pub(super) fn refused(label: impl Into<String>, reason: impl Into<String>) -> Self {
        Self { label: label.into(), action: None, reason: Some(reason.into()), ready: false }
    }
}
