//! The model a turn or a subagent uses, such as "Opus 5.5": small muted text with no fill, so it stays
//! quieter than a status [`crate::badge::Badge`] in a dense header. It may lead with the lab's mark.
//!
//! The mark is data ([`BrandMark`]): atelier-ui names no lab, and an agent crate hands the asset paths over. It
//! sits before the label at the label's size, in its own colours, as acepe draws it: grey and at half
//! opacity at rest, taking its colour when the pointer is on the badge, or on its row when the owner says
//! so ([`ModelBadge::lit`]), over `duration::REVEAL`. Under Reduce Motion it switches at once. With no
//! mark, a monogram of the label's first letter stands in its place.

mod helpers;
mod structs;
mod types;

pub use helpers::monogram_letter;
pub use structs::{BrandMark, ModelBadge};

#[cfg(test)]
use crate::theme::Appearance;

#[cfg(test)]
mod tests;
