//! ReleaseSheet: what is new in a version, for the panel that asks the reader to restart. A grain gradient fills the
//! sheet and drifts slowly behind the words. The version stands large at the top left. The notes sit in a dark panel
//! over the lower part, and the choice (later, or restart now) is the panel's foot.
//!
//! - Enter: the picture zooms out into place over 1.6s, then drifts without end on two slow sines. The kicker, the
//!   version, each note and the foot come in one after the other, each rising and fading on `ease::OUT`.
//! - Reduce Motion: everything shows at once and the picture keeps still.
//! - Put it in a [`Modal`](crate::modal::Modal) made with `flush`, so the picture reaches the corners.
//!
//! The picture is the one at [`HERO_PATH`], served by [`Assets`](crate::Assets). Its dark corner is at the top left,
//! so the version is light in every theme: it is the dark theme's foreground.
mod consts;
mod helpers;
mod structs;

pub use consts::HERO_PATH;
pub use helpers::{drift, reveal, zoom};
pub use structs::{ReleaseNote, ReleaseSheet};

/// The hero picture, for [`Assets`](crate::Assets) to serve.
pub(crate) fn bytes(path: &str) -> Option<&'static [u8]> {
    (path == HERO_PATH).then_some(include_bytes!("../assets/release-hero.jpg"))
}

#[cfg(test)]
mod tests;
