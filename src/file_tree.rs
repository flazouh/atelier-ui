//! The changed files as a tree, for [`crate::changed_file_tree::ChangedFileTree`]: folders first, then
//! files, each by name, as VS Code and GitQuiet list them. A folder that holds only one folder merges
//! with it into one row ("crates/beui/src"), and every folder sums the lines its files add and remove.
//! Pure, so the whole shape is tested without a window.

mod helpers;
mod structs;
mod types;

pub use structs::{FileTree, TreeRow};

#[cfg(test)]
use gpui_kit::SharedString;

#[cfg(test)]
mod tests;
