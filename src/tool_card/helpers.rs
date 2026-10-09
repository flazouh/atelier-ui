use gpui_kit::Hsla;

use super::types::{ROWS_SHOWN, ToolCardState, ToolGap, ToolTone};
use crate::{animated_badge::BadgeStatus, badge::Tone, theme::Theme};

/// How many rows of a list of `total` show.
pub(super) fn rows_shown(total: usize, expanded: bool) -> usize {
    if expanded {
        total
    } else {
        total.min(ROWS_SHOWN)
    }
}

/// The words on the list's button, `None` when the list has no more rows than it shows at first.
pub(super) fn more_label(total: usize, expanded: bool) -> Option<String> {
    if total <= ROWS_SHOWN {
        return None;
    }
    Some(if expanded {
        "Show less".to_string()
    } else {
        format!("Show {} more", total - ROWS_SHOWN)
    })
}

/// The words that say rows were left out by the app, `None` when none were.
pub(super) fn hidden_label(hidden: usize) -> Option<String> {
    (hidden > 0).then(|| {
        if hidden == 1 {
            "1 more not shown".to_string()
        } else {
            format!("{hidden} more not shown")
        }
    })
}

pub(super) fn state_badge(state: ToolCardState) -> (BadgeStatus, &'static str) {
    match state {
        ToolCardState::Running => (BadgeStatus::Loading, "Running"),
        ToolCardState::Waiting => (BadgeStatus::Warning, "Waiting for approval"),
        ToolCardState::Done => (BadgeStatus::Success, "Done"),
        ToolCardState::Failed => (BadgeStatus::Danger, "Failed"),
    }
}

/// The design pixels of a gap.
pub(super) fn gap_px(gap: ToolGap) -> f32 {
    match gap {
        ToolGap::Xs => 2.,
        ToolGap::Sm => 6.,
        ToolGap::Md => 10.,
    }
}

/// The colour of a mark or a word with this tone, `quiet` for the neutral one.
pub(super) fn tone_color(tone: ToolTone, quiet: Hsla, theme: &Theme) -> Hsla {
    match tone {
        ToolTone::Neutral => quiet,
        ToolTone::Info => theme.info,
        ToolTone::Success => theme.success,
        ToolTone::Warning => theme.warning,
        ToolTone::Danger => theme.danger,
    }
}

pub(super) fn badge_tone(tone: ToolTone) -> Tone {
    match tone {
        ToolTone::Neutral => Tone::Neutral,
        ToolTone::Info => Tone::Info,
        ToolTone::Success => Tone::Success,
        ToolTone::Warning => Tone::Warning,
        ToolTone::Danger => Tone::Danger,
    }
}

/// The letter an avatar shows: the first letter of the name, in capitals, or a dot for a name with none.
pub(super) fn avatar_letter(name: &str) -> String {
    name.chars()
        .find(|c| c.is_alphanumeric())
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "·".into())
}
