//! What a tool approval shows the reader instead of the tool's raw input: an edit as a diff under its path, a file
//! written as all-new lines, a shell call as its command. It takes plain data, so the app maps a tool call to
//! one of these and atelier-ui knows nothing of any agent. The raw input stays behind "View details".

mod helpers;
mod structs;
mod types;

pub use helpers::{line_diff, relative_path};
pub use structs::TextEdit;
pub use types::ToolPreview;

#[cfg(test)]
use crate::file_diff::{DiffLine, DiffLineKind};

#[cfg(test)]
mod tests;
