//! A decided hunk on its way out, in two steps.
//!
//! 1. The fade, over `duration::RESOLVE_FADE`: a cover in the editor's background colour thickens
//!    over the closing rows, so their text and wash fade together, while the surviving rows lose
//!    their wash.
//! 2. The collapse, over `duration::RESOLVE`: the edit has run and the closing rows are gone, but a
//!    row gap as tall as they were holds their place and shrinks to nothing, so the rows below slide
//!    up instead of jumping. A buffer row cannot shrink (see `docs/inline-review.md`); a gap above
//!    one can.
//!
//! Both steps run on `ease::MORPH`. Under Reduce Motion neither runs: the edit lands at once.

mod helpers;
mod structs;

pub use structs::Resolve;

#[cfg(test)]
use super::Decision;
#[cfg(test)]
use crate::motion::duration;
#[cfg(test)]
use std::time::Instant;

#[cfg(test)]
mod tests;
