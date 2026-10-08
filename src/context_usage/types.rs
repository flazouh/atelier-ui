use gpui_kit::SharedString;

/// The width of the panel.
pub const WIDTH: f32 = 320.;

/// The height of the bar.
pub(super) const BAR: f32 = 6.;

/// The space between two parts of the bar.
pub(super) const SEAM: f32 = 2.;

/// The size of the square before a part's name.
pub(super) const SWATCH: f32 = 8.;

/// The rows shown when the agent could not tell the parts: what is in use, and what is left of the window.
pub(super) const WHOLE: &str = "In context";
pub(super) const FREE: &str = "Free";

/// One part of what fills the window: a name and the tokens it holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextPart {
    pub label: SharedString,
    pub tokens: u64,
}

impl ContextPart {
    pub fn new(label: impl Into<SharedString>, tokens: u64) -> Self {
        Self {
            label: label.into(),
            tokens,
        }
    }
}
