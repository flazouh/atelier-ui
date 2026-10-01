use std::rc::Rc;

use gpui_kit::{App, Window};

/// How many faces a line shows before the rest become a count.
pub(super) const SHOWN: usize = 3;

/// What a press on "Show more" does.
pub type MoreHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// A callback that receives the index of a thread in the order the list shows: open threads first, each group
/// in its own order (see [`open_first`]).
pub type ThreadHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// Each face overlaps the one before by this much.
pub(super) const FACE: f32 = 18.;

pub(super) const FACE_STEP: f32 = 12.;
