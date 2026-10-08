use gpui_kit::SharedString;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TabOrder {
    pub(super) order: Vec<SharedString>,
    pub(super) active: Option<SharedString>,
}

impl TabOrder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn order(&self) -> &[SharedString] {
        &self.order
    }

    pub fn active(&self) -> Option<&SharedString> {
        self.active.as_ref()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.order.iter().any(|t| t == id)
    }

    /// Opens a tab at the end and makes it active. A tab already open is only made active.
    pub fn open(&mut self, id: SharedString) {
        if !self.contains(&id) {
            self.order.push(id.clone());
        }
        self.active = Some(id);
    }

    /// Makes an open tab active. Nothing else changes.
    pub fn activate(&mut self, id: &str) {
        if let Some(tab) = self.order.iter().find(|t| *t == id) {
            self.active = Some(tab.clone());
        }
    }

    /// Closes a tab. When it was the active one, the tab that was next to it on the right takes its
    /// place, else the one on the left. Gives the active tab after.
    pub fn close(&mut self, id: &str) -> Option<&SharedString> {
        let Some(at) = self.order.iter().position(|t| t == id) else {
            return self.active.as_ref();
        };
        self.order.remove(at);
        if self.active.as_deref() == Some(id) {
            self.active = self
                .order
                .get(at)
                .or_else(|| self.order.get(at.wrapping_sub(1)))
                .cloned();
        }
        self.active.as_ref()
    }

    /// Moves `id` to sit just before `target`, or to the end when `target` is `None`. Both must be open.
    pub fn move_before(&mut self, id: &str, target: Option<&str>) {
        let Some(from) = self.order.iter().position(|t| t == id) else {
            return;
        };
        let tab = self.order.remove(from);
        let to = match target {
            Some(target) => match self.order.iter().position(|t| t == target) {
                Some(to) => to,
                None => {
                    self.order.insert(from, tab);
                    return;
                }
            },
            None => self.order.len(),
        };
        self.order.insert(to, tab);
    }

    /// The tab after (or before) the active one in `sequence`, which is the order the tabs are shown in,
    /// wrapping round. That is `order()` when the tabs are ungrouped and [`visual_order`](crate::tab_order::visual_order) when grouped.
    pub fn cycle(&self, sequence: &[SharedString], forward: bool) -> Option<SharedString> {
        if sequence.is_empty() {
            return None;
        }
        let at = self
            .active
            .as_ref()
            .and_then(|a| sequence.iter().position(|t| t == a));
        let next = match (at, forward) {
            (None, true) => 0,
            (None, false) => sequence.len() - 1,
            (Some(i), true) => (i + 1) % sequence.len(),
            (Some(i), false) => (i + sequence.len() - 1) % sequence.len(),
        };
        Some(sequence[next].clone())
    }
}

/// Tabs of one project, with a separator and the project's name before them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabGroup {
    pub project: SharedString,
    pub tabs: Vec<SharedString>,
}
