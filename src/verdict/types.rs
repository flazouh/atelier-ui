use gpui_kit::SharedString;

/// The three verbs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    Approve,
    RequestChanges,
    Comment,
}

impl Verb {
    pub const ALL: [Verb; 3] = [Self::Approve, Self::RequestChanges, Self::Comment];

    pub fn word(self) -> &'static str {
        match self {
            Self::Approve => "Approve",
            Self::RequestChanges => "Request changes",
            Self::Comment => "Comment",
        }
    }

    /// What it says while it is on its way.
    pub fn working(self) -> &'static str {
        match self {
            Self::Approve => "Approving…",
            Self::RequestChanges => "Sending…",
            Self::Comment => "Posting…",
        }
    }

    pub(super) fn wordless(self) -> bool {
        self == Self::Approve
    }
}

/// A verdict already given.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Approved,
    ChangesRequested,
    Commented,
    Dismissed,
}

/// What the box reports: a verdict to send, about `head_sha`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerdictEvent {
    Send { verb: Verb, note: SharedString, head_sha: SharedString },
}
