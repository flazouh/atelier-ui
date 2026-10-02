#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Bold,
    Italic,
    Code,
    Link,
    Quote,
    List,
}

impl Format {
    pub const ALL: [Format; 6] = [Self::Bold, Self::Italic, Self::Code, Self::Link, Self::Quote, Self::List];

    /// The tooltip, which names the key where one exists (GitQuiet's ⌘B, ⌘I, ⌘E, ⌘K).
    pub fn word(self) -> &'static str {
        match self {
            Self::Bold => "Bold",
            Self::Italic => "Italic",
            Self::Code => "Code",
            Self::Link => "Link",
            Self::Quote => "Quote",
            Self::List => "List",
        }
    }
}
