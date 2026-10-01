//! Finds pull request references (`#3344`) in agent text, so [`crate::agent_text::AgentText`] can turn
//! the ones the app knows into [`crate::pr_chip::PrChip`]s.
//!
//! A reference is `#` and a number with no leading zero, standing alone: a word boundary before the `#`
//! and after the digits. It is not one inside a code span or a fenced code block, inside a URL or a
//! Markdown link, or after a backslash or `&` (an escape or an HTML entity). A `#N` that opens a line
//! counts: a Markdown heading needs `# ` with a space, and agents often start a line with the PR. Only
//! numbers the app confirms become chips, so a false match costs little.

use std::ops::Range;

/// Each reference's byte range, `#` included, and its number, in text order.
pub fn pr_refs(text: &str) -> Vec<(Range<usize>, u64)> {
    let skip = skipped(text);
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    for (at, _) in text.match_indices('#') {
        if skip.iter().any(|r| r.contains(&at)) || !stands_alone_before(text, at) {
            continue;
        }
        let digits = bytes[at + 1..].iter().take_while(|b| b.is_ascii_digit()).count();
        let end = at + 1 + digits;
        if digits == 0 || bytes[at + 1] == b'0' || text[end..].chars().next().is_some_and(is_word) {
            continue;
        }
        if let Ok(number) = text[at + 1..end].parse() {
            out.push((at..end, number));
        }
    }
    out
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Nothing that joins it to a word, an escape, an entity, or another `#` comes right before the `#`.
fn stands_alone_before(text: &str, at: usize) -> bool {
    text[..at].chars().next_back().is_none_or(|c| !is_word(c) && !matches!(c, '\\' | '&' | '#'))
}

/// Byte ranges where no reference counts: fenced code blocks, code spans, URLs, and Markdown links.
fn skipped(text: &str) -> Vec<Range<usize>> {
    let mut out = fences(text);
    out.extend(code_spans(text, &out.clone()));
    out.extend(urls(text));
    out.extend(links(text));
    out
}

/// Fenced code blocks: from a line that opens with three or more backticks or tildes to the line that
/// closes it with the same character, or to the end of the text.
fn fences(text: &str) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let mut open: Option<(usize, char, usize)> = None;
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_start_matches(' ');
        let fence = trimmed.chars().next().filter(|c| matches!(c, '`' | '~')).map(|c| (c, trimmed.chars().take_while(|&x| x == c).count()));
        match (open, fence) {
            (None, Some((c, n))) if n >= 3 => open = Some((at, c, n)),
            (Some((start, c, n)), Some((close, m))) if close == c && m >= n => {
                out.push(start..at + line.len());
                open = None;
            }
            _ => {}
        }
        at += line.len();
    }
    if let Some((start, _, _)) = open {
        out.push(start..text.len());
    }
    out
}

/// Code spans: a run of backticks up to the next run of the same length. A run with no partner is
/// plain text. Backticks inside `fenced` are left alone.
fn code_spans(text: &str, fenced: &[Range<usize>]) -> Vec<Range<usize>> {
    let bytes = text.as_bytes();
    let run_at = |i: usize| bytes[i..].iter().take_while(|&&b| b == b'`').count();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'`' || fenced.iter().any(|r| r.contains(&i)) {
            i += 1;
            continue;
        }
        let n = run_at(i);
        let mut j = i + n;
        let close = loop {
            match bytes[j..].iter().position(|&b| b == b'`') {
                None => break None,
                Some(k) => {
                    let m = run_at(j + k);
                    if m == n {
                        break Some(j + k + m);
                    }
                    j += k + m;
                }
            }
        };
        match close {
            Some(end) => {
                out.push(i..end);
                i = end;
            }
            None => i += n,
        }
    }
    out
}

/// Bare URLs and autolinks: from `http://`, `https://` or `www.` up to the next space or `>`.
fn urls(text: &str) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    for scheme in ["http://", "https://", "www."] {
        for (at, _) in text.match_indices(scheme) {
            let end = text[at..].find(|c: char| c.is_whitespace() || c == '>').map_or(text.len(), |n| at + n);
            out.push(at..end);
        }
    }
    out
}

/// Markdown links, `[text](destination)`, text and destination both.
fn links(text: &str) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    for (mid, _) in text.match_indices("](") {
        let Some(open) = text[..mid].rfind('[') else { continue };
        let Some(close) = text[mid..].find(')') else { continue };
        out.push(open..mid + close + 1);
    }
    out
}

#[cfg(test)]
mod tests;
