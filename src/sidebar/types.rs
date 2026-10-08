use gpui_kit::SharedString;

use crate::sidebar_layout::SidebarLayout;

/// The words of the row in a session's menu that carries it on elsewhere.
pub(super) const HANDOFF: &str = "Handoff";

/// How long a new session's row is wrapped in its entrance.
pub(super) const ENTERING: std::time::Duration = std::time::Duration::from_millis(500);

/// What the sidebar asks of the app. Ids are the ones in [`ProjectData`](crate::sidebar_model::ProjectData) and `sidebar_model::SessionData`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SidebarEvent {
    Open {
        project: SharedString,
        session: SharedString,
    },
    /// The reader pressed a row's archive button: the session goes into the archive, or comes out of it.
    Archive {
        project: SharedString,
        session: SharedString,
        archive: bool,
    },
    /// The reader chose "Copy session id" in a row's menu.
    CopySessionId {
        project: SharedString,
        session: SharedString,
    },
    /// The reader chose a target in a row's Handoff menu: a new session carries this one on, on that agent and
    /// provider. `target` is the id of the leaf of the project's [`Branch`](crate::menu::Branch) tree.
    Handoff {
        project: SharedString,
        session: SharedString,
        target: SharedString,
    },
    /// The reader chose "Close panel" in a row's menu: the session has a panel open, and it closes.
    CloseSession {
        project: SharedString,
        session: SharedString,
    },
    /// The reader changed the head's choices (the list mode or the filter): the app may keep them.
    LayoutChanged(SidebarLayout),
    /// The reader chose "Open folder…" from the add menu.
    AddFolder,
    /// The reader chose "Open over SSH…" from the add menu.
    AddRemote,
    NewSession {
        project: SharedString,
    },
    Retry {
        project: SharedString,
    },
    CloseProject {
        project: SharedString,
    },
    OpenFiles {
        project: SharedString,
    },
    /// The project's pull requests.
    PullRequests {
        project: SharedString,
    },
    /// The project's tasks.
    Tasks {
        project: SharedString,
    },
    /// The project's worktrees, in the Git view.
    Worktrees {
        project: SharedString,
    },
    CopyPath {
        project: SharedString,
    },
    /// The reader wants to pick an image file from the project as its badge.
    ChooseIcon {
        project: SharedString,
    },
}
