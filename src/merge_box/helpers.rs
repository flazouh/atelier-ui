use crate::{icon::IconName, merge::Blocker, theme::Theme};

/// A blocker's mark: what kind of hold it is, in the status tones.
pub(super) fn mark(blocker: &Blocker, theme: &Theme) -> (IconName, gpui_kit::Hsla) {
    match blocker {
        Blocker::Draft => (IconName::PrDraft, theme.muted_foreground),
        Blocker::Conflicts(_) => (IconName::Block, theme.danger),
        Blocker::Behind(_) => (IconName::ArrowDown, theme.warning),
        Blocker::ChecksFailing(_) => (IconName::Error, theme.danger),
        Blocker::ChecksRunning(_) => (IconName::Progress, theme.info),
        Blocker::ReviewMissing | Blocker::ChangesAsked(_) => (IconName::Visibility, theme.warning),
        Blocker::Queue(_) => (IconName::Schedule, theme.info),
        Blocker::NoRights => (IconName::Lock, theme.muted_foreground),
    }
}
