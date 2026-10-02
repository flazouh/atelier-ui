use crate::fuzzy;
use super::structs::CommandItem;
use super::types::Trigger;

/// What the text asks for with the caret at byte `cursor`, if anything.
pub fn trigger(text: &str, cursor: usize) -> Option<Trigger> {
    let before = text.get(..cursor)?;
    if let Some(query) = before.strip_prefix('/')
        && !query.contains(char::is_whitespace)
    {
        return Some(Trigger::Command { query: query.to_string() });
    }
    let start = before.rfind('@')?;
    let query = &before[start + 1..];
    let at_word_start = before[..start].chars().next_back().is_none_or(char::is_whitespace);
    (at_word_start && !query.contains(char::is_whitespace)).then(|| Trigger::Mention { start, query: query.to_string() })
}

/// The text with the mention from `start` to `cursor` replaced by `@path` and a space; and the caret after it.
pub fn mention(text: &str, start: usize, cursor: usize, path: &str) -> (String, usize) {
    let inserted = format!("@{path} ");
    let caret = start + inserted.len();
    (format!("{}{inserted}{}", &text[..start], &text[cursor..]), caret)
}

/// The indices of `items` that match `query`, best first; every one, in order, for no query.
pub fn ranked(query: &str, items: &[CommandItem]) -> Vec<usize> {
    if query.is_empty() {
        return (0..items.len()).collect();
    }
    fuzzy::rank(query, items.iter().map(|i| i.name.as_ref()), 50)
}
