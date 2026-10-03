use gpui_kit::Hsla;

use crate::theme::Theme;

use super::super::{
    consts::{HOT_AT, WARM_AT},
    enums::Pressure,
};

impl Pressure {
    /// How pressing a `used` fraction is.
    pub fn of(used: f32) -> Self {
        if used >= HOT_AT {
            Self::Hot
        } else if used >= WARM_AT {
            Self::Warm
        } else {
            Self::Calm
        }
    }

    /// The colour of a gauge at this pressure; calm is the quiet one.
    pub(in super::super) fn ink(self, theme: &Theme) -> Hsla {
        match self {
            Self::Calm => theme.muted_foreground,
            Self::Warm => theme.warning,
            Self::Hot => theme.danger,
        }
    }
}
