use gpui_kit::SharedString;

use crate::subagent_row::done_text;

/// The left side of the body line: "Done in 38s" once the card is done, else the live tool call, if any.
/// `finished` is `Some` once done, holding the run time in seconds if known. The count sits apart, on the
/// right.
pub fn lead_text(finished: Option<Option<u64>>, live_tool: Option<SharedString>) -> Option<SharedString> {
    match finished {
        Some(seconds) => Some(done_text(seconds)),
        None => live_tool,
    }
}
