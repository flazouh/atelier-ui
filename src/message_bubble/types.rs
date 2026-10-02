#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MessageBubbleVariant {
    Solid,
    #[default]
    Soft,
    Tint,
    /// beui's `outline`, borderless: a `card_strong` fill stands in for the border.
    Borderless,
    Danger,
    Ghost,
}

/// The bubble's padding across and down and its line height, in pixels.
pub const PAD_X: f32 = 12.;

pub const PAD_Y: f32 = 6.;

pub const LINE: f32 = 20.;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MessageBubbleAlign {
    #[default]
    Start,
    End,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MessageBubbleGroupSpacing {
    #[default]
    Compact,
    Default,
}
