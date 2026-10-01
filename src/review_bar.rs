//! The bar on top of a review, for the agent's turn and the pull request view alike.
//!
//! - Left: "7 changed", `+62 −12` in the diff colours, a thin progress line, and "3 of 7 reviewed". The line
//!   fills on [`Spring::LAYOUT`] when the count changes; once every file is reviewed it turns `success`,
//!   and the words read "All 7 reviewed".
//! - Right: the mark ("Mark file", "Seen"), Review mode, Previous and Next, each with its words and its key
//!   cap inside it (the cap read from the key table), and a menu (…) that holds Accept all and Reject all
//!   with their caps, and Put all back, each where the owner handles it. Next reads
//!   Done once every file is reviewed. It holds reading and moving only, as GitQuiet's top bar does:
//!   Accept file and Reject file act on one file, so they sit on its card
//!   ([`crate::review_file_header::ReviewFileHeader`]).
//!
//! A narrow pane takes things away one [`Step`] at a time, stopping at the first that fits: the progress
//! line, then the totals, then the navigation words. A button never shows its cap alone: without words
//! it shows its icon beside the cap, and its words move to its tooltip. The bar measures what each step needs off its own layout, so the fit rests
//! on real widths rather than guesses.
//!
//! A review with two scopes (one turn or the whole session) shows them as a switch before the summary:
//! the scope in force on a tone, the other with the switch's cap.
//!
//! The owner handles every action through [`ReviewHandlers`], from these buttons and from the keys alike.
//! The bar takes focus when pressed.

mod helpers;
mod structs;
mod types;

pub use helpers::{choose, menu_entries, need_at};
pub(crate) use helpers::worded;
pub use structs::{ReviewBar, Step};
pub use types::STEPS;

#[cfg(test)]
use types::FIT_SLACK;

#[cfg(test)]
use crate::review::ReviewHandlers;

#[cfg(test)]
mod tests;
