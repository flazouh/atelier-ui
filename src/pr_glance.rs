//! A pull request's card, as a chip's hover opens it: one view per chip that observes the app's one
//! [`PrCardStore`], so a read that lands draws only the open card again.
//!
//! The card draws from the chip's data at once, then from what the app read once it opened: fresher facts, the
//! first failing check and the first line of its log that says why, the biggest changed files, and the session
//! the pull request came from. Each sits in its own section: Checks, Review, Changes and Session, with the merge
//! standing beside Merge at the foot. Each part has a key ([`PrPart`]) the reader can hide from Settings, which
//! changes the store's [`PrParts`]. Merge asks once more before it merges, and the state pill opens the pull
//! request in the app.
//!
//! The app sets [`PrCardStore::on_open`] to hear each card open and close, so it reads only while one is open,
//! and [`PrCardStore::on_action`] for the card's buttons.

mod helpers;
mod structs;
mod types;

pub(crate) use helpers::size_text;
pub use helpers::{key_of, pr_cards, top_files};
pub use structs::{
    PrCardStore, PrFailing, PrFile, PrGlance, PrGlanceCard, PrKey, PrParts, PrSession,
};
pub use types::{PrAction, PrDoing, PrPart, TOP_FILES};

#[cfg(test)]
mod tests;
