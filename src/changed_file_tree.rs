//! The changed files as a tree, left of the diff, for the agent's turn and the pull request view alike.
//!
//! Folders show their summed `+a −r` and fold; a folder that holds only one folder merges with it into
//! one row ("crates/beui/src"). Files show a file icon, the name, `+a −r`, and a check once reviewed.
//! The current file has the accent's selection wash. Pressing a file reports it; pressing a folder folds
//! it.
//!
//! Keyboard, while the tree has focus: up and down move the keyboard row, left folds a folder or goes to
//! its parent, right opens a folder or goes to its first child, enter opens a file or folds a folder.
//! Bare `j` and `k` move between files through the owner's [`crate::review::ReviewHandlers`]. Folding
//! is instant: it happens tens of times a session, and the rows it hides are one keystroke away.

gpui_kit::actions!(
    changed_file_tree,
    [
        /// Moves the keyboard row up.
        SelectUp,
        /// Moves the keyboard row down.
        SelectDown,
        /// Folds the folder, or goes to the parent folder.
        FoldRow,
        /// Opens the folder, or goes to its first child.
        UnfoldRow,
        /// Opens the file, or folds and opens the folder.
        OpenRow,
    ]
);

mod helpers;
mod structs;
mod types;

pub use helpers::unfolds;
pub(crate) use helpers::bind_keys;
#[cfg(test)]
pub(crate) use helpers::key;
pub use structs::ChangedFileTree;
#[cfg(test)]
pub(crate) use types::TreeKey;

#[cfg(test)]
use std::collections::HashSet;
#[cfg(test)]
use gpui_kit::SharedString;
#[cfg(test)]
use crate::{changed_files::ChangedFile, file_tree::FileTree};

#[cfg(test)]
mod tests;
