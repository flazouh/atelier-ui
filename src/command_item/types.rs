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

/// What the text asks for at the caret.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Trigger {
    /// A `/` starts the text and the caret is still in the name.
    Command { query: String },
    /// An `@` at a word's start, and the caret after it with no space between; `start` is the `@`'s byte.
    Mention { start: usize, query: String },
}
