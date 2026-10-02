use std::rc::Rc;

use gpui_kit::{App, Window};

use crate::{icon::IconName, pr::PrChipData};

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

    pub(super) fn urgency(self) -> usize {
        Self::ALL.iter().position(|c| *c == self).unwrap_or(usize::MAX)
    }
}

pub(super) type OpenHandler = Rc<dyn Fn(&PrChipData, &mut Window, &mut App)>;

/// The working set, filed into Courts, one row per pull request: its author, its state mark, `#N`, the
/// The least room the title keeps: the other columns give way before it does, so a narrow pane still reads.
pub(super) const TITLE_LEAST: f32 = 200.;

impl Court {
    pub(super) fn icon(self) -> IconName {
        match self {
            Self::NeedsYou => IconName::PriorityHigh,
            Self::Waiting => IconName::Schedule,
            Self::Running => IconName::Progress,
            Self::Settled => IconName::Check,
        }
    }
}
