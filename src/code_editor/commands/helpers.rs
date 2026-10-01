use std::ops::Range;

use super::structs::Edit;

/// The comment marker for a language, as gpui-base names languages. `None` means the language has no
/// line comment, so the command does nothing rather than guess.
pub fn comment_prefix(language: &str) -> Option<&'static str> {
    match language {
        "rust" | "typescript" | "tsx" | "javascript" | "go" => Some("//"),
        "python" | "toml" | "yaml" => Some("#"),
        _ => None,
    }
}

pub(super) fn line_start(text: &str, offset: usize) -> usize {
    text[..offset].rfind('\n').map_or(0, |i| i + 1)
}

pub(super) fn line_end(text: &str, offset: usize) -> usize {
    text[offset..].find('\n').map_or(text.len(), |i| offset + i)
}

/// The whole lines a selection touches, without the last line's newline. A selection that ends at
/// the very start of a line does not take that line, as in Zed: selecting lines 2 and 3 by dragging
/// to the start of line 4 means lines 2 and 3. The empty line after a final newline is the exception:
/// a selection that reaches the end of the text takes everything, so select-all then delete empties it.
pub fn line_span(text: &str, selection: &Range<usize>) -> Range<usize> {
    let start = line_start(text, selection.start);
    let mut end = selection.end;
    if selection.end > selection.start && end > start && end < text.len() && line_start(text, end) == end {
        end -= 1;
    }
    start..line_end(text, end)
}

/// The line spans the selections touch, in text order, with spans that overlap or sit on adjacent
/// lines merged into one block, each with the selections inside it.
pub(super) fn blocks(text: &str, selections: &[Range<usize>]) -> Vec<(Range<usize>, Vec<Range<usize>>)> {
    let mut sorted = selections.to_vec();
    sorted.sort_by_key(|s| s.start);
    let mut blocks: Vec<(Range<usize>, Vec<Range<usize>>)> = Vec::new();
    for selection in sorted {
        let span = line_span(text, &selection);
        match blocks.last_mut() {
            Some((last, inside)) if span.start <= last.end + 1 => {
                last.end = last.end.max(span.end);
                inside.push(selection);
            }
            _ => blocks.push((span, vec![selection])),
        }
    }
    blocks
}

/// Joins edits that do not overlap into one [`Edit`]. Each part places its selections as if it were
/// the only edit; here they move by the height every earlier part added or removed.
pub(super) fn compose(text: &str, mut parts: Vec<Edit>) -> Option<Edit> {
    parts.sort_by_key(|part| part.range.start);
    let range = parts.first()?.range.start..parts.last()?.range.end;
    let mut out = String::new();
    let mut at = range.start;
    let mut delta: isize = 0;
    let mut selections = Vec::new();
    for part in parts {
        out.push_str(&text[at..part.range.start]);
        out.push_str(&part.text);
        at = part.range.end;
        let moved = |offset: usize| offset.saturating_add_signed(delta);
        selections.extend(part.selections.iter().map(|s| moved(s.start)..moved(s.end)));
        delta += part.text.len() as isize - part.range.len() as isize;
    }
    Some(Edit { range, text: out, selections })
}

/// Where the first non-blank character of the line holding `offset` is.
pub(super) fn indent_end(text: &str, offset: usize) -> usize {
    let start = line_start(text, offset);
    let end = line_end(text, offset);
    start + text[start..end].len() - text[start..end].trim_start().len()
}

/// Home in Zed: the first press goes to the indent, the next to column 0, and back again. One caret
/// per selection.
pub fn smart_home(text: &str, selections: &[Range<usize>]) -> Vec<Range<usize>> {
    selections
        .iter()
        .map(|selection| {
            let caret = selection.end;
            let indent = indent_end(text, caret);
            let home = if caret == indent { line_start(text, caret) } else { indent };
            home..home
        })
        .collect()
}

/// Moves `offset` through edits in text order. Each edit is `(at, change)`: a positive change inserted
/// that many bytes at `at`, a negative one removed them from `at` on. An offset at an insertion point
/// moves past it, unless it `opens` a selection: then the insertion joins the selection. One inside a
/// removed run lands where the run began.
pub(super) fn shift(offset: usize, opens: bool, edits: &[(usize, isize)]) -> usize {
    let mut delta: isize = 0;
    for &(at, change) in edits {
        if change > 0 {
            if at < offset || (at == offset && !opens) {
                delta += change;
            }
        } else {
            let removed = change.unsigned_abs();
            if offset >= at + removed {
                delta += change;
            } else if offset > at {
                delta -= (offset - at) as isize;
            }
        }
    }
    offset.saturating_add_signed(delta)
}

/// The rows of a span, each with the offset it starts at.
pub(super) fn rows<'a>(text: &'a str, span: &Range<usize>) -> Vec<(usize, &'a str)> {
    let mut at = span.start;
    text[span.clone()]
        .split('\n')
        .map(|line| {
            let start = at;
            at += line.len() + 1;
            (start, line)
        })
        .collect()
}

