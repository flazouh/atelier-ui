//! WelcomePage: the first thing a person sees. The changelog's grain gradient fills the window, dark at the left
//! and under the button, with its light in the top right. The Atelier mark stands at the top left, the title and
//! one line under it stand in the middle, and one button stands alone at the bottom right. There is nothing else
//! on the page: it asks for no choice, only a press.
//!
//! - It fills its parent. Put it in the window's content area.
//! - The picture is the one at [`HERO_PATH`], served by [`Assets`](crate::Assets): the gradient rendered for a
//!   whole window.
//! - The title, then the line, arrive as a message does in a session. The block fades in and rises 6px
//!   ([`Entrance`](crate::entrance::Entrance)) as its first word comes, and the words come one at a time, each
//!   piece fading in over the same 100ms the answer's tail uses ([`Flow`](crate::stream_text::Flow)). The button
//!   comes once the line is in. The page starts when the picture is loaded, and the picture fades in, so no word
//!   comes in over an empty page. What is shown at a moment is a pure function of the time since that start
//!   ([`helpers::shown`]), so a test reads it without a window. Under Reduce Motion everything stands from the
//!   first frame.
mod consts;
mod helpers;
mod structs;
pub use consts::HERO_PATH;
pub use structs::WelcomePage;

/// The picture, for [`Assets`](crate::Assets) to serve.
pub(crate) fn bytes(path: &str) -> Option<&'static [u8]> {
    (path == HERO_PATH).then_some(include_bytes!("../assets/welcome-hero.jpg"))
}

#[cfg(test)]
mod tests;
