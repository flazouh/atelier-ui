//! A message arriving in the conversation, as beui's `Message animateIn`: it fades in while rising from [`RISE`] px
//! below, on [`Spring::MESSAGE_POP`]. beui also grows it from 95% out of its bottom corner; a GPUI element cannot be
//! scaled, so the rise and the fade carry it. Under Reduce Motion it is there at once, as beui's is.
use crate::motion::Spring;

/// How far below its place a message starts.
pub const RISE: f32 = 8.;

/// Where a message is `elapsed` seconds after it arrived.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopFrame {
    pub opacity: f32,
    pub y: f32,
    pub settled: bool,
}

pub fn frame(elapsed: f32, reduce_motion: bool) -> PopFrame {
    if reduce_motion {
        return PopFrame { opacity: 1., y: 0., settled: true };
    }
    let x = Spring::MESSAGE_POP.position(elapsed.max(0.));
    if elapsed > 0.05 && (1. - x).abs() < 0.0005 {
        return PopFrame { opacity: 1., y: 0., settled: true };
    }
    PopFrame { opacity: x.clamp(0., 1.), y: RISE * (1. - x), settled: false }
}

#[cfg(test)]
mod tests;
