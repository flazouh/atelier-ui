//! WelcomePage: the first thing a person sees. The changelog's grain gradient fills the window and breathes, and in
//! its middle the Atelier mark, one line of welcome and one button come in, one after the other. There is
//! nothing else on the page: it asks for no choice, only a press.
//!
//! - It fills its parent. Put it in the window's content area.
//! - The pictures are the ones at [`HERO_PATH`] and [`HERO_B_PATH`], served by [`Assets`](crate::Assets): the
//!   gradient rendered for a whole window, and a second take of it that fades in and out over the first, so the
//!   light moves. The first zooms out a little as the page opens, and keeps a slow breath after.
//! - The timeline is a pure function of the time since the first frame ([`helpers::frame`]), so a test reads it
//!   without a window. Under Reduce Motion every part is at rest from the first frame and the picture stands still.
mod consts;
mod helpers;
mod structs;
pub use consts::{HERO_B_PATH, HERO_PATH};
pub use structs::WelcomePage;

/// The two pictures, for [`Assets`](crate::Assets) to serve.
pub(crate) fn bytes(path: &str) -> Option<&'static [u8]> {
    match path {
        HERO_PATH => Some(include_bytes!("../assets/welcome-hero.jpg")),
        HERO_B_PATH => Some(include_bytes!("../assets/welcome-hero-b.jpg")),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
