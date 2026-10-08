use gpui_kit::Hsla;
use crate::{IconName, theme::Theme};

/// What kind of change a note tells of. It gives the note's icon and its colour, so a list reads at a glance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReleaseKind {
    /// Something that was not there before.
    New,
    /// Something that was there, and is better.
    Improved,
    /// Something that was wrong, and is not now.
    Fixed,
}

impl ReleaseKind {
    /// The mark in the note's tile: a plus, an arrow up, a wrench.
    pub fn icon(self) -> IconName {
        match self {
            Self::New => IconName::Add,
            Self::Improved => IconName::ArrowUp,
            Self::Fixed => IconName::Build,
        }
    }

    /// The colour of the tile and the mark, from the theme's full-strength tones, which stay apart in every theme: green and purple (the pull request states, in their own colours) and gold (the warning fill).
    pub fn tone(self, theme: &Theme) -> Hsla {
        match self {
            Self::New => theme.pull[0],
            Self::Improved => theme.pull[2],
            Self::Fixed => theme.warning_fill,
        }
    }
}
