/// `existing` with `words` after it, a space between when `existing` does not end in whitespace.
/// The box with `words` shown after `base`: `base` alone when there are none yet.
pub(super) fn live_text(base: &str, words: &str) -> String {
    if words.trim().is_empty() { base.to_string() } else { append_transcript(base, words) }
}

pub fn append_transcript(existing: &str, words: &str) -> String {
    let words = words.trim();
    if existing.is_empty() || existing.ends_with(char::is_whitespace) {
        format!("{existing}{words}")
    } else {
        format!("{existing} {words}")
    }
}
