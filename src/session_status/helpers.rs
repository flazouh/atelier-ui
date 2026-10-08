use gpui_kit::SharedString;

use super::types::REASON_CHARS;

/// The first line of `reason`, cut to what a row can hold.
pub fn short_reason(reason: &str) -> SharedString {
    let line = reason.lines().find(|l| !l.trim().is_empty()).unwrap_or("").trim();
    if line.chars().count() <= REASON_CHARS {
        return line.to_string().into();
    }
    let mut cut: String = line.chars().take(REASON_CHARS - 1).collect();
    cut.push('…');
    cut.into()
}
