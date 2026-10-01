//! A popover that finds one thing among many: a filter field over a list, the arrow keys to walk it,
//! Enter to open the chosen row and Escape to close. "Go to file", "Go to name", "Names in this file"
//! and "Uses" are each one of these; the owner fills the rows and says what a pick does.
//!
//! - It filters its own rows ([`Filter::Here`], fuzzy: see [`crate::fuzzy`]), or it hands each change
//!   of the words to the owner, who answers with new rows ([`Filter::Owner`]), as a language server's
//!   "Go to name" does.
//! - A note stands in for the rows while there are none, such as "Asking rust-analyzer".
//! - The title wears the key cap that opens it, so the key is learned where it is used.
//! - A `card` fill with the popover shadow and no border, as every popover here.

mod helpers;
mod structs;
mod types;

pub use structs::{Finder, FinderItem};
pub use types::{Filter, FinderEvent, ROWS};

#[cfg(test)]
mod tests;
