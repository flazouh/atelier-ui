//! Segmented: beui.dev's inline choice control (`file-upload.preview.tsx`), in atelier's look: a row of small
//! buttons, as the app had before, with no track. A segment is a `Sm` button (28px, 10px across, `rounded LG`):
//! the chosen one wears the `Secondary` fill and the others the `Ghost` one. Hover takes the text to the
//! foreground. Fill and text change over 150ms, and a pressed segment shrinks to 95%. Reduce Motion makes
//! every change a jump.
//!
//! Each segment is a Tab stop. Enter and Space choose it. A key cap belongs to the segment that key picks; a key
//! that toggles between the segments is shown once, after the control ([`Segmented::cap`]).

mod helpers;
mod structs;
mod types;

pub use helpers::{pill_inset, segment_fill, segment_text};
pub use structs::{Segment, Segmented};
pub use types::{CAP_GAP, GAP, LINE, SEGMENT_HEIGHT, SEGMENT_PAD, TEXT};

#[cfg(test)]
use structs::Motion;

#[cfg(test)]
use crate::theme::Theme;

#[cfg(test)]
mod tests;
