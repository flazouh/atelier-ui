//! Open/close motion shared by every section that opens and closes on its own: a chevron that turns
//! 180° on [`Spring::SWAP`] and a body that grows to its height and fades in on beui's `AgentDisclosure`
//! timing ([`motion::disclosure`]). TodoList, ToolCall, FileDiff, ToolApproval, ChangedFiles, SubagentCard
//! and AgentText's source panel each hold one of these instead of duplicating the channels and the
//! open/close function, and draw their body through [`body`].

use std::{cell::Cell, rc::Rc};

use gpui_kit::{AnyElement, IntoElement, ParentElement, Styled, canvas, div};

use crate::motion::{self, Channel, Curve, Spring};
use crate::scale::px;

pub(crate) struct Reveal {
    pub open: bool,
    pub reveal: Channel,
    pub chevron: Channel,
    /// The body's own height when it was last drawn, in window pixels, for the frames it is part open.
    pub height: Rc<Cell<f32>>,
}

impl Reveal {
    pub fn new(open: bool) -> Self {
        Self {
            open,
            reveal: Channel::new(if open { 1. } else { 0. }),
            chevron: Channel::new(if open { 180. } else { 0. }),
            height: Rc::default(),
        }
    }

    pub fn set_open(&mut self, open: bool, reduce: bool) {
        self.open = open;
        self.reveal.animate(
            if open { 1. } else { 0. },
            motion::disclosure(open),
            0.,
            reduce,
        );
        self.chevron.animate(
            if open { 180. } else { 0. },
            Curve::Spring(Spring::SWAP),
            0.,
            reduce,
        );
    }

    pub fn is_moving(&self) -> bool {
        self.reveal.is_running() || self.chevron.is_running()
    }
}

/// `body` at `reveal` (0 closed, 1 open): it fades in and settles 4px down, and what is under it moves
/// with it, because it takes `reveal` of its own height. A body that only faded would hold its full height
/// through the close and drop it at the end, and take it all on the first frame of the open: the jump
/// this is for. `height` is the [`Reveal::height`] the body is measured into; open, the body takes its height
/// from its content, so one that grows (a streaming log) grows freely.
pub(crate) fn body(body: impl IntoElement, reveal: f32, height: &Rc<Cell<f32>>) -> AnyElement {
    let measured = height.clone();
    let measure = canvas(
        move |bounds, _, _| measured.set(f32::from(bounds.size.height)),
        |_, _, _, _| {},
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full();
    let content = div()
        .relative()
        .top(px(-4. * (1. - reveal)))
        .opacity(reveal)
        .child(body)
        .child(measure);
    if reveal >= 0.999 {
        return content.into_any_element();
    }
    // Part open, the body is laid out at its own height out of the flow, and shown through a window that
    // is `reveal` of it.
    div()
        .relative()
        .overflow_hidden()
        .h(gpui_kit::px(height.get() * reveal.max(0.)))
        .child(div().absolute().top_0().left_0().right_0().child(content))
        .into_any_element()
}
