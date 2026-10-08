use super::*;

/// The tick for the message in view is whole; its neighbours are 68%, 44% and then 25% of it.
#[test]
fn ticks_swell_like_a_dock_round_the_lit_one() {
    let scales: Vec<f32> = (0..6).map(|i| tick_scale(i, Some(2))).collect();
    assert_eq!(scales, [0.44, 0.68, 1., 0.68, 0.44, 0.25]);
    assert_eq!(
        tick_scale(0, None),
        0.25,
        "with none lit every tick is short"
    );
}

/// The label and the answer's start are cut at a word, with an ellipsis, as the web does.
#[test]
fn an_excerpt_is_cut_at_a_word() {
    assert_eq!(excerpt("short text", 56), "short text");
    let long = "Run these shell commands one at a time, each as its own tool call please";
    let cut = excerpt(long, 40);
    assert!(cut.ends_with('…') && cut.chars().count() <= 41, "{cut}");
    assert!(!cut.trim_end_matches('…').ends_with(' '));
    assert_eq!(excerpt("  many   spaces \n here ", 56), "many spaces here");
}

#[test]
fn the_ticks_shrink_when_many_messages_would_not_fit() {
    assert_eq!(item_size(5, 400.), 14.);
    assert_eq!(item_size(100, 400.), 4.);
    assert!((item_size(40, 400.) - 10.).abs() < 1e-4);
}

mod card {
    use gpui_kit::{
        Context, IntoElement, ParentElement, Render, Styled, TestAppContext, Window, div,
    };

    use crate::{
        message_rail::{MessageRail, RailItem},
        scale::px,
        theme::{Appearance, set_appearance},
    };

    struct Host {
        items: Vec<RailItem>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .relative()
                .w(px(700.))
                .h(px(400.))
                .child(MessageRail::new("rail", self.items.clone(), 0))
        }
    }

    fn item(label: &str, description: Option<&str>) -> RailItem {
        RailItem {
            label: label.to_string().into(),
            description: description.map(|d| d.to_string().into()),
        }
    }

    /// Hovers the first tick of a rail of `items` and gives the card's bounds and the tick's.
    fn hovered(
        items: Vec<RailItem>,
        cx: &mut TestAppContext,
    ) -> (
        gpui_kit::Bounds<gpui_kit::Pixels>,
        gpui_kit::Bounds<gpui_kit::Pixels>,
    ) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Dark, cx);
            cx.set_reduce_motion(true);
        });
        let (_, cx) = cx.add_window_view(move |_, _| Host { items });
        cx.simulate_resize(gpui_kit::size(px(700.), px(400.)));
        cx.run_until_parked();
        let tick = cx.debug_bounds("rail-tick-0").expect("the tick is drawn");
        cx.simulate_mouse_move(tick.center(), None, gpui_kit::Modifiers::default());
        cx.run_until_parked();
        (cx.debug_bounds("rail-card").expect("the card shows"), tick)
    }

    /// A card that is one line of words is one line tall, not the height of a message with an answer.
    #[gpui_kit::test]
    fn a_card_with_only_a_label_is_one_line_tall(cx: &mut TestAppContext) {
        let (card, _) = hovered(vec![item("Hello", None), item("Two", None)], cx);
        assert!(f32::from(card.size.height) < 36., "{card:?}");
        assert!(
            f32::from(card.size.width) < 120.,
            "as wide as its words: {card:?}"
        );
    }

    #[gpui_kit::test]
    fn an_answer_makes_it_taller_but_never_past_two_lines_of_it(cx: &mut TestAppContext) {
        let long = "word ".repeat(80);
        let (alone, _) = hovered(vec![item("Hello", None)], cx);
        let (answered, _) = hovered(vec![item("Hello", Some(long.as_str()))], cx);
        assert!(answered.size.height > alone.size.height);
        assert!(
            f32::from(answered.size.height) <= 80.,
            "a label and two lines: {answered:?}"
        );
        assert!(f32::from(answered.size.width) <= 256., "{answered:?}");
    }

    #[gpui_kit::test]
    fn the_card_is_centred_on_its_tick(cx: &mut TestAppContext) {
        let (card, tick) = hovered(
            vec![item("Hello", Some("and the answer")), item("Two", None)],
            cx,
        );
        let off = (f32::from(card.center().y) - f32::from(tick.center().y)).abs();
        assert!(off < 1.5, "{card:?} against {tick:?}");
        assert!(card.right() < tick.left(), "beside the tick, not over it");
    }
}
