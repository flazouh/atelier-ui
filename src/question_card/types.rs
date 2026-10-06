use gpui_kit::SharedString;

/// One question as the card shows it. `options` are a label and what it means; the card adds the "other" line itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestionView {
    /// The short word that names the question, such as "Crates".
    pub header: SharedString,
    pub question: SharedString,
    pub options: Vec<(SharedString, SharedString)>,
    /// The reader may pick more than one.
    pub multiple: bool,
}

/// Where the question stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestionStatus {
    /// The agent is still writing it: what has come is shown and nothing can be picked.
    Streaming,
    /// It waits for the reader.
    Pending,
    /// The reader answered.
    Answered,
}
