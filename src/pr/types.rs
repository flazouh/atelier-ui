use gpui_kit::{Hsla, SharedString};

use crate::{icon::IconName, theme::Theme};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrState {
    Open,
    Draft,
    Merged,
    Closed,
}

impl PrState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::Draft => "Draft",
            Self::Merged => "Merged",
            Self::Closed => "Closed",
        }
    }

    pub fn icon(self) -> IconName {
        match self {
            Self::Open => IconName::PrOpen,
            Self::Draft => IconName::PrDraft,
            Self::Merged => IconName::PrMerged,
            Self::Closed => IconName::PrClosed,
        }
    }

    /// GitHub's colour for the state, from [`Theme::pull`].
    pub fn color(self, theme: &Theme) -> Hsla {
        theme.pull[self as usize]
    }
}

/// What the checks summary says, from worst to best: one failure outweighs any number of passes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChecksSummary {
    None,
    Failing(u32),
    Running,
    Passed(u32),
}

impl ChecksSummary {
    pub fn text(&self) -> SharedString {
        match self {
            Self::None => SharedString::default(),
            Self::Failing(n) => format!("{n} failing").into(),
            Self::Running => "Checks running".into(),
            Self::Passed(1) => "1 check passed".into(),
            Self::Passed(n) => format!("{n} checks passed").into(),
        }
    }

    pub fn color(&self, theme: &Theme) -> Hsla {
        match self {
            Self::Failing(_) => theme.danger,
            Self::Passed(_) => theme.success,
            Self::None | Self::Running => theme.muted_foreground,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReviewState {
    #[default]
    None,
    Requested,
    Approved,
    ChangesRequested,
}

impl ReviewState {
    pub fn text(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Requested => "Review asked",
            Self::Approved => "Approved",
            Self::ChangesRequested => "Changes asked",
        }
    }

    pub fn color(self, theme: &Theme) -> Hsla {
        match self {
            Self::Approved => theme.success,
            Self::ChangesRequested => theme.warning,
            Self::None | Self::Requested => theme.muted_foreground,
        }
    }
}

/// Where one reviewer stands: a verdict, or still asked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrVerdict {
    Approved,
    ChangesAsked,
    Commented,
    Waiting,
}

impl PrVerdict {
    pub fn text(self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::ChangesAsked => "changes",
            Self::Commented => "commented",
            Self::Waiting => "waiting",
        }
    }

    pub fn color(self, theme: &Theme) -> Hsla {
        match self {
            Self::Approved => theme.success,
            Self::ChangesAsked => theme.warning,
            Self::Commented | Self::Waiting => theme.muted_foreground,
        }
    }
}

/// How a merge standing reads: it can go, something holds it, it waits its turn, or it is over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StandingTone {
    Ready,
    Held,
    Waiting,
    Merged,
    Closed,
}

impl StandingTone {
    pub fn color(self, theme: &Theme) -> Hsla {
        match self {
            Self::Ready => theme.success,
            Self::Held => theme.warning,
            Self::Waiting => theme.muted_foreground,
            Self::Merged => PrState::Merged.color(theme),
            Self::Closed => PrState::Closed.color(theme),
        }
    }
}
