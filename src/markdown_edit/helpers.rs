use std::ops::Range;

use super::types::Format;

/// `text` with `format` applied to `chosen`, and the range to choose after.
pub fn apply(text: &str, chosen: Range<usize>, format: Format) -> (String, Range<usize>) {
    let wrap = |open: &str, close: &str| {
        let inner = &text[chosen.clone()];
        let out = format!("{}{open}{inner}{close}{}", &text[..chosen.start], &text[chosen.end..]);
        let start = chosen.start + open.len();
        (out, start..start + inner.len())
    };
    match format {
        Format::Bold => wrap("**", "**"),
        Format::Italic => wrap("_", "_"),
        Format::Code => wrap("`", "`"),
        Format::Link => {
            let (out, _) = wrap("[", "](url)");
            let url = chosen.end + 3;
            (out, url..url + 3)
        }
        Format::Quote => mark_lines(text, chosen, "> "),
        Format::List => mark_lines(text, chosen, "- "),
    }
}

/// Puts `mark` at the start of every line `chosen` touches.
fn mark_lines(text: &str, chosen: Range<usize>, mark: &str) -> (String, Range<usize>) {
    let first = text[..chosen.start].rfind('\n').map_or(0, |i| i + 1);
    let last = text[chosen.end..].find('\n').map_or(text.len(), |i| chosen.end + i);
    let lines: Vec<String> = text[first..last].split('\n').map(|l| format!("{mark}{l}")).collect();
    let marked = lines.join("\n");
    let added = marked.len() - (last - first);
    let out = format!("{}{marked}{}", &text[..first], &text[last..]);
    (out, chosen.start + mark.len()..chosen.end + added)
}
