use gpui_kit::Hsla;

use crate::theme::Theme;
use super::types::{FULL_AT, Level, MILLION, THOUSAND, WARN_AT};

/// The share of `window` that `used` fills, from 0 to 1. An empty window counts as full.
pub fn fraction(used: u64, window: u64) -> f32 {
    if window == 0 {
        return 1.;
    }
    (used as f64 / window as f64).clamp(0., 1.) as f32
}

pub fn level(fraction: f32) -> Level {
    if fraction >= FULL_AT {
        Level::Full
    } else if fraction >= WARN_AT {
        Level::Filling
    } else {
        Level::Room
    }
}

/// A token count in few characters: 950, 84k, 1.2M.
pub fn tokens(count: u64) -> String {
    if count >= MILLION {
        let millions = count as f64 / MILLION as f64;
        let words = format!("{millions:.1}");
        format!("{}M", words.strip_suffix(".0").unwrap_or(&words))
    } else if count >= THOUSAND {
        format!("{}k", count / THOUSAND)
    } else {
        count.to_string()
    }
}

/// The words on hover: "84k of 200k tokens (42%)".
pub fn summary(used: u64, window: u64) -> String {
    let percent = (fraction(used, window) * 100.).round() as u32;
    format!("{} of {} tokens ({percent}%)", tokens(used), tokens(window))
}

/// The colour of the ring at `level`, and of a part of the bar that stands for the whole of what is in use.
pub fn ink(level: Level, theme: &Theme) -> Hsla {
    match level {
        Level::Room => theme.muted_foreground,
        Level::Filling => theme.warning,
        Level::Full => theme.danger,
    }
}
