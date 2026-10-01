//! The tabs of the single view as data: which are open in what order, which is active, what closing and
//! moving do, and how they group by project. The tab bar draws what this decides.
use gpui_kit::SharedString;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TabOrder {
    order: Vec<SharedString>,
    active: Option<SharedString>,
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
        let Some(at) = self.order.iter().position(|t| t == id) else { return self.active.as_ref() };
        self.order.remove(at);
        if self.active.as_deref() == Some(id) {
            self.active = self.order.get(at).or_else(|| self.order.get(at.wrapping_sub(1))).cloned();
        }
        self.active.as_ref()
    }

    /// Moves `id` to sit just before `target`, or to the end when `target` is `None`. Both must be open.
    pub fn move_before(&mut self, id: &str, target: Option<&str>) {
        let Some(from) = self.order.iter().position(|t| t == id) else { return };
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
    /// wrapping round. That is `order()` when the tabs are ungrouped and [`visual_order`] when grouped.
    pub fn cycle(&self, sequence: &[SharedString], forward: bool) -> Option<SharedString> {
        if sequence.is_empty() {
            return None;
        }
        let at = self.active.as_ref().and_then(|a| sequence.iter().position(|t| t == a));
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

/// The tabs in groups, the groups in the sidebar's order (`project_order`) and a project the sidebar does
/// not know after them, in the order its first tab was opened. Tabs keep their order inside a group.
pub fn grouped(order: &[SharedString], project_of: impl Fn(&SharedString) -> SharedString, project_order: &[SharedString]) -> Vec<TabGroup> {
    let mut groups: Vec<TabGroup> = Vec::new();
    for tab in order {
        let project = project_of(tab);
        match groups.iter_mut().find(|g| g.project == project) {
            Some(group) => group.tabs.push(tab.clone()),
            None => groups.push(TabGroup { project, tabs: vec![tab.clone()] }),
        }
    }
    groups.sort_by_key(|g| project_order.iter().position(|p| *p == g.project).unwrap_or(usize::MAX));
    groups
}

/// The tabs as the bar shows them, left to right.
pub fn visual_order(groups: &[TabGroup]) -> Vec<SharedString> {
    groups.iter().flat_map(|g| g.tabs.iter().cloned()).collect()
}

#[cfg(test)]
mod tests;
