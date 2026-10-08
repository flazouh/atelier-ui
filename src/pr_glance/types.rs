use std::{rc::Rc, time::Duration};

use gpui_kit::{App, SharedString, Window};

use crate::pr::PrChipData;

/// One part of a pull request's card that the reader can hide.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrPart {
    Failing,
    Reviewers,
    Merge,
    Sessions,
    Files,
    Actions,
    Live,
}

impl PrPart {
    pub const ALL: [PrPart; 7] = [
        Self::Failing,
        Self::Reviewers,
        Self::Merge,
        Self::Sessions,
        Self::Files,
        Self::Actions,
        Self::Live,
    ];

    /// The part's name in saved settings: it never changes.
    pub fn key(self) -> &'static str {
        match self {
            Self::Failing => "failing-check",
            Self::Reviewers => "reviewers",
            Self::Merge => "merge",
            Self::Sessions => "sessions",
            Self::Files => "files",
            Self::Actions => "actions",
            Self::Live => "live",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.key() == key)
    }

    pub(super) fn bit(self) -> u8 {
        1 << self as u8
    }
}

/// What the reader asked of a card from it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrAction {
    Merge,
    Approve,
    /// Ask the agent of the session to fix the failing check.
    AskToFix,
    OpenSession,
    OpenLog,
}

/// An action's progress, as the card says it under the buttons.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrDoing {
    Working(SharedString),
    Done(SharedString),
    Failed(SharedString),
}

pub type CardOpenHandler = Rc<dyn Fn(&PrChipData, bool, &mut App)>;
pub type CardActionHandler = Rc<dyn Fn(PrAction, &PrChipData, &mut Window, &mut App)>;

/// How long Merge waits for its confirmation before it goes back.
pub(super) const CONFIRM_FOR: Duration = Duration::from_secs(4);

/// The card's width: fixed, so the card does not jump from one pull request to the next.
pub(crate) const CARD_WIDTH: f32 = 340.;

/// The column of the names of the card's lines: Checks, Review, Changes, From.
pub(super) const LABEL_WIDTH: f32 = 52.;

/// The squares of the size bar, as GitHub draws them.
pub(super) const SIZE_SQUARES: u32 = 5;

/// How many changed files the card lists.
pub const TOP_FILES: usize = 3;
