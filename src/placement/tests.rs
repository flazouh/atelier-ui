use gpui_kit::{Bounds, point, px, size};

use super::*;

fn anchor(top: f32) -> Bounds<Pixels> {
    Bounds::new(point(px(0.), px(top)), size(px(100.), px(32.)))
}

#[test]
fn a_panel_opens_below_while_it_fits() {
    assert!(!opens_upward(anchor(100.), 200., 8., 800.));
    // Exactly enough room below still opens below.
    assert!(!opens_upward(anchor(560.), 200., 8., 800.));
}

#[test]
fn a_panel_turns_up_when_below_is_short_and_above_is_larger() {
    assert!(opens_upward(anchor(700.), 200., 8., 800.));
}

#[test]
fn a_panel_stays_below_when_above_is_shorter_still() {
    // A 300px window: 60 above, 200 below, and the panel fits neither; below has more room.
    assert!(!opens_upward(anchor(68.), 400., 8., 300.));
}
