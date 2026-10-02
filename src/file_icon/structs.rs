use gpui_kit::{App, IntoElement, Pixels, RenderOnce, SharedString, Styled, Window, img};

use crate::scale::px;
use super::helpers::{file_icon, folder_icon};

/// A file's or a folder's icon, in its own colours.
#[derive(IntoElement)]
pub struct FileIcon {
    pub(super) path: SharedString,
    size: Pixels,
}

impl FileIcon {
    /// The icon for the file at `path`.
    pub fn file(path: &str) -> Self {
        Self { path: file_icon(path), size: px(14.) }
    }

    /// The icon for a folder named `name`, open or closed.
    pub fn folder(name: &str, open: bool) -> Self {
        Self { path: folder_icon(name, open), size: px(14.) }
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for FileIcon {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        img(self.path).flex_none().size(self.size)
    }
}
