use gpui_kit::SharedString;

use crate::subagent_row::{done_text, tool_calls_text};

/// The body line's segments: the count while it runs, then "Done in 38s" and the count. They part by
/// space, never by a glyph. `finished` is `Some` once done, holding the run time in seconds if known.
pub fn status_line(finished: Option<Option<u64>>, tool_calls: u64) -> Vec<SharedString> {
    match finished {
        None => vec![tool_calls_text(tool_calls)],
        Some(seconds) => vec![done_text(seconds), tool_calls_text(tool_calls)],
    }
}
