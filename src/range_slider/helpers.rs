use super::types::{FILL_INSET, HANDLE_START, KNOB, MOST_TICKS, TRAVEL_LOSS};

/// The nearest legal value on `[min, max]` for `step` (`snapSliderValue`). `max` counts as a candidate when the
/// step does not divide the range.
pub fn snap(next: f32, min: f32, max: f32, step: f32) -> f32 {
    if max <= min {
        return min;
    }
    if step <= 0. {
        return next.clamp(min, max);
    }
    let whole = (((max - min) / step) * 1e6).round() / 1e6;
    let whole = whole.floor();
    let last = min + whole * step;
    let to_grid = (((next - min) / step).round() * step + min).clamp(min, last);
    let snapped = if last < max && (next - max).abs() <= (next - to_grid).abs() { max } else { to_grid };
    (snapped * 1e6).round() / 1e6
}

/// The value as a percent of the range.
pub fn percent(value: f32, min: f32, max: f32) -> f32 {
    if max > min { (value.clamp(min, max) - min) / (max - min) * 100. } else { 0. }
}

/// The values that get a dot, when there are any: one for each step, if there are 50 steps or fewer.
pub fn ticks(min: f32, max: f32, step: f32) -> Vec<f32> {
    if max <= min || step <= 0. {
        return Vec::new();
    }
    let steps = ((((max - min) / step) * 1e6).round() / 1e6).floor() as usize;
    if steps == 0 || steps > MOST_TICKS {
        return Vec::new();
    }
    (0..=steps).map(|i| ((min + i as f32 * step) * 1e6).round() / 1e6).collect()
}

/// Where the key goes: the value a key press asks for, or `None` for a key the slider does not use.
pub fn key_value(key: &str, current: f32, min: f32, max: f32, step: f32) -> Option<f32> {
    Some(match key {
        "right" | "up" => current + step,
        "left" | "down" => current - step,
        "pageup" => current + step * 10.,
        "pagedown" => current - step * 10.,
        "home" => min,
        "end" => max,
        _ => return None,
    })
}

/// The handle's left edge and the fill's offset, for a track `width` wide and the value at `percent`.
pub fn geometry(width: f32, percent: f32) -> (f32, f32) {
    let handle = HANDLE_START + (width - TRAVEL_LOSS).max(0.) * percent / 100.;
    let clip = width - 2. * FILL_INSET;
    let fill = if percent >= 100. { 0. } else { (percent - 100.) / 100. * clip + 14. - 0.16 * percent };
    (handle, fill)
}

/// The compact knob's left edge and the fill's width at `percent` of a rail `width` wide: the knob travels
/// inside the rail, and the fill ends at its centre.
pub fn compact_geometry(width: f32, percent: f32) -> (f32, f32) {
    let knob_left = (width - KNOB).max(0.) * percent.clamp(0., 100.) / 100.;
    (knob_left, knob_left + KNOB / 2.)
}
