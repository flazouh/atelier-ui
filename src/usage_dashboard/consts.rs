/// The height of the plot of the day chart, in design pixels.
pub(super) const PLOT_HEIGHT: f32 = 188.;
pub(super) const BAR_GAP: f32 = 10.;
/// The top corners of the top segment of a stacked bar.
pub(super) const TOP_RADIUS: f32 = 4.;
/// The top corners of a bar of a one-colour mini chart.
pub(super) const MINI_RADIUS: f32 = 3.;
pub(super) const MINI_HEIGHT: f32 = 76.;
pub(super) const MINI_GAP: f32 = 4.;
pub(super) const MINI_WIDTH: f32 = 520.;
/// The ticks above the axis' zero.
pub(super) const AXIS_TICKS: usize = 4;
/// The width of the axis' labels.
pub(super) const AXIS_WIDTH: f32 = 34.;
/// A bar that is not the highlighted day, and a mini bar that is not the last, are a little quieter.
pub(super) const PAST_ALPHA: f32 = 0.86;
pub(super) const MINI_PAST_ALPHA: f32 = 0.7;
pub(super) const TILE_HEIGHT: f32 = 104.;
pub(super) const GAUGE_HEIGHT: f32 = 4.;
pub(super) const DOT: f32 = 8.;
/// The sparkline of a session row.
pub(super) const SPARK_WIDTH: f32 = 150.;
pub(super) const SPARK_HEIGHT: f32 = 22.;
pub(super) const SPARK_STROKE: f32 = 1.5;
/// The columns of the sessions table, after the title: the sparkline, the tokens, the cost and the chevron.
pub(super) const TOKENS_WIDTH: f32 = 110.;
pub(super) const COST_WIDTH: f32 = 90.;
pub(super) const CHEVRON_WIDTH: f32 = 24.;
/// The width of the "where the tokens went" column of an open session.
pub(super) const SPLIT_WIDTH: f32 = 292.;
