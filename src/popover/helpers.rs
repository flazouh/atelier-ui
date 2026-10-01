use gpui_kit::{Anchor, Bounds, Pixels, Size, Window, point};

use crate::scale::px;
use crate::placement::opens_upward;
use super::structs::Registry;
use super::types::{Align, Side};

/// Counts the window's frames for the open popover `key`, and closes it when it has not been drawn for
/// [`HIDDEN_AFTER_FRAMES`] of them. It stops when the popover is closed or gone from the registry.
pub(super) fn watch(window: &mut Window, key: String) {
    window.on_next_frame(move |window, cx| {
        let hidden = {
            let registry = cx.default_global::<Registry>();
            registry.frame += 1;
            let Some(frames) = registry.frames.get_mut(&key) else { return };
            if frames.tick() {
                registry.frames.remove(&key).and_then(|f| f.close)
            } else {
                None
            }
        };
        match hidden {
            Some(close) => close(window, cx),
            None => {
                watch(window, key);
            }
        }
    });
}

/// The side the panel takes, and the point it hangs from.
pub fn placement(anchor: Bounds<Pixels>, side: Side, align: Align, gap: f32, height: f32, window: f32) -> (bool, Anchor, gpui_kit::Point<Pixels>) {
    let upward = match side {
        Side::Above | Side::CoverAbove => true,
        Side::Below | Side::CoverBelow => false,
        Side::Auto => opens_upward(anchor, height, gap, window),
    };
    let x = match align {
        Align::Start => anchor.left(),
        Align::End => anchor.right(),
        Align::Center => anchor.center().x,
    };
    let cover = matches!(side, Side::CoverBelow | Side::CoverAbove);
    let below = if cover { anchor.top() } else { anchor.bottom() + gpui_kit::px(gap) };
    let above = if cover { anchor.bottom() } else { anchor.top() - gpui_kit::px(gap) };
    let (y, corner) = match (upward, align) {
        (false, Align::Start) => (below, Anchor::TopLeft),
        (false, Align::End) => (below, Anchor::TopRight),
        (false, Align::Center) => (below, Anchor::TopCenter),
        (true, Align::Start) => (above, Anchor::BottomLeft),
        (true, Align::End) => (above, Anchor::BottomRight),
        (true, Align::Center) => (above, Anchor::BottomCenter),
    };
    (upward, corner, point(x, y))
}

/// Rectangles that cover the window but the holes: one band per run of rows that the same holes cross,
/// split around them.
pub(super) fn cover(window: Size<Pixels>, holes: &[Bounds<Pixels>]) -> Vec<Bounds<Pixels>> {
    let (w, h) = (f32::from(window.width), f32::from(window.height));
    let holes: Vec<[f32; 4]> = holes
        .iter()
        .map(|b| [f32::from(b.left()).max(0.), f32::from(b.top()).max(0.), f32::from(b.right()).min(w), f32::from(b.bottom()).min(h)])
        .filter(|[l, t, r, b]| l < r && t < b)
        .collect();
    let mut ys: Vec<f32> = holes.iter().flat_map(|[_, t, _, b]| [*t, *b]).chain([0., h]).collect();
    ys.sort_by(f32::total_cmp);
    ys.dedup();
    let rect = |l: f32, t: f32, r: f32, b: f32| Bounds::new(point(px(l), px(t)), gpui_kit::size(px(r - l), px(b - t)));
    let mut out = Vec::new();
    for band in ys.windows(2) {
        let (top, bottom) = (band[0], band[1]);
        let mut across: Vec<(f32, f32)> = holes.iter().filter(|[_, t, _, b]| *t < bottom && *b > top).map(|[l, _, r, _]| (*l, *r)).collect();
        across.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut x = 0.;
        for (l, r) in across {
            if l > x {
                out.push(rect(x, top, l, bottom));
            }
            x = f32::max(x, r);
        }
        if x < w {
            out.push(rect(x, top, w, bottom));
        }
    }
    out
}
