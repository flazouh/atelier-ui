use super::super::structs::{BarSegment, Series};

/// The segments of one stacked bar, the lowest first, in design pixels: `per_token` pixels to each token. The edges are
/// rounded as a run (each segment's top is the rounded running total), so the segments touch with no gap and add up to
/// the day's height. A part too small to show is left out. Only the top segment has rounded top corners, of `radius` or
/// less when it is shorter.
pub fn bar_segments(parts: &[(Series, u64)], per_token: f32, radius: f32) -> Vec<BarSegment> {
    let mut segments: Vec<BarSegment> = Vec::new();
    let (mut tokens, mut below) = (0u64, 0f32);
    for (series, value) in parts {
        tokens += value;
        let edge = (tokens as f32 * per_token).round();
        let height = edge - below;
        if *value > 0 && height > 0. {
            segments.push(BarSegment { series: *series, height, top_radius: 0. });
            below = edge;
        }
    }
    if let Some(top) = segments.last_mut() {
        top.top_radius = radius.min(top.height);
    }
    segments
}
