use std::rc::Rc;

use gpui_kit::{App, SharedString, Window};

/// A callback that receives a file's path.
pub(crate) type PathHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// How many rows show before the rest fold away.
pub const FOLD_AT: usize = 5;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum FileChange {
    #[default]
    Modified,
    Added,
    Deleted,
    Renamed {
        from: SharedString,
    },
}

impl FileChange {
    /// The word after the path. A modified file needs none.
    pub fn word(&self) -> Option<&'static str> {
        match self {
            Self::Modified => None,
            Self::Added => Some("Added"),
            Self::Deleted => Some("Deleted"),
            Self::Renamed { .. } => Some("Renamed"),
        }
    }
}
