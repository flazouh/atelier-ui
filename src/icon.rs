//! Material Symbols Rounded (Apache 2.0), weight 400, unfilled, copied into `assets/icons` by
//! `tools/material.sh` from a pinned `@material-symbols/svg-400`. Never draw one by hand, and never use a
//! sparkle or magic-wand glyph for AI features.

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
    Add => "add",
    Mic => "mic",
    Archive => "archive",
    ArrowDownward => "arrow_downward",
    Public => "public",
    CreateNewFolder => "create_new_folder",
    FilterList => "filter_list",
    Unarchive => "unarchive",
    AddPhoto => "add_photo_alternate",
    ArrowBack => "arrow_back",
    ArrowDown => "arrow_downward",
    ArrowForward => "arrow_forward",
    ArrowOutward => "arrow_outward",
    ArrowUp => "arrow_upward",
    AttachFile => "attach_file",
    Backspace => "backspace",
    Block => "block",
    Bot => "smart_toy",
    Build => "build",
    Cancel => "cancel",
    Check => "check",
    Circle => "radio_button_unchecked",
    Warning => "warning",
    CheckCircle => "check_circle",
    Checklist => "checklist",
    ChevronDown => "keyboard_arrow_down",
    ChevronRight => "keyboard_arrow_right",
    ChevronUp => "keyboard_arrow_up",
    ChatBubble => "chat_bubble",
    Close => "close",
    CloseFullscreen => "close_fullscreen",
    Code => "code",
    Commit => "commit",
    Command => "keyboard_command_key",
    Control => "keyboard_control_key",
    Copy => "content_copy",
    Delete => "delete",
    Download => "download",
    DarkMode => "dark_mode",
    DataObject => "data_object",
    Edit => "edit",
    Description => "description",
    EditDocument => "edit_document",
    Error => "error",
    Extension => "extension",
    FormatBold => "format_bold",
    FormatItalic => "format_italic",
    FormatListBulleted => "format_list_bulleted",
    FormatQuote => "format_quote",
    Forum => "forum",
    File => "draft",
    Dns => "dns",
    Folder => "folder",
    Globe => "globe",
    LightMode => "light_mode",
    Lock => "lock",
    Link => "link",
    MoreHoriz => "more_horiz",
    Info => "info",
    Notifications => "notifications",
    NotificationsOff => "notifications_off",
    RotateRight => "rotate_right",
    OpenInFull => "open_in_full",
    OpenInNew => "open_in_new",
    Option => "keyboard_option_key",
    PrClosed => "do_not_disturb_on",
    PrDraft => "edit_note",
    PrMerged => "merge",
    PrOpen => "fork_right",
    PriorityHigh => "priority_high",
    Progress => "progress_activity",
    Refresh => "refresh",
    Return => "keyboard_return",
    Schedule => "schedule",
    Search => "search",
    Settings => "settings",
    Shift => "shift",
    Square => "check_box_outline_blank",
    Stop => "stop-fill",
    Tab => "keyboard_tab",
    Terminal => "terminal",
    ThumbDown => "thumb_down",
    ThumbUp => "thumb_up",
    UnfoldMore => "unfold_more",
    VerifiedUser => "verified_user",
    Visibility => "visibility",
    Dashboard => "dashboard",
    AudioFile => "audio_file",
    CloudUpload => "cloud_upload",
    Draft => "draft",
    FolderZip => "folder_zip",
    Image => "image",
    Movie => "movie",
    RotateLeft => "rotate_left",
    TableChart => "table_chart",
}

/// Serves the embedded icons to GPUI. An agent crate that ships strips for [`crate::sprite::Sprite`] serves
/// them from its own source and falls back to this one.
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(bytes) = icon_bytes(path).or_else(|| crate::file_icon::bytes(path)).or_else(|| crate::atelier_mark::bytes(path)) {
            return Ok(Some(Cow::Borrowed(bytes)));
        }
        // gpui-component draws icons of its own, such as the editor search bar's, and an app that
        // hands gpui this source replaces gpui-kit's. So what we do not ship comes from gpui-kit.
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
