use gpui_kit::SharedString;

/// One replacement: the text that was there, and the text that goes in its place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextEdit {
    pub old: SharedString,
    pub new: SharedString,
}

impl TextEdit {
    pub fn new(old: impl Into<SharedString>, new: impl Into<SharedString>) -> Self {
        Self {
            old: old.into(),
            new: new.into(),
        }
    }
}
