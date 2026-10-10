//! The release card: what is new in one version, in the shape of a small modal. A picture with the app's name and the
//! version on it, a title with its date, a few notes each with an icon, and two buttons at the foot. It sits in a
//! `Modal` made with `flush` and says nothing about how it opens: the app gives it the notes and hears the two buttons.
//!
//! The picture is the one at [`HERO_PATH`], served by [`Assets`](crate::Assets).
mod consts;
mod structs;
pub use consts::HERO_PATH;
pub use structs::{ReleaseCard, ReleaseCardNote};

/// The hero picture, for [`Assets`](crate::Assets) to serve.
pub(crate) fn bytes(path: &str) -> Option<&'static [u8]> {
    (path == HERO_PATH).then_some(include_bytes!("../assets/release-card-hero.jpg"))
}
#[cfg(test)]
mod tests;
