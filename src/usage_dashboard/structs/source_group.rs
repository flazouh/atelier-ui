use gpui_kit::SharedString;

/// Sources that stand together under one caption: a provider's accounts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceGroup {
    pub caption: SharedString,
    pub first: usize,
    pub len: usize,
}
