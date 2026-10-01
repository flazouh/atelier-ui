use crate::{
    status_mark::{Mark},
    theme::{StatusTone, Theme},
    };

/// beui's `maxHeight` for the output.
pub(super) const MAX_OUTPUT_HEIGHT: f32 = 220.;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolStatus {
    Running,
    Done,
    Failed,
    Cancelled,
}

impl ToolStatus {
    /// Which tone of the muted ramp the mark uses.
    pub(super) fn tone(self) -> StatusTone {
        match self {
            Self::Running => StatusTone::Running,
            Self::Done => StatusTone::Done,
            Self::Failed => StatusTone::Failed,
            Self::Cancelled => StatusTone::Cancelled,
        }
    }

    /// The solid disc, with the glyph knocked out of it: a check when done, a cross when it failed, a
    /// slash when it never ran, and a plain disc while it runs.
    pub(super) fn mark(self, theme: &Theme) -> Mark {
        let filled = Mark::filled(theme.status_tone(self.tone()), theme.background);
        match self {
            Self::Running => filled,
            Self::Done => filled.check(1.),
            Self::Failed => filled.cross(1.),
            Self::Cancelled => filled.slash(1.),
        }
    }
}
