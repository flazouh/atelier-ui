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
