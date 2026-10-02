use gpui_kit::SharedString;

/// "1 tool call", "12 tool calls".
pub fn tool_calls_text(count: u64) -> SharedString {
    if count == 1 { "1 tool call".into() } else { format!("{count} tool calls").into() }
}

/// "Done in 38s", or "Done" when the run time is not known.
pub fn done_text(seconds: Option<u64>) -> SharedString {
    match seconds {
        Some(seconds) => format!("Done in {seconds}s").into(),
        None => "Done".into(),
    }
}
