#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeStatus {
    #[default]
    Neutral,
    Info,
    Success,
    Warning,
    Danger,
    Loading,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeSize {
    Small,
    #[default]
    Medium,
}

impl BadgeSize {
    /// Height, padding across, gap, words and icon.
    pub fn metrics(self) -> (f32, f32, f32, f32, f32) {
        match self {
            BadgeSize::Small => (20., 8., 4., 11., 12.),
            BadgeSize::Medium => (24., 10., 6., 12., 14.),
        }
    }
}

/// How long the loading pulse takes, and how strong the wash is at its ends and its middle.
pub const PULSE_MILLIS: u128 = 1600;

pub const PULSE_WASH: (f32, f32) = (0.08, 0.16);
