use std::path::PathBuf;

use gpui_kit::{ElementId, InteractiveElement, IntoElement, MouseButton, ParentElement, Styled, div};

use crate::scale::px;
use crate::{
    icon::{Icon, IconName},
    theme::{Theme},
};
use super::structs::UploadItem;
use super::types::UploadStatus;

/// The progress shown: 100 for a file that arrived, 0 for one with no number, else the number within 0 to 100.
pub fn clamp_progress(progress: f32, status: UploadStatus) -> f32 {
    if status == UploadStatus::Success {
        100.
    } else if progress.is_nan() {
        0.
    } else {
        progress.clamp(0., 100.)
    }
}

/// A size in the fewest digits: "0 B", "512 B", "1.5 KB", "18 MB".
pub fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".into();
    }
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let exponent = ((bytes as f64).ln() / 1024f64.ln()).floor().clamp(0., 4.) as usize;
    let value = bytes as f64 / 1024f64.powi(exponent as i32);
    if value >= 10. || exponent == 0 { format!("{value:.0} {}", UNITS[exponent]) } else { format!("{value:.1} {}", UNITS[exponent]) }
}

/// The kind, in capitals: the extension, else the media type's subtype, else "FILE".
pub fn kind_of(item: &UploadItem) -> String {
    if let Some((_, extension)) = item.name.rsplit_once('.').filter(|(_, ext)| !ext.is_empty()) {
        return extension.to_uppercase();
    }
    item.mime.as_deref().and_then(|m| m.rsplit('/').next()).filter(|s| !s.is_empty()).map_or_else(|| "FILE".into(), str::to_uppercase)
}

/// The icon for a file: by media type, else by extension.
pub fn icon_of(item: &UploadItem) -> IconName {
    let extension = item.name.rsplit_once('.').map(|(_, e)| e.to_lowercase()).unwrap_or_default();
    let mime = item.mime.as_deref().unwrap_or("");
    let has = |list: &[&str]| list.contains(&extension.as_str());
    if mime.starts_with("image/") {
        IconName::Image
    } else if mime.starts_with("video/") {
        IconName::Movie
    } else if mime.starts_with("audio/") {
        IconName::AudioFile
    } else if mime.contains("zip") || mime.contains("compressed") || has(&["zip", "rar", "7z", "tar", "gz"]) {
        IconName::FolderZip
    } else if mime.contains("spreadsheet") || mime.contains("excel") || has(&["csv", "xls", "xlsx"]) {
        IconName::TableChart
    } else if mime.contains("pdf") || mime.starts_with("text/") || has(&["pdf", "doc", "docx", "md", "txt"]) {
        IconName::Description
    } else if has(&["css", "html", "js", "jsx", "json", "mdx", "ts", "tsx", "xml", "yaml", "yml", "rs", "py", "go"]) {
        IconName::Code
    } else if has(&["png", "jpg", "jpeg", "gif", "webp", "svg"]) {
        IconName::Image
    } else if has(&["mov", "mp4", "mkv", "webm"]) {
        IconName::Movie
    } else if has(&["mp3", "wav", "flac", "ogg", "m4a"]) {
        IconName::AudioFile
    } else {
        IconName::Draft
    }
}

/// Which of `paths` the queue takes: those with an accepted extension (any, when `accept` is empty), no more than the
/// room left, and only one when several are not allowed.
pub fn take_paths(paths: &[PathBuf], accept: &[String], room: Option<usize>, multiple: bool) -> Vec<PathBuf> {
    let allowed = |p: &PathBuf| {
        accept.is_empty() || p.extension().and_then(|e| e.to_str()).is_some_and(|e| accept.iter().any(|a| a.eq_ignore_ascii_case(e)))
    };
    let mut taken: Vec<PathBuf> = paths.iter().filter(|p| allowed(p) && p.is_file()).cloned().collect();
    let limit = match (room, multiple) {
        (Some(room), true) => room,
        (Some(room), false) => room.min(1),
        (None, true) => usize::MAX,
        (None, false) => 1,
    };
    taken.truncate(limit);
    taken
}

pub(super) fn status_mark(theme: &Theme, status: UploadStatus, spin: f32) -> impl IntoElement {
    let (icon, color) = match status {
        UploadStatus::Success => (IconName::CheckCircle, theme.success),
        UploadStatus::Error => (IconName::Error, theme.danger),
        UploadStatus::Uploading => (IconName::Progress, theme.foreground),
        UploadStatus::Queued => (IconName::Draft, theme.muted_foreground),
    };
    let icon = Icon::new(icon).size(px(16.)).color(color);
    if status == UploadStatus::Uploading { icon.turn(spin) } else { icon }
}

pub(super) fn round_button(theme: &Theme, id: ElementId, icon: IconName, selector: &'static str) -> gpui_kit::Stateful<gpui_kit::Div> {
    div()
        .id(id)
        .debug_selector(move || selector.into())
        .flex_none()
        .size(px(28.))
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .text_color(theme.muted_foreground)
        .cursor_pointer()
        .hover(|s| s.bg(theme.foreground.opacity(0.08)).text_color(theme.foreground))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .child(Icon::new(icon).size(px(14.)))
}
