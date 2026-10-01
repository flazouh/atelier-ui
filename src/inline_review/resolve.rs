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

use std::{ops::Range, time::Instant};

use gpui_kit::SharedString;

use super::Decision;
use crate::motion::{cubic_bezier, duration, ease};

/// One hunk the user decided, fading until its edit runs and then closing.
#[derive(Clone, Debug)]
pub struct Resolve {
    pub id: SharedString,
    pub decision: Decision,
    started: Instant,
    /// Set once the edit ran: the first row after the deleted ones, how many were deleted, and when.
    collapse: Option<(usize, usize, Instant)>,
}

/// How far `started` is into a step of length `step`, eased, or `None` once the step is over.
fn progress(started: Instant, now: Instant, step: std::time::Duration) -> Option<f32> {
    let t = now.saturating_duration_since(started).as_secs_f32() / step.as_secs_f32();
    (t < 1.).then(|| cubic_bezier(ease::MORPH, t))
}

impl Resolve {
    /// A decision made now.
    pub fn new(id: impl Into<SharedString>, decision: Decision) -> Self {
        Self::at(id, decision, Instant::now())
    }

    /// A decision made at `started`.
    pub fn at(id: impl Into<SharedString>, decision: Decision, started: Instant) -> Self {
        Self { id: id.into(), decision, started, collapse: None }
    }

    /// How much of the closing rows is left at `now`, eased from 1 to 0. `None` once the fade is over,
    /// which is when the edit runs, and always `None` after it ran. Under Reduce Motion there is no
    /// fade, so the edit runs at once.
    pub fn fade(&self, now: Instant, reduce_motion: bool) -> Option<f32> {
        if reduce_motion || self.collapse.is_some() {
            return None;
        }
        progress(self.started, now, duration::RESOLVE_FADE).map(|done| 1. - done)
    }

    /// Whether the edit ran.
    pub fn is_edited(&self) -> bool {
        self.collapse.is_some()
    }

    /// Records that the edit ran now and deleted `closed`, which starts the collapse.
    pub fn edited(&mut self, closed: Range<usize>) {
        self.edited_at(closed, Instant::now());
    }

    /// Records that the edit ran at `at` and deleted `closed`.
    pub fn edited_at(&mut self, closed: Range<usize>, at: Instant) {
        self.collapse = Some((closed.start, closed.len(), at));
    }

    /// The gap holding the deleted rows' place at `now`, as `(row, rows tall)`: `row` is the first row
    /// after them. `None` before the edit, once the collapse is over, for a hunk that deleted nothing,
    /// and under Reduce Motion.
    pub fn gap(&self, now: Instant, reduce_motion: bool) -> Option<(usize, f32)> {
        let (row, rows, started) = self.collapse?;
        if reduce_motion || rows == 0 {
            return None;
        }
        progress(started, now, duration::RESOLVE).map(|done| (row, rows as f32 * (1. - done)))
    }

    /// Whether nothing is left to draw: the edit ran and its gap has closed. The owner can drop it.
    pub fn is_over(&self, now: Instant, reduce_motion: bool) -> bool {
        self.is_edited() && self.gap(now, reduce_motion).is_none()
    }
}

#[cfg(test)]
mod tests;
