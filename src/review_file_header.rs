//! The head of a file's card in a review: a file icon, the path with its folder muted and its name in
//! ink, `+a −r`, and on the right Accept file (primary) and Reject file, each with its key cap. It sits
//! above the editor and does not scroll with the text, as the file heading does in GitQuiet's pull
//! request screen. Where a breadcrumb over the card already gives the path, [`ReviewFileHeader::path_shown`] takes
//! the path out of the head, leaving the counts and the buttons.
//!
//! In a narrow card the path gives way first: its folder truncates while the name stays whole. Only when
//! even the name would be cut do the two buttons drop the word "file". The header measures both off its
//! own layout, as [`crate::review_bar::ReviewBar`] does.
//!
//! A file the pull request did not change, opened to read beside it, says "Brought in" where the counts
//! go ([`ReviewFileHeader::brought_in`]). Where a language server answers, the header also holds its
//! four lookups (Uses, Names in this file, Go to name, Go to file), each an icon beside its key cap, the
//! word in its tooltip: they show for each handler set in [`ReviewHandlers`].

mod helpers;
mod structs;
mod types;

pub use structs::ReviewFileHeader;

#[cfg(test)]
use crate::review::ReviewHandlers;

#[cfg(test)]
mod tests;
