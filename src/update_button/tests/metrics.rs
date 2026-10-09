use crate::update_button::consts::*;

/// The look is the DS `Button` Sm: 28 high, 10 padding each side, a 6 gap, text 11 and a 14 ring, with the default corner.
#[test]
fn the_metrics_are_the_button_sm_ones() {
    assert_eq!(
        (HEIGHT, PAD_LEFT, PAD_RIGHT, GAP, TEXT, RING),
        (28., 10., 10., 6., 11., 14.)
    );
    assert_eq!(RING_STROKE, 2.);
    assert_eq!(crate::theme::radius::lg(), gpui_kit::px(8.), "the default corner, not a pill");
}
