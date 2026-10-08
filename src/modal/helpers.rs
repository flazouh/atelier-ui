use super::types::{BORDER, PAD};
use crate::theme::Theme;

/// The dim over the page, at progress `t` of its fade: the theme's shadow colour.
pub fn scrim(theme: &Theme, t: f32) -> gpui_kit::Hsla {
    gpui_kit::Hsla {
        a: 0.28 * t,
        ..theme.shadow
    }
}

/// The panel's height for a view of `content` px: the view, its padding and its border.
pub fn panel_height(content: f32) -> f32 {
    content + 2. * PAD + 2. * BORDER
}
