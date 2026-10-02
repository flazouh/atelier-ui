/// `existing` with `words` after it, a space between when `existing` does not end in whitespace.
pub fn append_transcript(existing: &str, words: &str) -> String {
    let words = words.trim();
    if existing.is_empty() || existing.ends_with(char::is_whitespace) {
        format!("{existing}{words}")
    } else {
        format!("{existing} {words}")
    }
}
