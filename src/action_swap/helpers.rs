use gpui_kit::Hsla;

use crate::{
    theme::{Theme},
};
use super::types::SwapVariant;

/// The fill and the words of a variant, at hover progress `hover` (0 to 1).
pub fn colors(variant: SwapVariant, theme: &Theme, hover: f32) -> (Hsla, Hsla) {
    // The same fills and words as the button of that variant: the commit strip had buttons before this part.
    let button = match variant {
        SwapVariant::Primary => crate::button::ButtonVariant::Primary,
        SwapVariant::Secondary => crate::button::ButtonVariant::Secondary,
        SwapVariant::Ghost => crate::button::ButtonVariant::Ghost,
    };
    crate::button::colors(button, theme, hover, false)
}
