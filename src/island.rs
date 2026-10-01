//! Island: beui.dev's Dynamic Island (`components/motion/dynamic-island.tsx`). A dark pill that holds live state and
//! grows or shrinks to fit it. The shell's width and height follow the content's natural size on a spring
//! given in Apple's form, `duration 0.8, bounce 0.2` (`{ stiffness 61.7, damping 12.6, mass 1 }`), which makes
//! one long glide with barely a bounce. The corner is fixed at 32px, which the size clamps to half the
//! height, so the pill and the rounded box are one shape.
//!
//! [`Island`] is the shell. [`SessionsIsland`] is atelier's use of it: plain counts in, no names of agents:
//! how many sessions are running, how many need the reader, and how many finished and are not yet seen. A
//! press runs [`SessionsIsland::on_press`]. It draws nothing when all three are zero.
//!
//! The pill is atelier's look, which the title bar sets: a `card_strong` chip with the foreground on it, no shadow.
//! (The web's is the foreground colour with the page's on it, with `shadow-2xl`.) Under Reduce Motion the
//! shell takes the content's size at once. What gpui cannot draw is left out: the blur and the 0.9 scale of the
//! content as it changes (the words stay in place and the digits roll).

mod helpers;
mod structs;
mod types;

pub use helpers::{colors, counts_of, most_urgent};
pub use structs::{Island, IslandCounts, SessionsIsland};
pub use types::{HEIGHT, PILL, RADIUS, SHELL};

#[cfg(test)]
use crate::{
    session_status::{Need, SessionStatus},
    sidebar_model::ProjectData,
};

#[cfg(test)]
mod tests;
