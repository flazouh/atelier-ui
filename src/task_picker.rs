//! The small picker that `s`, `p`, `a` and `l` open: a title, the text typed so far, and the candidates
//! with the cursor on one. It draws a [`crate::task_edit::Picker`]; the owner routes the keys.

mod helpers;
mod structs;
mod types;

pub use helpers::{enter, handle_key, list_height, morph_popover, picker_popover, picker_view};
pub use structs::{Chip, PickerMorph};
pub use types::{Outcome, Pick, WIDTH};

#[cfg(test)]
mod tests;
