//! How an agent looks in the agent panel: its animated mark, the colours of its status label, and the
//! words for each phase. atelier-ui knows no agent; an agent crate builds an [`AgentLook`] and hands it to
//! [`crate::thinking::Thinking`] and [`crate::subagent_row::SubagentRow`] as data.
//!
//! [`AgentLook::neutral`] is a plain fallback for tests and for stories that show no particular agent.

use gpui_kit::{ElementId, Hsla, SharedString};

use crate::sprite::{Sprite, Strip};

/// An agent's animated mark: the strips each moment plays, and the colour they draw in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mark {
    /// While the agent works. Its first frame is also the still mark: work has ended, or Reduce Motion
    /// is on.
    pub working: Strip,
    /// While subagents run.
    pub orbiting: Strip,
    pub color: Hsla,
    /// The frame of `working` that is the agent's whole mark, held still where an icon stands for the agent (a list
    /// row). The first frame is the resting mark of the working line, which can be a dot.
    pub icon_frame: usize,
}

impl Mark {
    /// A [`Sprite`] that plays `strip` in the mark's colour and rests on the mark's still frame.
    pub fn sprite(&self, id: impl Into<ElementId>, strip: Strip) -> Sprite {
        Sprite::new(id, strip, self.color).rest(self.working)
    }
}

/// The status label's words for each [`crate::thinking::ThinkingPhase`]. "Thought for 4s" is not here:
/// it is a count, the same for every agent.
#[derive(Clone, Debug, PartialEq)]
pub struct PhaseLabels {
    /// While thinking: each label shows from its number of seconds on. The first starts at 0.
    pub thinking: &'static [(f32, &'static str)],
    pub connecting: SharedString,
    pub sending: SharedString,
    pub starting: SharedString,
    pub preparing: SharedString,
    pub waiting: SharedString,
    pub running_tools: SharedString,
}

impl Default for PhaseLabels {
    /// Words that name no agent.
    fn default() -> Self {
        Self {
            thinking: &[
                (0., "Thinking…"),
                (15., "Still thinking…"),
                (30., "Thinking more…"),
                (45., "Thinking some more…"),
                (60., "Almost done thinking…"),
            ],
            connecting: "Connecting…".into(),
            sending: "Sending…".into(),
            starting: "Starting session…".into(),
            preparing: "Preparing…".into(),
            waiting: "Waiting for the agent…".into(),
            running_tools: "Running tools…".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentLook {
    pub mark: Mark,
    /// The status label's colour while it moves, and a running subagent's name.
    pub message: Hsla,
    /// The lighter colour that walks across the label.
    pub glimmer: Hsla,
    pub labels: PhaseLabels,
}

/// A one-frame mark: Material's `smart_toy`, which holds still.
const NEUTRAL_STRIP: Strip = Strip {
    path: "icons/smart_toy.svg",
    bytes: include_bytes!("../assets/icons/smart_toy.svg"),
    frames: 1,
    frame_ms: 1000,
    loops: false,
};

impl AgentLook {
    /// A grey look that names no agent, in `theme`'s muted text.
    pub fn neutral(theme: &crate::theme::Theme) -> Self {
        let muted = theme.muted_foreground;
        Self {
            mark: Mark { working: NEUTRAL_STRIP, orbiting: NEUTRAL_STRIP, color: muted, icon_frame: 0 },
            message: muted,
            glimmer: crate::theme::mix(muted, theme.foreground, 0.5),
            labels: PhaseLabels::default(),
        }
    }
}

#[cfg(test)]
mod tests;
