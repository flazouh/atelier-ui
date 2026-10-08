use super::super::{consts::AXIS_TICKS, structs::Axis};

/// The scale for a tallest day of `max` tokens: a round step (1, 2 or 5 times a power of ten), so that the top is a round
/// number that holds the day. A chart of nothing still has a scale.
pub fn axis(max: u64) -> Axis {
    let raw = (max.max(1) as f64 / AXIS_TICKS as f64).max(1.);
    let magnitude = 10f64.powf(raw.log10().floor());
    let step = [1., 2., 5., 10.]
        .iter()
        .map(|unit| unit * magnitude)
        .find(|step| *step >= raw)
        .unwrap_or(10. * magnitude)
        .round() as u64;
    Axis { top: step * AXIS_TICKS as u64, ticks: (1..=AXIS_TICKS as u64).map(|n| step * n).collect() }
}
