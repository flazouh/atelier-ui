/// The points of a sparkline in a unit box, x from 0 to 1 and y from 0 at the top to 1 at the bottom, scaled to the
/// tallest value with a margin above and below. One value is a flat line across.
pub fn spark_points(values: &[f32]) -> Vec<(f32, f32)> {
    let clean: Vec<f32> = values.iter().map(|v| if v.is_finite() { v.max(0.) } else { 0. }).collect();
    let max = clean.iter().copied().fold(0., f32::max);
    let y = |v: f32| if max > 0. { 0.92 - 0.84 * v / max } else { 0.92 };
    match clean.as_slice() {
        [] => Vec::new(),
        [only] => vec![(0., y(*only)), (1., y(*only))],
        _ => clean.iter().enumerate().map(|(i, v)| (i as f32 / (clean.len() - 1) as f32, y(*v))).collect(),
    }
}
