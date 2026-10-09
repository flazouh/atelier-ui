//! ReleaseSheet: what is new, as a changelog page. A band of grain gradient holds the title at its bottom left and
//! the close button at its top right. Under it stand the releases, the current one first and the earlier ones after
//! it: the version (and its date, when it has one) in a left column, the notes in a right column, and a hairline
//! between two releases. A note is a bold lead and a muted text, with no icon, no kind and no colour.
//!
//! - Put it in a [`Modal`](crate::modal::Modal) made with `flush`, so the picture reaches the corners. The modal
//!   caps its height to the window and scrolls the sheet, band and all.
//!
//! The picture is the one at [`HERO_PATH`], served by [`Assets`](crate::Assets). Its dark corner is at the top left,
//! so the title is light in every theme: it is the dark theme's foreground.
mod consts;
mod structs;
pub use consts::HERO_PATH;
pub use structs::{ReleaseNote, ReleaseSheet, ReleaseVersion};

/// The hero picture, for [`Assets`](crate::Assets) to serve.
pub(crate) fn bytes(path: &str) -> Option<&'static [u8]> {
    (path == HERO_PATH).then_some(include_bytes!("../assets/release-hero.jpg"))
}

#[cfg(test)]
mod tests;
