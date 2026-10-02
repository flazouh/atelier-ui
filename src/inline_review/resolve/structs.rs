use std::{ops::Range, time::Instant};

use gpui_kit::SharedString;

use super::super::Decision;
use crate::motion::duration;
use super::helpers::progress;

/// One hunk the user decided, fading until its edit runs and then closing.
#[derive(Clone, Debug)]
pub struct Resolve {
    pub id: SharedString,
    pub decision: Decision,
    pub(super) started: Instant,
    /// Set once the edit ran: the first row after the deleted ones, how many were deleted, and when.
    pub(super) collapse: Option<(usize, usize, Instant)>,
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
