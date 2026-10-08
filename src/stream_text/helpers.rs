/// The byte where the tail begins: after the last blank line, unless that line is inside a code fence, where the
/// whole text is Markdown and the answer is `text.len()`.
pub fn split_tail(text: &str) -> usize {
    let mut fence = false;
    let mut last = 0;
    let mut at = 0;
    let mut blank_run = false;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fence = !fence;
        }
        at += line.len();
        if trimmed.is_empty() && !fence {
            blank_run = true;
        } else if blank_run {
            // `last` is where this first line of the next paragraph began.
            last = at - line.len();
            blank_run = false;
        }
    }
    if fence {
        return text.len();
    }
    if blank_run {
        return text.len();
    }
    last
}

/// Whether `tail` is text that reads the same with or without Markdown: one paragraph of words.
pub fn is_plain(tail: &str) -> bool {
    let first = tail.trim_start_matches(' ');
    if tail.starts_with("    ") || tail.contains('\n') {
        return false;
    }
    let starts_block = first.starts_with(['#', '>', '|'])
        || first.starts_with("- ")
        || first.starts_with("* ")
        || first.starts_with("+ ")
        || first.starts_with("```")
        || first.starts_with("~~~")
        || first.starts_with("---")
        || first.split_once(['.', ')']).is_some_and(|(n, rest)| {
            !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()) && rest.starts_with(' ')
        });
    !starts_block && !tail.contains(['`', '*', '_', '[', '<', '\\', '!', '~'])
}

/// CSS `ease-in-out`, the curve of Cursor's `fade-in-fast`.
pub(super) fn ease_in_out(t: f32) -> f32 {
    crate::motion::cubic_bezier(crate::motion::ease::STREAM_FADE, t.clamp(0., 1.))
}
