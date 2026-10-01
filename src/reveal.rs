//! Open/close motion shared by every section that opens and closes on its own: a chevron that turns
//! 180° on [`Spring::SWAP`] and a body that fades in on beui's `AgentDisclosure` timing
//! ([`motion::disclosure`]). TodoList, ToolCall, FileDiff, ToolApproval, and AgentText's source panel
//! each hold one of these instead of duplicating the two channels and the open/close function.

use crate::motion::{self, Channel, Curve, Spring};

pub(crate) struct Reveal {
    pub open: bool,
    pub reveal: Channel,
    pub chevron: Channel,
}

impl Reveal {
    pub fn new(open: bool) -> Self {
        Self {
            open,
            reveal: Channel::new(if open { 1. } else { 0. }),
            chevron: Channel::new(if open { 180. } else { 0. }),
        }
    }

    pub fn set_open(&mut self, open: bool, reduce: bool) {
        self.open = open;
        self.reveal.animate(if open { 1. } else { 0. }, motion::disclosure(open), 0., reduce);
        self.chevron.animate(if open { 180. } else { 0. }, Curve::Spring(Spring::SWAP), 0., reduce);
    }

    pub fn is_moving(&self) -> bool {
        self.reveal.is_running() || self.chevron.is_running()
    }
}
