use super::structs::Ribbon;

/// A bar never gets wider than this, so a wide row keeps its look and centers instead of growing fat bars.
pub const MAX_BAR_WIDTH: f32 = 6.;

pub const RIBBONS: [Ribbon; 4] = [
    Ribbon { waves: 1.6, speed: 1.0, phase: 0.0, alpha: 0.22, gain: 1.00 },
    Ribbon { waves: 2.4, speed: -1.3, phase: 1.7, alpha: 0.17, gain: 0.80 },
    Ribbon { waves: 3.3, speed: 1.9, phase: 3.1, alpha: 0.13, gain: 0.62 },
    Ribbon { waves: 4.6, speed: -2.4, phase: 4.4, alpha: 0.10, gain: 0.45 },
];

/// The quietest the ribbons get while it is on: a thin line, so the field never looks dead.
pub const FLOOR: f32 = 0.06;

/// What the ribbons' average is multiplied by, because the average of four ribbons that rarely peak together sits
/// well under the tallest one.
pub const BAR_LIFT: f32 = 1.9;

/// The shortest a bar gets, in pixels: a dot as wide as the bar, so a quiet row still reads as a row of bars.
pub const MIN_BAR: f32 = 3.;
