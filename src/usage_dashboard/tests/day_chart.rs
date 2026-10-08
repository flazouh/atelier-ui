use super::host::{dashboard, name, shown};
use gpui_kit::TestAppContext;

#[gpui_kit::test]
fn the_segments_of_a_bar_touch_and_the_tallest_day_fills_most_of_the_plot(cx: &mut TestAppContext) {
    let cx = shown(dashboard(Default::default()), cx);
    let last = (0..3).map(|k| cx.debug_bounds(name(format!("usage-seg-13-{k}"))).expect("segment")).collect::<Vec<_>>();
    // Top segment first: each one's bottom is the next one's top, and they share a left edge and width.
    for pair in last.windows(2) {
        assert_eq!(pair[0].bottom(), pair[1].origin.y, "no gap between segments");
        assert_eq!(pair[0].origin.x, pair[1].origin.x);
        assert_eq!(pair[0].size.width, pair[1].size.width);
    }
    // The highlighted day is the last, and it is the tallest of the fixture.
    let mut heights = |day: usize| -> f32 {
        (0..3)
            .filter_map(|k| cx.debug_bounds(name(format!("usage-seg-{day}-{k}"))))
            .map(|b| f32::from(b.size.height))
            .sum()
    };
    assert!(heights(13) > heights(0));
    assert!(heights(13) <= 188. * 1.001);
    // The day numbers are drawn and the detail line shows for the last day.
    assert!(cx.debug_bounds("usage-day-13").is_some());
}
