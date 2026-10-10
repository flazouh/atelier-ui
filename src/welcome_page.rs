//! WelcomePage: the first thing a person sees. The changelog's grain gradient fills the window, and in its middle
//! stands one card with no border: the Atelier mark, a few words of welcome that stream in as an agent's answer
//! does, and one button that comes once the words are in. There is nothing else on the page: it asks for no
//! choice, only a press.
//!
//! - It fills its parent. Put it in the window's content area.
//! - The picture is the one at [`HERO_PATH`], served by [`Assets`](crate::Assets): the gradient rendered for a
//!   whole window.
//! - The words come in one at a time, and each piece fades in over the same 100ms the answer's tail uses
//!   ([`Flow`](crate::stream_text::Flow)). What is shown at a moment is a pure function of the time since the first
//!   frame ([`helpers::shown`]), so a test reads it without a window. Under Reduce Motion everything stands from
//!   the first frame.
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
