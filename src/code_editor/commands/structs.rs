use std::ops::Range;

/// One replacement, and where every selection goes afterwards, in the new text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub range: Range<usize>,
    pub text: String,
    pub selections: Vec<Range<usize>>,
}
