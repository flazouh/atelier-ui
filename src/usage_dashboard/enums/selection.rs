use gpui_kit::SharedString;

/// What the dashboard shows the data of: everything, one provider's group of accounts, or one account or key.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Selection {
    #[default]
    All,
    /// The `group` caption of a provider's sources.
    Group(SharedString),
    /// The `id` of a source.
    Source(SharedString),
}
