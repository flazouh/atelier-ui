//! Hugeicons (MIT), the free stroke-rounded set on a 24 grid, copied into `assets/icons` by `tools/hugeicons.sh`
//! from a pinned `@iconify-json/hugeicons`. Every UI icon comes from it: never draw one by hand, never take one
//! from another set, and never use a sparkle or magic-wand glyph for AI features.

use std::{borrow::Cow, f32::consts::TAU};

use gpui_kit::{
    App, AssetSource, Hsla, IntoElement, Pixels, RenderOnce, Result, SharedString, Styled, Transformation, Window,
    prelude::FluentBuilder, radians, svg,
};
use crate::scale::px;

macro_rules! icons {
    ($($variant:ident => $file:literal),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum IconName { $($variant),* }

        impl IconName {
            pub const ALL: &[IconName] = &[$(Self::$variant),*];

            pub fn name(self) -> &'static str {
                match self { $(Self::$variant => $file),* }
            }

            fn path(self) -> &'static str {
                match self { $(Self::$variant => concat!("icons/", $file, ".svg")),* }
            }
        }

        fn icon_bytes(path: &str) -> Option<&'static [u8]> {
            match path {
                $(concat!("icons/", $file, ".svg") => Some(include_bytes!(concat!("../assets/icons/", $file, ".svg")))),*,
                _ => None,
            }
        }
    };
}

icons! {
    Add => "plus-sign",
    Star => "star",
    StarFilled => "star-fill",
    Grip => "drag-drop-vertical",
    Mic => "mic-01",
    Archive => "archive-arrow-down",
    ArrowDownward => "arrow-down-02",
    Public => "earth",
    CreateNewFolder => "folder-add",
    FilterList => "list-filter",
    Unarchive => "archive-arrow-up",
    AddPhoto => "image-add-02",
    ArrowBack => "arrow-left-02",
    ArrowDown => "arrow-down-02",
    ArrowForward => "arrow-right-02",
    ArrowOutward => "arrow-up-right-01",
    ArrowUp => "arrow-up-02",
    AttachFile => "attachment-01",
    Backspace => "eraser-01",
    Block => "unavailable",
    Bot => "bot",
    Build => "wrench-01",
    Cancel => "cancel-circle",
    Check => "tick-02",
    Circle => "circle",
    Warning => "alert-02",
    CheckCircle => "checkmark-circle-02",
    Checklist => "check-list",
    ChevronDown => "arrow-down-01",
    ChevronRight => "arrow-right-01",
    ChevronUp => "arrow-up-01",
    ChatBubble => "bubble-chat",
    Close => "cancel-01",
    CloseFullscreen => "arrow-shrink-01",
    Code => "source-code",
    Commit => "git-commit",
    GitBranch => "git-branch",
    Command => "command",
    Control => "arrow-up-01",
    Copy => "copy-01",
    Delete => "delete-02",
    Download => "download-01",
    DarkMode => "moon-02",
    DataObject => "code-square",
    Edit => "pencil-edit-01",
    Description => "file-02",
    EditDocument => "file-edit",
    Error => "alert-circle",
    Extension => "puzzle",
    FormatBold => "text-bold",
    FormatItalic => "text-italic",
    FormatListBulleted => "left-to-right-list-bullet",
    FormatQuote => "quote-down",
    Forum => "message-multiple-02",
    File => "file-empty-02",
    Dns => "server-stack-01",
    Folder => "folder-01",
    Globe => "globe-02",
    LightMode => "sun-03",
    Lock => "square-lock-02",
    Link => "link-01",
    MoreHoriz => "more-horizontal",
    Info => "information-circle",
    Help => "help-circle",
    Idea => "idea-01",
    Flash => "flash",
    Bug => "bug-01",
    Exchange => "exchange-01",
    Brush => "paint-brush-01",
    Notifications => "notification-01",
    NotificationsOff => "notification-off-01",
    RotateRight => "rotate-clockwise",
    OpenInFull => "arrow-expand-01",
    OpenInNew => "link-square-02",
    Option => "option",
    PrClosed => "git-pull-request-closed",
    PrDraft => "git-pull-request-draft",
    PrMerged => "git-merge",
    PrOpen => "git-pull-request",
    PriorityHigh => "exclamation-mark",
    Progress => "loading-03",
    Refresh => "refresh",
    Return => "arrow-turn-backward",
    Schedule => "clock-01",
    Search => "search-01",
    Settings => "settings-01",
    Shift => "arrow-up-big",
    Square => "square",
    Stop => "stop-fill",
    Tab => "arrow-right-to-line",
    Terminal => "command-line",
    ThumbDown => "thumbs-down",
    ThumbUp => "thumbs-up",
    UnfoldMore => "unfold-more",
    VerifiedUser => "security-check",
    Visibility => "view",
    Dashboard => "dashboard-square-01",
    AudioFile => "file-audio",
    CloudUpload => "cloud-upload",
    Draft => "file-empty-02",
    FolderZip => "folder-zip",
    Image => "image-01",
    Movie => "film-01",
    RotateLeft => "rotate-ccw",
    TableChart => "layout-table-01",
    ChevronLeft => "arrow-left-01",
    Minus => "minus-sign",
    VisibilityOff => "view-off-slash",
    Replace => "replace",
    CaseSensitive => "case-sensitive",
    Asterisk => "asterisk",
    SidebarLeft => "sidebar-left",
    Mail => "mail-01",
}

