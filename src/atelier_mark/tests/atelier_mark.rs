use gpui_kit::{IntoElement, ParentElement, Render, TestAppContext, Window, div};

use crate::{atelier_mark::AtelierMark, scale::px};

struct Host;

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        div().child(AtelierMark::new(40.))
    }
}

#[gpui_kit::test]
fn the_tile_is_square_and_the_a_is_a_little_over_half_of_it_and_centred(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_reduce_motion(true);
    });
    let (_, cx) = cx.add_window_view(|_, _| Host);
    cx.run_until_parked();
    let tile = cx.debug_bounds("atelier-mark").expect("the tile is drawn");
    let a = cx.debug_bounds("atelier-mark-a").expect("the A is drawn");
    assert_eq!((tile.size.width, tile.size.height), (px(40.), px(40.)));
    let share = f32::from(a.size.width) / f32::from(tile.size.width);
    assert!((share - 0.62).abs() < 0.01, "{share}");
    assert!((a.center().x - tile.center().x).abs() < px(0.5) && (a.center().y - tile.center().y).abs() < px(0.5), "{a:?} in {tile:?}");
}
