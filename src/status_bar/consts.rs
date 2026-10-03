/// The bar's height, in design pixels.
pub const HEIGHT: f32 = 28.;

/// How many samples of the processor the sparkline draws, newest at the right.
pub(super) const SPARK_BARS: usize = 24;
pub(super) const SPARK_WIDTH: f32 = 2.;
pub(super) const SPARK_GAP: f32 = 1.;
pub(super) const SPARK_HEIGHT: f32 = 12.;

/// A gauge's track.
pub(super) const GAUGE_WIDTH: f32 = 40.;
pub(super) const GAUGE_HEIGHT: f32 = 4.;

/// A provider's mark.
pub(super) const MARK: f32 = 13.;

/// From where a fraction used is warm, and hot.
pub(super) const WARM_AT: f32 = 0.6;
pub(super) const HOT_AT: f32 = 0.85;
