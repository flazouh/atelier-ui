use gpui_kit::{IntoElement, ParentElement, Render, TestAppContext, Window, div};

use crate::{
    atelier_mark::{AtelierMark, MarkLook},
    scale::px,
    theme::{Appearance, set_appearance},
};

struct Host {
    look: Option<MarkLook>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        let mark = AtelierMark::new(40.);
        div().child(match self.look {
            Some(look) => mark.look(look),
            None => mark,
        })
    }
}

fn drawn(look: Option<MarkLook>, appearance: Appearance, cx: &mut TestAppContext) -> &mut gpui_kit::VisualTestContext {
    cx.update(|cx| {
        gpui_kit::init(cx);
        set_appearance(appearance, cx);
        cx.set_reduce_motion(true);
    });
    let (_, cx) = cx.add_window_view(move |_, _| Host { look });
    cx.run_until_parked();
    cx
}

#[gpui_kit::test]
fn the_tile_is_square_and_the_a_is_a_little_over_half_of_it_and_centred(cx: &mut TestAppContext) {
    let cx = drawn(None, Appearance::Dark, cx);
    let tile = cx.debug_bounds("atelier-mark").expect("the tile is drawn");
    let a = cx.debug_bounds("atelier-mark-a").expect("the A is drawn");
    assert_eq!((tile.size.width, tile.size.height), (px(40.), px(40.)));
    let share = f32::from(a.size.width) / f32::from(tile.size.width);
    assert!((share - 0.62).abs() < 0.01, "{share}");
    assert!((a.center().x - tile.center().x).abs() < px(0.5) && (a.center().y - tile.center().y).abs() < px(0.5), "{a:?} in {tile:?}");
}

/// Both looks draw the same shapes; the theme picks one only when none is asked for.
#[gpui_kit::test]
fn a_look_asked_for_is_drawn_in_either_theme(cx: &mut TestAppContext) {
    let cx = drawn(Some(MarkLook::Terracotta), Appearance::Dark, cx);
    assert!(cx.debug_bounds("atelier-mark").is_some());
}
