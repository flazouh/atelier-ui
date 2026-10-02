use std::time::Instant;

/// What the agent is doing. The label follows from it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThinkingPhase {
    /// Thinking since `since`. The label changes at 15, 30, 45, and 60 seconds.
    Thinking { since: Instant },
    /// Thinking has ended: "Thought for {seconds}s", with no shimmer.
    Thought { seconds: u64 },
    Connecting,
    Sending,
    Starting,
    Preparing,
    Waiting,
    RunningTools,
}

impl ThinkingPhase {
    /// The CLI's `requesting` mode: the glimmer walks left to right, four times as fast.
    pub(super) fn requesting(self) -> bool {
        matches!(self, Self::Connecting | Self::Sending | Self::Starting | Self::Preparing | Self::Waiting)
    }

    pub(super) fn elapsed_s(self) -> f32 {
        match self {
            Self::Thinking { since } => since.elapsed().as_secs_f32(),
            _ => 0.,
        }
    }
}

/// How the label moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThinkingStyle {
    /// The CLI's glyph glimmer.
    Shimmer(Shimmer),
    /// The desktop app's opacity breath: 1 to 0.75 and back over 2s, after 3s.
    Breath,
}

impl Default for ThinkingStyle {
    fn default() -> Self {
        Self::Shimmer(Shimmer::default())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Shimmer {
    /// A soft band that glides between clusters.
    #[default]
    Smooth,
    /// The CLI exactly: three whole clusters, stepping.
    Stepped,
    /// Cursor's status shimmer: the label at 60% ink, with a full-ink band crossing it left to right once a second.
    Cursor,
}

/// What sits between segments in text form, for tests and accessibility: nothing but the gap.
pub(crate) const SEGMENT_GAP_TEXT: &str = "";
