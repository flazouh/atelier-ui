/// What a press on a clipped body does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Press {
    /// Whether the body is opened to its taller view after the press.
    pub expanded: bool,
    /// Whether the owner hears of it, to take the reader to the whole thing.
    pub open: bool,
}

impl Press {
    /// The press on a body that is `expanded` now, and `clipped` when it has more rows than it shows.
    ///
    /// A body that fits has nothing to open, but the owner still hears of the press. A clipped one only opens or folds
    /// in place: the owner does not hear of it.
    pub(crate) fn on(expanded: bool, clipped: bool) -> Self {
        match (clipped, expanded) {
            (false, _) => Self {
                expanded: false,
                open: true,
            },
            (true, false) => Self {
                expanded: true,
                open: false,
            },
            (true, true) => Self {
                expanded: false,
                open: false,
            },
        }
    }
}
