use std::rc::Rc;

use gpui_kit::{App, Corners, KeyDownEvent, Window};

use super::structs::Metrics;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Ghost,
    /// The text color as the fill: the strongest action on a card, such as "Allow once".
    Invert,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    /// The default, and the size of dense places: a card's actions, the review bar, the hunk bar.
    #[default]
    Sm,
    /// For an action that must stand out: the start screen's Open Folder, an empty state's first action.
    Md,
    Lg,
    Xl,
    /// A square icon-only button, as tall as the default (28) so it lines up with a Small text button.
    Icon,
    /// A denser square icon-only button (24), for rows and headers: a project header's + and more, a
    /// tab's close, a row's actions. Its hit area is never below 24.
    IconSm,
}

impl ButtonSize {
    pub(super) fn metrics(self) -> Metrics {
        match self {
            // One step below Medium, in the same ratio Medium keeps to Large.
            Self::Sm => Metrics { height: 28., pad_x: 10., gap: 6., text: 11., icon: 14., chip_inset: 4. },
            Self::Md => Metrics { height: 32., pad_x: 12., gap: 8., text: 12., icon: 14., chip_inset: 5. },
            Self::Lg => Metrics { height: 36., pad_x: 16., gap: 10., text: 13., icon: 14., chip_inset: 6. },
            Self::Xl => Metrics { height: 40., pad_x: 18., gap: 10., text: 14., icon: 16., chip_inset: 6. },
            Self::Icon => Metrics { height: 28., pad_x: 0., gap: 0., text: 13., icon: 14., chip_inset: 0. },
            Self::IconSm => Metrics { height: 24., pad_x: 0., gap: 0., text: 13., icon: 14., chip_inset: 0. },
        }
    }
}

/// Every corner rounded, as a button alone has them.
pub const ROUND: Corners<bool> = Corners { top_left: true, top_right: true, bottom_left: true, bottom_right: true };

pub(super) type KeyHandler = Rc<dyn Fn(&KeyDownEvent, &mut Window, &mut App)>;
