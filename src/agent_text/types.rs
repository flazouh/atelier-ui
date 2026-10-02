#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AgentTextStatus {
    /// Tokens are still arriving. Hides the completion row entirely.
    #[default]
    Streaming,
    Complete,
    /// Shows Copy, Retry, and sources.
    Error,
}
