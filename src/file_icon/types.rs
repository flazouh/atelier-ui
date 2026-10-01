use gpui_kit::{AnyElement, App, Pixels};

pub(super) const FALLBACK: &str = "file";

/// What a [`FileIcon`](super::FileIcon) stands for, as an icon source sees it.
pub enum IconFor<'a> {
    /// A file at this path, a trailing `:line` removed.
    File(&'a str),
    /// A folder with this name, open or closed.
    Folder { name: &'a str, open: bool },
}

/// Draws an icon at a size, or `None` to keep the built-in one.
pub type Source = fn(&IconFor, Pixels, &App) -> Option<AnyElement>;
