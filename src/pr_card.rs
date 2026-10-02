//! The session's pull request, one row above the composer: its state mark, `#3344`, its title on one
//! line, the checks summary, and the review state. Copy link and Open in browser sit at the right edge;
//! they take their space at all times and only show on hover, so nothing moves when the pointer arrives.
//! Pressing the row reports it. When the reader can merge an open pull request, a
//! [`MergeButton`](crate::merge_button::MergeButton) sits at the far right; a press on it reports the action, not the row.

mod helpers;
mod structs;
mod types;

pub use structs::PrCard;
pub(crate) use structs::LinkActions;

#[cfg(test)]
use crate::{
    merge::{MergeFacts, PullState, Rights},
    pr::PrChipData,
};

#[cfg(test)]
mod tests;
