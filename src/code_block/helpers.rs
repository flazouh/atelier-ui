use std::ops::Range;

use gpui_kit::{HighlightStyle, SharedString};

/// "1\n2\n...\nN": the whole gutter as one text.
pub(super) fn gutter_numbers(count: usize) -> SharedString {
    use std::fmt::Write;
    let mut out = String::with_capacity(count * 4);
    for n in 1..=count {
        if n > 1 {
            out.push('\n');
        }
        let _ = write!(out, "{n}");
    }
    out.into()
}

/// The per-line runs of `code` as runs over the whole text, each range moved by its line's start.
pub(super) fn whole_text_runs(code: &str, lines: &[crate::syntax::LineRuns]) -> Vec<(Range<usize>, HighlightStyle)> {
    let mut out = Vec::with_capacity(lines.iter().map(Vec::len).sum());
    let mut start = 0;
    for (line, runs) in code.split('\n').zip(lines) {
        out.extend(runs.iter().map(|(range, style)| (start + range.start..start + range.end, *style)));
        start += line.len() + 1;
    }
    out
}