/// Comments out the lines the selections touch, or uncomments them when every non-blank one is
/// already commented. The marker goes at each block's shallowest indent, as in Zed, so a block stays
/// aligned. Blank lines are left alone. `None` when there is nothing but blank lines.
pub fn toggle_comment(text: &str, selections: &[Range<usize>], prefix: &str) -> Option<Edit> {
    let blocks = blocks(text, selections);
    let filled: Vec<&str> = blocks
        .iter()
        .flat_map(|(span, _)| rows(text, span))
        .map(|(_, line)| line)
        .filter(|line| !line.trim().is_empty())
        .collect();
    if filled.is_empty() {
        return None;
    }
    let commented = filled.iter().all(|line| line.trim_start().starts_with(prefix));
    let parts = blocks
        .into_iter()
        .map(|(span, inside)| {
            let rows = rows(text, &span);
            let indent = rows
                .iter()
                .filter(|(_, line)| !line.trim().is_empty())
                .map(|(_, line)| line.len() - line.trim_start().len())
                .min()
                .unwrap_or(0);
            let mut out = Vec::with_capacity(rows.len());
            let mut edits: Vec<(usize, isize)> = Vec::new();
            for (start, line) in rows {
                if line.trim().is_empty() {
                    out.push(line.to_string());
                } else if commented {
                    let at = line.len() - line.trim_start().len();
                    let removed = prefix.len() + usize::from(line[at + prefix.len()..].starts_with(' '));
                    out.push(format!("{}{}", &line[..at], &line[at + removed..]));
                    edits.push((start + at, -(removed as isize)));
                } else {
                    out.push(format!("{}{prefix} {}", &line[..indent], &line[indent..]));
                    edits.push((start + indent, prefix.len() as isize + 1));
                }
            }
            let selections = inside
                .iter()
                .map(|s| shift(s.start, !s.is_empty(), &edits)..shift(s.end, false, &edits))
                .collect();
            Edit { range: span, text: out.join("\n"), selections }
        })
        .collect();
    compose(text, parts)
}

/// Swaps each block of lines the selections touch with the line above (`up`) or below. `None` when a
/// block sits at the top or the bottom, where it has nothing to swap with; Zed moves none then.
pub fn move_lines(text: &str, selections: &[Range<usize>], up: bool) -> Option<Edit> {
    let mut parts = Vec::new();
    for (span, inside) in blocks(text, selections) {
        let block = &text[span.clone()];
        let part = if up {
            if span.start == 0 {
                return None;
            }
            let above = line_start(text, span.start - 1);
            let moved = span.start - above;
            Edit {
                range: above..span.end,
                text: format!("{block}\n{}", &text[above..span.start - 1]),
                selections: inside.iter().map(|s| s.start - moved..s.end - moved).collect(),
            }
        } else {
            if span.end >= text.len() {
                return None;
            }
            let below = line_end(text, span.end + 1);
            let neighbour = &text[span.end + 1..below];
            let moved = neighbour.len() + 1;
            Edit {
                range: span.start..below,
                text: format!("{neighbour}\n{block}"),
                selections: inside.iter().map(|s| s.start + moved..s.end + moved).collect(),
            }
        };
        parts.push(part);
    }
    compose(text, parts)
}

/// Copies each block of lines the selections touch below itself, and moves its selections onto the
/// copy.
pub fn duplicate_lines(text: &str, selections: &[Range<usize>]) -> Option<Edit> {
    let parts = blocks(text, selections)
        .into_iter()
        .map(|(span, inside)| {
            let block = &text[span.clone()];
            let moved = block.len() + 1;
            Edit {
                range: span.end..span.end,
                text: format!("\n{block}"),
                selections: inside.iter().map(|s| s.start + moved..s.end + moved).collect(),
            }
        })
        .collect();
    compose(text, parts)
}

/// Removes the lines the selections touch, newline and all. Each block leaves one caret, at the first
/// selection's column on the line that takes the block's place, clamped to that line's length.
pub fn delete_lines(text: &str, selections: &[Range<usize>]) -> Option<Edit> {
    let parts = blocks(text, selections)
        .into_iter()
        .map(|(span, inside)| {
            let column = inside[0].start - line_start(text, inside[0].start);
            if span.end < text.len() {
                // The line below slides up to where the block began.
                let landing = span.end + 1;
                let caret = span.start + column.min(line_end(text, landing) - landing);
                Edit { range: span.start..landing, text: String::new(), selections: vec![caret..caret] }
            } else {
                // The block is the last line: take the newline before it, and land on the line above.
                let range = span.start.saturating_sub(1)..span.end;
                let above = line_start(text, range.start);
                let caret = above + column.min(range.start - above);
                Edit { range, text: String::new(), selections: vec![caret..caret] }
            }
        })
        .collect();
    compose(text, parts)
}

/// What copy and cut take when no selection has any text: every line a caret sits on, once each and
/// in order, each with its newline, as in Zed. The ranges are those lines in the buffer.
pub fn whole_lines(text: &str, carets: &[Range<usize>]) -> (Vec<Range<usize>>, String) {
    let mut ranges: Vec<Range<usize>> = Vec::new();
    let mut copied = String::new();
    let mut sorted = carets.to_vec();
    sorted.sort_by_key(|c| c.start);
    for caret in sorted {
        let start = line_start(text, caret.start);
        if ranges.last().is_some_and(|r| r.start == start) {
            continue;
        }
        let end = line_end(text, caret.start);
        ranges.push(if end < text.len() { start..end + 1 } else { start..end });
        copied.push_str(&text[start..end]);
        copied.push('\n');
    }
    (ranges, copied)
}

/// Where a whole line from the clipboard goes when pasted at bare carets: above each caret's line,
/// once per line, so every caret keeps its line and column, as in Zed.
pub fn whole_line_paste(text: &str, carets: &[Range<usize>], line: &str) -> Option<Edit> {
    let mut parts: Vec<Edit> = Vec::new();
    let mut sorted = carets.to_vec();
    sorted.sort_by_key(|c| c.start);
    for caret in sorted {
        let start = line_start(text, caret.start);
        let moved = caret.start + line.len();
        match parts.last_mut() {
            Some(part) if part.range.start == start => part.selections.push(moved..moved),
            _ => parts.push(Edit { range: start..start, text: line.to_string(), selections: vec![moved..moved] }),
        }
    }
    compose(text, parts)
}
