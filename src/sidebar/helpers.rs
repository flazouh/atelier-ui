use std::rc::Rc;

use gpui_kit::SharedString;

use crate::icon::IconName;
use crate::menu::{Branch, Entry, MenuItem, Pick, entries_of};
use super::types::{HANDOFF, SidebarEvent};

/// An element id from a word and the id of a project or a session.
pub(super) fn name(prefix: &str, id: &str) -> gpui_kit::ElementId {
    gpui_kit::ElementId::Name(format!("{prefix}-{id}").into())
}

/// The "Handoff" row of a session's menu: a menu of the project's targets, or a dimmed row when it has none.
pub(super) fn handoff_entry(sidebar: &gpui_kit::Entity<super::structs::Sidebar>, project: &SharedString, session: &SharedString, targets: Option<&Vec<Branch>>) -> Entry {
    let row = MenuItem::new(HANDOFF).icon(IconName::Replace).debug_name("session-menu-handoff");
    let Some(targets) = targets.filter(|targets| !targets.is_empty()) else { return row.disabled(true).into() };
    let (sidebar, project, session) = (sidebar.clone(), project.clone(), session.clone());
    // A choice shuts the session menu, then asks the app for the handoff.
    let pick: Pick = Rc::new(move |target, _, cx| {
        let event = SidebarEvent::Handoff { project: project.clone(), session: session.clone(), target: target.clone() };
        sidebar.update(cx, |s, cx| {
            s.session_menu = None;
            cx.emit(event);
            cx.notify();
        })
    });
    row.submenu(entries_of(targets, &pick)).into()
}
