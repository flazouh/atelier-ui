use gpui_kit::{TestAppContext, px, size};

use super::*;

/// L17: the empty list looks like the other panes' empty states: a sm title over an xs muted line.
#[gpui_kit::test]
fn the_empty_list_shows_a_small_title_over_a_muted_line(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::theme::set_appearance(crate::theme::Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let (_list, cx) = cx.add_window_view(|_, cx| TaskList::new("alex", cx));
    cx.simulate_resize(size(px(600.), px(400.)));
    cx.run_until_parked();
    let title = cx.debug_bounds("tasks-empty-title").expect("the title is drawn");
    let line = cx.debug_bounds("tasks-empty-line").expect("the line is drawn");
    assert_eq!(f32::from(title.size.height), 20., "sm: 14 on a 20 line");
    assert_eq!(f32::from(line.size.height), 16., "xs: 12 on a 16 line");
    assert!(title.bottom() <= line.top(), "the title is above the line");
}
