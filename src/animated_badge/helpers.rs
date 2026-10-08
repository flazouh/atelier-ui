use gpui_kit::Hsla;

use super::types::{BadgeStatus, PULSE_MILLIS, PULSE_WASH};
use crate::{icon::IconName, motion::keyframes, theme::Theme};

/// The wash's strength `millis` into the pulse.
pub fn pulse_at(millis: u128) -> f32 {
    let t = (millis % PULSE_MILLIS) as f32 / PULSE_MILLIS as f32;
    keyframes(
        &[PULSE_WASH.0, PULSE_WASH.1, PULSE_WASH.0],
        &[0., 0.5, 1.],
        1.,
        [0.42, 0., 0.58, 1.],
        t,
    )
}

/// The words' colour and the fill of a status: the old `Badge`'s.
pub fn colors(status: BadgeStatus, theme: &Theme) -> (Hsla, Hsla) {
    let tone = |c: Hsla| (c, c.opacity(0.14));
    match status {
        BadgeStatus::Neutral => (theme.muted_foreground, theme.card_strong),
        BadgeStatus::Info | BadgeStatus::Loading => tone(theme.info),
        BadgeStatus::Success => tone(theme.success),
        BadgeStatus::Warning => tone(theme.warning),
        BadgeStatus::Danger => tone(theme.danger),
    }
}

pub(super) fn icon_of(status: BadgeStatus) -> IconName {
    match status {
        BadgeStatus::Neutral => IconName::Circle,
        BadgeStatus::Info => IconName::Info,
        BadgeStatus::Success => IconName::Check,
        BadgeStatus::Warning => IconName::Warning,
        BadgeStatus::Danger => IconName::Close,
        BadgeStatus::Loading => IconName::Progress,
    }
}
