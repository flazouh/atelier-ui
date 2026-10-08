use crate::IconName;
/// What kind of change a note tells of. It gives the note's icon and its label, so a list reads at a glance. Its colour is the
/// owner's to give (see [`ReleaseSheet::kind_colors`](super::ReleaseSheet::kind_colors)): this crate names no colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReleaseKind {
    /// Something that was not there before.
    Added,
    /// Something that was there, and is better.
    Improved,
    /// Something that was slow, and is quick now.
    Faster,
    /// Something that was wrong, and is not now.
    Fixed,
    /// Something that works another way now.
    Changed,
    /// Something that looks another way now.
    Design,
}
impl ReleaseKind {
    /// Every kind, in the order a list of kinds reads.
    pub const ALL: [ReleaseKind; 6] = [Self::Added, Self::Improved, Self::Faster, Self::Fixed, Self::Changed, Self::Design];
    /// The mark at the note's left: a plus, an arrow up, a flash, a bug, two turning arrows, a brush.
    pub fn icon(self) -> IconName {
        match self {
            Self::Added => IconName::Add,
            Self::Improved => IconName::ArrowUp,
            Self::Faster => IconName::Flash,
            Self::Fixed => IconName::Bug,
            Self::Changed => IconName::Exchange,
            Self::Design => IconName::Brush,
        }
    }
    /// The kind in words, for the small label over the note.
    pub fn label(self) -> &'static str {
        match self {
            Self::Added => "Added",
            Self::Improved => "Improved",
            Self::Faster => "Faster",
            Self::Fixed => "Fixed",
            Self::Changed => "Changed",
            Self::Design => "Design",
        }
    }
}
