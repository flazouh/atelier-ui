use std::time::Duration;

use gpui_kit::{Hsla, SharedString};

use crate::{icon::IconName, theme::Theme};
use super::helpers::disc_for;

/// The stack is at most this wide (`max-w-sm`), and 16px short of the window's width when that is less.
pub(super) const MAX_WIDTH: f32 = 384.;

pub(super) const SIDE_GUTTER: f32 = 32.;

/// Distance from the window edge: `left-4`, `right-4`, `top-4`, `bottom-6`.
pub(super) const EDGE_X: f32 = 16.;

pub(super) const EDGE_TOP: f32 = 16.;

pub(super) const EDGE_BOTTOM: f32 = 24.;

/// The gap between toasts (`gap-2`).
pub(super) const GAP: f32 = 8.;

/// A toast comes in from this far below, and leaves this far to the right.
pub(super) const ENTER_RISE: f32 = 22.;

pub(super) const EXIT_SLIDE: f32 = 32.;

pub(super) const EXIT: f32 = 0.18;

/// The swap of icon and words: 280ms on the out curve, 8px of travel.
pub(super) const SWAP: f32 = 0.28;

pub(super) const SWAP_RISE: f32 = 8.;

/// A drag follows the pointer at this part of the distance, and lets go past 72px or 520px/s.
pub(super) const ELASTIC: f32 = 0.18;

pub(super) const DRAG_DISTANCE: f32 = 72.;

pub(super) const DRAG_SPEED: f32 = 520.;

/// The toast: `rounded-2xl p-3`, and its icon is 28px with a 14px glyph.
pub(super) const RADIUS: f32 = 16.;

pub(super) const PAD: f32 = 12.;

pub(super) const ICON_BOX: f32 = 28.;

pub(super) const ICON_GLYPH: f32 = 14.;

pub(super) const PILL: f32 = 28.;

/// The default time a toast stays: the hook's `defaultDuration`.
pub const DEFAULT_DURATION: Duration = Duration::from_millis(4200);

pub(super) const BORDER: f32 = 0.06;

/// A disc must differ from the surface by this contrast ratio to show, and is never stronger than this share of its tone.
pub(super) const DISC_VISIBLE: f32 = 1.08;

pub(super) const DISC_MOST: f32 = 0.3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastStatus {
    Neutral,
    Info,
    Loading,
    Success,
    Error,
}

impl ToastStatus {
    pub(super) fn icon(self) -> IconName {
        match self {
            ToastStatus::Neutral => IconName::Notifications,
            ToastStatus::Info => IconName::Info,
            ToastStatus::Loading => IconName::Progress,
            ToastStatus::Success => IconName::Check,
            ToastStatus::Error => IconName::Error,
        }
    }

    /// The glyph's colour and the disc behind it, on `surface`: the web's `text-muted-foreground bg-muted/60`,
    /// `text-primary bg-muted`, `text-emerald bg-emerald/10`, `text-destructive bg-destructive/10`. The tones of a theme
    /// are duller than emerald and rose, so a disc at the web's strength can vanish: it is made as strong as the
    /// tone needs to show against the surface (never past 30%), and the glyph as dark or light as needs 3:1 on it.
    pub(super) fn tones(self, theme: &Theme, surface: Hsla) -> (Hsla, Hsla) {
        let wash = theme.foreground.opacity(0.06);
        let (tone, web_alpha) = match self {
            ToastStatus::Neutral => return (theme.muted_foreground, wash),
            ToastStatus::Info | ToastStatus::Loading => return (theme.primary, wash),
            ToastStatus::Success => (theme.success, 0.1),
            ToastStatus::Error => (theme.danger, 0.1),
        };
        disc_for(theme, tone, web_alpha, surface)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastPosition {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl ToastPosition {
    pub const ALL: [ToastPosition; 6] = [
        ToastPosition::TopLeft,
        ToastPosition::TopCenter,
        ToastPosition::TopRight,
        ToastPosition::BottomLeft,
        ToastPosition::BottomCenter,
        ToastPosition::BottomRight,
    ];

    pub fn word(self) -> &'static str {
        match self {
            ToastPosition::TopLeft => "top-left",
            ToastPosition::TopCenter => "top-center",
            ToastPosition::TopRight => "top-right",
            ToastPosition::BottomLeft => "bottom-left",
            ToastPosition::BottomCenter => "bottom-center",
            ToastPosition::BottomRight => "bottom-right",
        }
    }

    pub(super) fn bottom(self) -> bool {
        matches!(self, ToastPosition::BottomLeft | ToastPosition::BottomCenter | ToastPosition::BottomRight)
    }
}

pub enum ToastEvent {
    /// The action button of the toast with this id was pressed.
    Action(SharedString),
}
