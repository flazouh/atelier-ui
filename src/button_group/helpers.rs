use gpui_kit::{Axis, Corners};

/// The corners segment `index` of `len` rounds: the group's outer corners only, as gpui-component's
/// group sets them.
pub fn segment_corners(index: usize, len: usize, layout: Axis) -> Corners<bool> {
    let vertical = layout == Axis::Vertical;
    let (first, last) = (index == 0, index + 1 == len);
    Corners {
        top_left: first,
        top_right: if vertical { first } else { last },
        bottom_left: if vertical { last } else { first },
        bottom_right: last,
    }
}
