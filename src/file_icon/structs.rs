use gpui_kit::{App, Global, InteractiveElement, IntoElement, Pixels, RenderOnce, SharedString, Styled, Window, img};

use crate::scale::px;
use super::helpers::{file_icon, folder_icon, strip_location};
use super::types::{IconFor, Source};

pub(super) struct OwnSource(pub(super) Source);

impl Global for OwnSource {}

/// A file's or a folder's icon, in its own colours.
#[derive(IntoElement)]
pub struct FileIcon {
    pub(super) path: SharedString,
    pub(super) folder: Option<bool>,
    pub(super) asset: SharedString,
    size: Pixels,
}

impl FileIcon {
    /// The icon for the file at `path`.
    pub fn file(path: &str) -> Self {
        Self { path: strip_location(path).to_owned().into(), folder: None, asset: file_icon(path), size: px(14.) }
    }

    /// The icon for a folder named `name`, open or closed.
    pub fn folder(name: &str, open: bool) -> Self {
        Self { path: name.to_owned().into(), folder: Some(open), asset: folder_icon(name, open), size: px(14.) }
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for FileIcon {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let icon = match self.folder {
            None => IconFor::File(&self.path),
            Some(open) => IconFor::Folder { name: &self.path, open },
        };
        if let Some(own) = cx.try_global::<OwnSource>().and_then(|source| (source.0)(&icon, self.size, cx)) {
            return own;
        }
        img(self.asset).debug_selector(|| "file-icon".into()).flex_none().size(self.size).into_any_element()
    }
}
