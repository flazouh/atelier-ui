use crate::update_button::consts::*;

#[test]
fn the_metrics_are_the_ones_alex_chose_from_the_mockup() {
    assert_eq!(
        (HEIGHT, PAD_LEFT, PAD_RIGHT, GAP, TEXT, RING, ICON),
        (28., 10., 13., 8., 13., 16., 14.)
    );
    assert_eq!(RING_STROKE, 2.);
    assert_eq!(HEIGHT / 2., 14., "a pill: the corner is half the height");
}
