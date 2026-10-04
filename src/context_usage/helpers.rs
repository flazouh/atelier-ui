use gpui_kit::Hsla;

use crate::{context_meter::fraction, theme::{Theme, mix}};
use super::types::{BAR, ContextPart};

/// A token count with one decimal when it has one: 950, 4.1K, 106.3K, 300K, 1.2M.
pub fn precise(count: u64) -> String {
    let (value, unit) = match count {
        1_000_000.. => (count as f64 / 1_000_000., "M"),
        1_000.. => (count as f64 / 1_000., "K"),
        _ => return count.to_string(),
    };
    let words = format!("{value:.1}");
    format!("{}{unit}", words.strip_suffix(".0").unwrap_or(&words))
}

/// The two ends of the header: "35% Full" and "~106.3K / 300K Tokens".
pub fn header(used: u64, window: u64) -> (String, String) {
    let percent = (fraction(used, window) * 100.).round() as u32;
    (format!("{percent}% Full"), format!("~{} / {} Tokens", precise(used), precise(window)))
}

/// What the parts add up to, and what is left of the window.
pub fn totals(parts: &[ContextPart], window: u64) -> (u64, u64) {
    let sum = parts.iter().map(|part| part.tokens).sum::<u64>();
    (sum, window.saturating_sub(sum))
}

/// The share of the bar each part takes, in order. A part is its share of the window; parts that add up past the window
/// are shrunk to fit it, so the bar never runs over.
pub fn shares(parts: &[ContextPart], window: u64) -> Vec<f32> {
    let scale = parts.iter().map(|part| part.tokens).sum::<u64>().max(window).max(1) as f64;
    parts.iter().map(|part| (part.tokens as f64 / scale) as f32).collect()
}

/// The colour of the `index`th part's swatch and segment, from the theme. Seven kinds of part have their own; more cycle.
pub fn swatch(theme: &Theme, index: usize) -> Hsla {
    match index % 7 {
        0 => theme.muted_foreground,
        1 => theme.info,
        2 => theme.warning_fill,
        3 => mix(theme.info, theme.danger, 0.5),
        4 => mix(theme.info, theme.success, 0.5),
        5 => theme.success,
        _ => mix(theme.danger, theme.foreground, 0.35),
    }
}

/// The height of the panel with `rows` rows, for a popover that has to know it before it draws.
pub fn height(rows: usize) -> f32 {
    let rows = rows.max(1) as f32;
    // The border, the padding, the title and numbers, the bar, then the rows, with the gaps between all.
    2. + 32. + 38. + BAR + 3. * 12. + rows * 18. + (rows - 1.) * 6.
}
