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

/// The most characters of one line of pasted text that go into the box as they are; more is a chip.
const MOST_INLINE_PASTE: usize = 200;

/// Whether pasted `text` is short enough to belong in the box as typed: one line (a trailing line break aside) of a
/// few words, a path, a link or a command. Anything longer or taller is something to hand over, not to read in place.
pub(super) fn is_inline_paste(text: &str) -> bool {
    let line = text.trim_end_matches(['\n', '\r']);
    !line.contains(['\n', '\r']) && line.chars().count() <= MOST_INLINE_PASTE
}
