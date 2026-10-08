use super::super::structs::MiniBar;

/// The bars of a one-colour chart `height` tall, scaled to the tallest value. A value above nothing shows at least a
/// pixel. Only the top corners are rounded.
pub fn mini_bars(values: &[f32], height: f32, radius: f32) -> Vec<MiniBar> {
    let clean = |v: &f32| if v.is_finite() { v.max(0.) } else { 0. };
    let max = values.iter().map(clean).fold(0., f32::max);
    values
        .iter()
        .map(clean)
        .map(|v| {
            let h = if max > 0. && v > 0. { (height * v / max).max(1.) } else { 0. };
            MiniBar { height: h, top_radius: radius.min(h) }
        })
        .collect()
}
