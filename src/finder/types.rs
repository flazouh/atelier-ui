use gpui_kit::SharedString;

use crate::motion::Spring;

/// How many rows it shows at most.
pub const ROWS: usize = 50;

/// The panel's width (`max-w-xl`), and a row's height (`py-2` on a 20px line).
pub(super) const WIDTH: f32 = 576.;

pub(super) const ROW_HEIGHT: f32 = crate::combobox::ROW_HEIGHT;

/// The panel's entrance spring: it opens many times a day, so it reads as instant.
pub(super) const ENTER: Spring = Spring {
    stiffness: 560.,
    damping: 40.,
    mass: 0.5,
};

/// Who narrows the rows as the words change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter {
    /// The finder, fuzzily, over its label and detail.
    Here,
    /// The owner, from [`FinderEvent::Query`]; every row it sets is shown.
    Owner,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FinderEvent {
    /// The words changed. With [`Filter::Owner`], answer with [`Finder::set_items`](crate::finder::Finder::set_items).
    Query(SharedString),
    /// The row at this index of the items was chosen.
    Pick(usize),
    Dismiss,
}
