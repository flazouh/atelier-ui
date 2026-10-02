use gpui_kit::SharedString;

/// What a session waits for from the reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Need {
    /// A tool waits for a yes or a no.
    Approval,
    /// The agent asked a question.
    Question,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionStatus {
    /// The agent works: its mark animates.
    Working,
    /// The reader owes the next move.
    NeedsYou(Need),
    /// A turn ended and the reader has not looked yet: its changes are ready to look at. Opening the
    /// session clears it, and the app then sets [`SessionStatus::Idle`].
    Finished,
    /// Seen, and nothing going on.
    Idle,
    /// The session stopped. The reason is short.
    Failed(SharedString),
}

/// How much of a failure's reason a row shows.
pub(super) const REASON_CHARS: usize = 40;

/// What a status draws in the place of the time: the mark beside a title.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    /// The agent's own mark, animating.
    AgentWorking,
    /// The warning tone.
    Warning,
    /// A small amber dot: finished, and not yet seen.
    AmberDot,
    /// The danger tone.
    Danger,
    /// A hollow ring in the muted tone: seen, and nothing going on.
    Idle,
}

impl SessionStatus {
    /// The words for the status, for a row's second line, a tab's tooltip and a screen reader.
    pub fn words(&self) -> SharedString {
        match self {
            Self::Working => "Working".into(),
            Self::NeedsYou(Need::Approval) => "Needs approval".into(),
            Self::NeedsYou(Need::Question) => "Asks a question".into(),
            Self::Finished => "Finished, ready to look at".into(),
            Self::Idle => "Idle".into(),
            Self::Failed(reason) => format!("Stopped: {reason}").into(),
        }
    }

    /// Whether a row shows the words under its title. A working, finished or idle session shows none: its
    /// mark says it.
    pub fn has_note(&self) -> bool {
        matches!(self, Self::NeedsYou(_) | Self::Failed(_))
    }

    pub fn mark(&self) -> Mark {
        match self {
            Self::Working => Mark::AgentWorking,
            Self::NeedsYou(_) => Mark::Warning,
            Self::Finished => Mark::AmberDot,
            Self::Failed(_) => Mark::Danger,
            Self::Idle => Mark::Idle,
        }
    }

    /// The reader owes a move or has something to look at: what a sidebar must never fold away.
    pub fn wants_attention(&self) -> bool {
        matches!(self, Self::NeedsYou(_) | Self::Finished)
    }

    /// The title is in ink, not muted: a session that is going on or that has news.
    pub fn title_is_ink(&self) -> bool {
        !matches!(self, Self::Idle)
    }

    pub fn needs_you(&self) -> bool {
        matches!(self, Self::NeedsYou(_))
    }
}
