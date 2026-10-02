/// What a section's news means, which colours its summary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SectionTone {
    #[default]
    Plain,
    Bad,
    Done,
    /// Owed a move by the reader.
    Attention,
}
