//! The subagents of a running turn, one [`SubagentRow`] each, above the composer.
//!
//! The strip grows and shrinks as rows join and leave, never in a jump: each row's slot opens from no
//! height to its own on [`Spring::LAYOUT`] and fades in with it, and closes the same way. A finished row
//! keeps its check for [`duration::FINISH_HOLD`], then leaves, and stays gone while the data still
//! holds it finished; it joins again only if it runs again. A row that drops out of the data leaves
//! at once, drawn from its last known state. Rows already there on the first paint show at once. Under
//! Reduce Motion slots open and close at once; the hold stays, since it is timing, not movement.

use std::{collections::HashSet, time::Instant};

use gpui_kit::{App, ElementId, IntoElement, ParentElement, RenderOnce, Styled, Window, div, };
use crate::scale::px;

use crate::{
    motion::{Channel, Curve, Spring, duration},
    subagent_row::{ROW_HEIGHT, SubagentRow},
    wake::Wake,
};

/// Space between rows.
const GAP: f32 = 4.;

struct Slot {
    row: SubagentRow,
    presence: Channel,
    /// When the row first showed as finished.
    finished_at: Option<Instant>,
    leaving: bool,
}

/// The rows on screen, in order, with how far each slot is open.
#[derive(Default)]
pub(crate) struct StripState {
    painted: bool,
    slots: Vec<Slot>,
    /// Finished rows that have held and left.
    dismissed: HashSet<ElementId>,
}

fn open(to: f32, reduce: bool, now: Instant, from: f32) -> Channel {
    let mut c = Channel::new(from);
    c.animate_at(to, Curve::Spring(Spring::LAYOUT), 0., reduce, now);
    c
}

impl StripState {
    /// Brings the slots in line with `rows` at `now`. Returns when a held row is due to leave, so the
    /// caller can wake then.
    pub fn sync(&mut self, rows: Vec<SubagentRow>, now: Instant, reduce: bool) -> Option<Instant> {
        let first = !std::mem::replace(&mut self.painted, true);
        // A row that is gone from the data leaves from where it is.
        for slot in &mut self.slots {
            if !slot.leaving && !rows.iter().any(|r| r.id() == slot.row.id()) {
                slot.leaving = true;
                slot.presence.animate_at(0., Curve::Spring(Spring::LAYOUT), 0., reduce, now);
            }
        }
        for row in rows {
            if row.is_finished() && self.dismissed.contains(row.id()) {
                continue;
            }
            self.dismissed.remove(row.id());
            match self.slots.iter_mut().find(|s| s.row.id() == row.id()) {
                Some(slot) => {
                    if row.is_finished() {
                        slot.finished_at.get_or_insert(now);
                    } else {
                        slot.finished_at = None;
                        if slot.leaving {
                            slot.leaving = false;
                            slot.presence.animate_at(1., Curve::Spring(Spring::LAYOUT), 0., reduce, now);
                        }
                    }
                    slot.row = row;
                }
                None => {
                    let finished_at = row.is_finished().then_some(now);
                    let presence = if first { Channel::new(1.) } else { open(1., reduce, now, 0.) };
                    self.slots.push(Slot { row, presence, finished_at, leaving: false });
                }
            }
        }
        // A held row whose time is up leaves.
        for slot in &mut self.slots {
            if let Some(at) = slot.finished_at
                && !slot.leaving
                && now >= at + duration::FINISH_HOLD
            {
                slot.leaving = true;
                slot.presence.animate_at(0., Curve::Spring(Spring::LAYOUT), 0., reduce, now);
                self.dismissed.insert(slot.row.id().clone());
            }
        }
        // A slot that has closed all the way is gone.
        self.slots.retain(|s| !(s.leaving && !s.presence.is_running_at(now)));
        self.slots.iter().filter(|s| !s.leaving).filter_map(|s| s.finished_at).map(|at| at + duration::FINISH_HOLD).min()
    }

    pub fn is_moving(&self, now: Instant) -> bool {
        self.slots.iter().any(|s| s.presence.is_running_at(now))
    }

    /// Each slot's row and how far it is open, from 0 to 1.
    pub fn slots(&self, now: Instant) -> impl Iterator<Item = (&SubagentRow, f32)> {
        self.slots.iter().map(move |s| (&s.row, s.presence.value_at(now).clamp(0., 1.)))
    }
}

#[derive(IntoElement)]
pub struct SubagentStrip {
    id: ElementId,
    rows: Vec<SubagentRow>,
}

impl SubagentStrip {
    pub fn new(id: impl Into<ElementId>, rows: Vec<SubagentRow>) -> Self {
        Self { id: id.into(), rows }
    }
}

struct StripMotion {
    state: StripState,
    wake: Wake,
}

impl RenderOnce for SubagentStrip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduce = cx.reduce_motion();
        let now = Instant::now();
        let motion = window.use_keyed_state(self.id, cx, |_, _| StripMotion { state: StripState::default(), wake: Wake::default() });
        let rows = self.rows;
        motion.update(cx, |m, cx| match m.state.sync(rows, now, reduce) {
            Some(due) => m.wake.at(due, cx),
            None => m.wake.cancel(),
        });
        let m = motion.read(cx);
        if m.state.is_moving(now) {
            window.request_animation_frame();
        }
        // Each slot carries the gap above its row, so a closing slot takes its gap with it; the strip's
        // own negative top margin hides the first one.
        let slot = ROW_HEIGHT + GAP;
        div().flex().flex_col().mt(px(-GAP)).children(m.state.slots(now).map(|(row, open)| {
            div().flex_none().h(px(slot * open)).overflow_hidden().opacity(open).pt(px(GAP)).child(row.clone())
        }))
    }
}

#[cfg(test)]
mod tests;