/// The icons gpui-component draws for the parts atelier uses (the editor's search bar, inputs, notifications, the copy
/// button), by the path it asks for, and ours in their place. Without them gpui-kit serves its own, from another set.
const COMPONENT_ICONS: &[(&str, IconName)] = &[
    ("icons/asterisk.svg", IconName::Asterisk),
    ("icons/case-sensitive.svg", IconName::CaseSensitive),
    ("icons/chevron-down.svg", IconName::ChevronDown),
    ("icons/chevron-left.svg", IconName::ChevronLeft),
    ("icons/chevron-right.svg", IconName::ChevronRight),
    ("icons/chevron-up.svg", IconName::ChevronUp),
    ("icons/circle-check.svg", IconName::CheckCircle),
    ("icons/circle-x.svg", IconName::Cancel),
    ("icons/close.svg", IconName::Close),
    ("icons/copy.svg", IconName::Copy),
    ("icons/eye.svg", IconName::Visibility),
    ("icons/eye-off.svg", IconName::VisibilityOff),
    ("icons/info.svg", IconName::Info),
    ("icons/minus.svg", IconName::Minus),
    ("icons/plus.svg", IconName::Add),
    ("icons/replace.svg", IconName::Replace),
    ("icons/search.svg", IconName::Search),
    ("icons/triangle-alert.svg", IconName::Warning),
];

/// Ours for a path gpui-component asks for, if we have one.
fn component_icon(path: &str) -> Option<&'static [u8]> {
    COMPONENT_ICONS.iter().find(|(asked, _)| *asked == path).and_then(|(_, ours)| icon_bytes(ours.path()))
}

/// Serves the embedded icons to GPUI. An agent crate that ships strips for [`crate::sprite::Sprite`] serves
/// them from its own source and falls back to this one.
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(bytes) = icon_bytes(path)
            .or_else(|| component_icon(path))
            .or_else(|| crate::file_icon::bytes(path))
            .or_else(|| crate::atelier_mark::AtelierMark::bytes(path))
            .or_else(|| crate::release_sheet::bytes(path))
        {
            return Ok(Some(Cow::Borrowed(bytes)));
        }
        // gpui-component draws icons of its own, and an app that hands gpui this source replaces gpui-kit's. The ones it
        // draws in the parts atelier uses are ours (above); any other still comes from gpui-kit, so nothing draws empty.
        Ok(gpui_kit::assets::Assets::new("").load(path).ok().flatten())
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}

#[derive(IntoElement)]
pub struct Icon {
    name: IconName,
    size: Pixels,
    color: Option<Hsla>,
    turn: f32,
}

impl Icon {
    pub fn new(name: IconName) -> Self {
        Self { name, size: px(16.), color: None, turn: 0. }
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    /// Rotates the icon by `turn` full turns, for a spinner.
    pub fn turn(mut self, turn: f32) -> Self {
        self.turn = turn;
        self
    }

    /// Defaults to the surrounding text color.
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let color = self.color.unwrap_or(window.text_style().color);
        svg()
            .path(self.name.path())
            .flex_none()
            .size(self.size)
            .text_color(color)
            .when(self.turn != 0., |s| s.with_transformation(Transformation::rotate(radians(self.turn * TAU))))
    }
}

#[cfg(test)]
mod tests;
