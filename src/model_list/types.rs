use gpui_kit::SharedString;

/// One model of the list: the id the agent knows it by, the name the reader sees, and a line of what it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelRow {
    pub id: SharedString,
    pub label: SharedString,
    pub detail: Option<SharedString>,
    /// Left out of the picker: the row stays in the list, dimmed, to be shown again.
    pub hidden: bool,
}

impl ModelRow {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self { id: id.into(), label: label.into(), detail: None, hidden: false }
    }

    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// A row being dragged: which list it came from, and which row it is.
#[derive(Clone)]
pub(super) struct Dragged {
    pub list: SharedString,
    pub id: SharedString,
    pub label: SharedString,
}
