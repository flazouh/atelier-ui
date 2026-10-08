use gpui_kit::SharedString;

use super::structs::TabGroup;

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
