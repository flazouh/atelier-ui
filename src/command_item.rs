//! What the composer offers after `/` (commands) and `@` (files), as neutral data, and the rules for when
//! each list opens and what a pick writes. The composer ([`crate::prompt_input::PromptInput`]) draws the
//! list; the app fills it (the agent's commands, atelier's, the project's skills and files) and runs a pick.

use gpui_kit::SharedString;

use crate::fuzzy;

/// Where a command comes from, for its row's words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandSource {
    Agent,
    Atelier,
    Skill,
}

impl CommandSource {
    pub fn words(self) -> &'static str {
        match self {
            Self::Agent => "Agent",
            Self::Atelier => "atelier",
            Self::Skill => "Skill",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandItem {
    /// Without the slash.
    pub name: SharedString,
    pub source: CommandSource,
    pub summary: SharedString,
    /// What goes after the name. A command with one keeps the list closed and waits for its words.
    pub args_hint: Option<SharedString>,
}

/// What the text asks for at the caret.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Trigger {
    /// A `/` starts the text and the caret is still in the name.
    Command { query: String },
    /// An `@` at a word's start, and the caret after it with no space between; `start` is the `@`'s byte.
    Mention { start: usize, query: String },
}

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

#[cfg(test)]
mod tests;
