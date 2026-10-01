//! A line break inside a paragraph stays a line break. CommonMark joins the lines of a paragraph with a space,
//! so a poem, an address or a list of short lines an agent writes would run together. Claude's own apps keep
//! the break; this puts a hard break (two spaces) at the end of each such line before the text is drawn.
//!
//! Left alone: a fenced code block (its lines are code), a table row, a line that already ends a hard
//! break, and the line before a blank line or the end of the text.

/// `markdown` with each line break inside a paragraph made a hard break.
pub fn keep(markdown: &str) -> String {
    let lines: Vec<&str> = markdown.split('\n').collect();
    let mut out = String::with_capacity(markdown.len() + 16);
    let mut fence: Option<(char, usize)> = None;
    for (i, line) in lines.iter().enumerate() {
        out.push_str(line);
        let trimmed = line.trim_start();
        if let Some((mark, len)) = fence {
            if closes(trimmed, mark, len) {
                fence = None;
            }
        } else if let Some(opening) = opens(trimmed) {
            fence = Some(opening);
        } else if let Some(next) = lines.get(i + 1)
            && !trimmed.is_empty()
            && !next.trim().is_empty()
            && !line.ends_with("  ")
            && !line.ends_with('\\')
            && !trimmed.starts_with('|')
            && !next.trim_start().starts_with('|')
            && opens(next.trim_start()).is_none()
        {
            out.push_str("  ");
        }
        if i + 1 < lines.len() {
            out.push('\n');
        }
    }
    out
}

/// The fence character and its length when `line` opens a fenced block.
fn opens(line: &str) -> Option<(char, usize)> {
    let mark = line.chars().next().filter(|c| *c == '`' || *c == '~')?;
    let len = line.chars().take_while(|c| *c == mark).count();
    (len >= 3).then_some((mark, len))
}

/// Whether `line` closes a block opened with `len` of `mark`.
fn closes(line: &str, mark: char, len: usize) -> bool {
    let run = line.chars().take_while(|c| *c == mark).count();
    run >= len && line[run * mark.len_utf8()..].trim().is_empty()
}

#[cfg(test)]
mod tests;
