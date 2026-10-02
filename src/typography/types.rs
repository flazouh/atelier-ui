use gpui_kit::Pixels;

use crate::scale::px;

pub const FONT_FAMILY: &str = "Geist";

/// Code, paths, commands, and numbers that change.
pub const MONO_FONT_FAMILY: &str = "Geist Mono";

/// Space between the segments of a status line, such as "Done in 38s" and "12 tool calls". Segments
/// part by space alone, never by a glyph such as a middle dot.
pub const SEGMENT_GAP: f32 = 12.;

/// Tailwind sizes used by beui, as (font size, line height).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextSize {
    Xs,
    Sm,
    Base,
    Lg,
    Xl,
}

impl TextSize {
    pub fn font_size(self) -> Pixels {
        px(match self {
            Self::Xs => 12.,
            Self::Sm => 14.,
            Self::Base => 16.,
            Self::Lg => 18.,
            Self::Xl => 20.,
        })
    }

    pub fn line_height(self) -> Pixels {
        px(match self {
            Self::Xs => 16.,
            Self::Sm => 20.,
            Self::Base => 24.,
            Self::Lg => 28.,
            Self::Xl => 28.,
        })
    }
}
