//! ReleaseSheet: what is new in a version, for the panel that asks the reader to restart. A grain gradient fills the
//! sheet behind the words. The version stands large at the top left. The notes sit in a dark panel
//! over the lower part, and the choice (later, or restart now) is the panel's foot.
//!
//! - Put it in a [`Modal`](crate::modal::Modal) made with `flush`, so the picture reaches the corners.
//!
//! The picture is the one at [`HERO_PATH`], served by [`Assets`](crate::Assets). Its dark corner is at the top left,
//! so the version is light in every theme: it is the dark theme's foreground.
mod consts;
mod helpers;
mod structs;

pub use consts::HERO_PATH;
pub use structs::{ReleaseNote, ReleaseSheet, ReleaseVersion};

/// The hero picture, for [`Assets`](crate::Assets) to serve.
pub(crate) fn bytes(path: &str) -> Option<&'static [u8]> {
    (path == HERO_PATH).then_some(include_bytes!("../assets/release-hero.jpg"))
}

#[cfg(test)]
mod tests;
