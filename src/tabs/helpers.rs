use gpui_kit::Hsla;

use crate::theme::Theme;
use super::types::TabsVariant;

/// How much of a tab (`tab_left`, `tab_width`) the indicator (`left`, `width`) covers, 0 to 1.
pub fn covered(tab_left: f32, tab_width: f32, left: f32, width: f32) -> f32 {
    if tab_width <= 0. {
        return 0.;
    }
    let overlap = (tab_left + tab_width).min(left + width) - tab_left.max(left);
    (overlap / tab_width).clamp(0., 1.)
}

pub(super) fn list_fill(theme: &Theme, variant: TabsVariant) -> Option<Hsla> {
    (!matches!(variant, TabsVariant::Underline) && !variant.is_editor()).then_some(theme.card)
}
