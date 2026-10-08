use std::rc::Rc;

use gpui_kit::{App, Window};

use crate::icon::IconName;
use crate::menu::Origin;

pub(super) type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

pub(super) type Chooser = Rc<dyn Fn(MenuChoice, &mut Window, &mut App)>;

/// Where the menu unfolds from: the panel hangs from the `⋯` button at its right, so it grows out of its top-right
/// corner, under the button the reader pressed, and not from the far left.
pub const MENU_ORIGIN: Origin = Origin::TopRight;

/// What the `⋯` menu offers, in order.
pub const MENU: [&str; 7] = [
    "Pull requests",
    "Tasks",
    "Worktrees",
    "Choose an icon…",
    "Close project",
    "Files",
    "Copy path",
];

/// Which of the menu's entries a press chose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuChoice {
    PullRequests,
    Tasks,
    Worktrees,
    ChooseIcon,
    Close,
    Files,
    CopyPath,
}

impl MenuChoice {
    pub const ALL: [MenuChoice; 7] = [
        Self::PullRequests,
        Self::Tasks,
        Self::Worktrees,
        Self::ChooseIcon,
        Self::Close,
        Self::Files,
        Self::CopyPath,
    ];

    /// The icon before the row's words.
    pub fn icon(self) -> IconName {
        match self {
            Self::PullRequests => IconName::PrOpen,
            Self::Tasks => IconName::Checklist,
            Self::Worktrees => IconName::GitBranch,
            Self::ChooseIcon => IconName::AddPhoto,
            Self::Close => IconName::Close,
            Self::Files => IconName::Folder,
            Self::CopyPath => IconName::Copy,
        }
    }

    /// The key that reaches the same thing from anywhere, shown on the menu row.
    pub fn cap(self) -> Option<&'static str> {
        match self {
            Self::PullRequests => Some("⌘⇧p"),
            Self::Tasks => Some("⌘⇧l"),
            Self::Worktrees | Self::ChooseIcon | Self::Close | Self::Files | Self::CopyPath => None,
        }
    }

    pub fn words(self) -> &'static str {
        MENU[Self::ALL.iter().position(|c| *c == self).unwrap_or(0)]
    }
}
