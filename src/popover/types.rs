use std::rc::Rc;

use gpui_kit::{App, Window};

/// The deferred priority of the backdrop; the panel is one above. Above gpui-base's popups and dialogs.
pub const PRIORITY: usize = 200;

/// Distance kept between a panel and the window's edge.
pub(super) const MARGIN: f32 = 8.;

/// A popover counts as hidden (its view went away) when the window has drawn this many frames since it was last
/// drawn. Frames are counted, never timed: a slow frame is still one frame.
pub(super) const HIDDEN_AFTER_FRAMES: u64 = 3;

pub type CloseHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// Where the panel goes against its trigger.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// Below when it fits, else above when there is more room there.
    Auto,
    Below,
    Above,
    /// Over the trigger, the panel's top edge on the trigger's top edge and growing down: a surface that
    /// grew out of the trigger.
    CoverBelow,
    /// Over the trigger, the panel's bottom edge on the trigger's bottom edge and growing up.
    CoverAbove,
}

/// Which edge of the trigger the panel lines up with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Start,
    End,
    Center,
}

/// Where a popover with no trigger of its own hangs: a point of its parent's box, in the parent's frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hang {
    /// `x` from the parent's left edge, `y` from its top.
    Left(f32, f32),
    /// `x` from the parent's right edge, `y` from its top.
    Right(f32, f32),
    /// Centred on the parent, `y` from its top.
    Centre(f32),
}
