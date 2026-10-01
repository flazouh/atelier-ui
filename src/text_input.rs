//! TextInput: beui.dev's Input (`components/motion/input.tsx`), in atelier's look. A field 28px tall with an
//! optional label over it, an icon at either end, and a line for an error under it.
//!
//! - Field: the app's own field from before this part: `rounded LG` (8), the `card_strong` fill with no border, the
//!   text at 14px, 10px in from the edge (30px with an icon), and a small muted label over it. The web's pill is
//!   44px with a border; that is its look, not ours.
//! - Focus: the quiet ring (atelier's ring: 2px, 3:1 with the surface).
//! - Error: a 2px ring in the danger tone, the field shakes once
//!   (`x: 0, -6, 6, -4, 4, -2, 0` over 450ms), and the message comes up under it over 200ms. With
//!   [`TextInput::reserve_error_line`] the line keeps its 16px, so nothing below it moves.
//! - Success: a check at the right end. Under Reduce Motion nothing shakes or slides.
//!
//! What gpui cannot draw is left out: the blur on the message as it comes and goes, the message's exit
//! (it goes at once), and the check's draw-on (it fades in).

mod helpers;
mod structs;
mod types;

pub use helpers::{edge, fill, shake_offset};
pub use structs::TextInput;
pub use types::{
    CORNER, GAP, HEIGHT, MESSAGE_LINE, SHAKE, SHAKE_SECONDS, TEXT_INSET, TEXT_INSET_ICON,
};

#[cfg(test)]
mod tests;
