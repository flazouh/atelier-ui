use super::Series;

/// One segment of a stacked bar, in design pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarSegment {
    pub series: Series,
    pub height: f32,
    /// The radius of both top corners: only the top segment of a bar has one; the bottom corners are always square.
    pub top_radius: f32,
}
