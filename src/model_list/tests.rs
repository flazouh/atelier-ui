use gpui_kit::SharedString;

use super::moved;

fn ids(of: &[&str]) -> Vec<SharedString> {
    of.iter().map(|s| SharedString::from(s.to_string())).collect()
}

#[test]
fn a_row_goes_before_the_one_it_is_dropped_on() {
    let order = ids(&["a", "b", "c", "d"]);
    assert_eq!(moved(&order, &"d".into(), Some(&"b".into())), ids(&["a", "d", "b", "c"]), "up");
    assert_eq!(moved(&order, &"a".into(), Some(&"c".into())), ids(&["b", "a", "c", "d"]), "down, to just before c");
}

#[test]
fn a_row_dropped_below_the_last_goes_to_the_end() {
    assert_eq!(moved(&ids(&["a", "b", "c"]), &"a".into(), None), ids(&["b", "c", "a"]));
}

#[test]
fn a_row_dropped_on_itself_or_unknown_changes_nothing() {
    let order = ids(&["a", "b", "c"]);
    assert_eq!(moved(&order, &"b".into(), Some(&"b".into())), order);
    assert_eq!(moved(&order, &"z".into(), Some(&"a".into())), order);
}

mod drawn {
    use std::{cell::RefCell, rc::Rc};

    use gpui_kit::{IntoElement, ParentElement, Render, SharedString, Styled, TestAppContext, div, px, size};

    use crate::{
        model_list::{ModelList, ModelRow},
        theme::{Appearance, set_appearance},
    };

    #[derive(Default)]
    struct Heard {
        default: Option<SharedString>,
        order: Option<Vec<SharedString>>,
        hide: Option<(SharedString, bool)>,
    }

    struct Host(Rc<RefCell<Heard>>);

    impl Render for Host {
        fn render(&mut self, _: &mut gpui_kit::Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
            let (a, b, c) = (self.0.clone(), self.0.clone(), self.0.clone());
            div().w(px(500.)).child(
                ModelList::new("models", "claude", vec![ModelRow::new("a", "Opus 5.5"), ModelRow::new("b", "Sonnet 5.5"), ModelRow::new("c", "Haiku 5.5").hidden(true)])
                    .default(Some("a".into()))
                    .on_default(move |id, _, _| a.borrow_mut().default = Some(id.clone()))
                    .on_reorder(move |order, _, _| b.borrow_mut().order = Some(order))
                    .on_hide(move |id, hidden, _, _| c.borrow_mut().hide = Some((id.clone(), hidden))),
            )
        }
    }

    fn open(cx: &mut TestAppContext) -> (Rc<RefCell<Heard>>, &mut gpui_kit::VisualTestContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Dark, cx);
        });
        let heard = Rc::new(RefCell::new(Heard::default()));
        let shared = heard.clone();
        let (_host, cx) = cx.add_window_view(move |_, _| Host(shared));
        cx.simulate_resize(size(px(500.), px(300.)));
        cx.run_until_parked();
        (heard, cx)
    }

    /// A press on a row's star makes that model the default.
    #[gpui_kit::test]
    fn a_press_on_a_star_makes_the_model_the_default(cx: &mut TestAppContext) {
        let (heard, cx) = open(cx);
        let star = cx.debug_bounds("model-star-claude-b").expect("the star is drawn");
        cx.simulate_click(star.center(), gpui_kit::Modifiers::default());
        cx.run_until_parked();
        assert_eq!(heard.borrow().default.as_ref().map(|s| s.as_ref()), Some("b"));
    }

    /// A row dragged by its grip and dropped on another goes before it, and the list says the new order.
    #[gpui_kit::test]
    fn a_row_dragged_onto_another_goes_before_it(cx: &mut TestAppContext) {
        let (heard, cx) = open(cx);
        let (from, onto) = (cx.debug_bounds("model-row-claude-c").unwrap(), cx.debug_bounds("model-row-claude-a").unwrap());
        let start = gpui_kit::point(from.left() + px(14.), from.center().y);
        let end = gpui_kit::point(onto.left() + px(100.), onto.center().y);
        let none = gpui_kit::Modifiers::default();
        cx.simulate_mouse_move(start, None, none);
        cx.simulate_mouse_down(start, gpui_kit::MouseButton::Left, none);
        for step in 1..=8 {
            let at = gpui_kit::point(start.x + (end.x - start.x) * (step as f32 / 8.), start.y + (end.y - start.y) * (step as f32 / 8.));
            cx.simulate_mouse_move(at, Some(gpui_kit::MouseButton::Left), none);
            cx.run_until_parked();
        }
        cx.simulate_mouse_up(end, gpui_kit::MouseButton::Left, none);
        cx.run_until_parked();
        let order: Vec<String> = heard.borrow().order.clone().expect("the list reported an order").iter().map(|s| s.to_string()).collect();
        assert_eq!(order, ["c", "a", "b"]);
    }

    /// An eye hides a model from the picker, and on a hidden one shows it again; the default has no eye to press.
    #[gpui_kit::test]
    fn an_eye_hides_a_model_and_the_default_has_none_to_press(cx: &mut TestAppContext) {
        let (heard, cx) = open(cx);
        let none = gpui_kit::Modifiers::default();
        let at = cx.debug_bounds("model-eye-claude-b").expect("an eye").center();
        cx.simulate_click(at, none);
        cx.run_until_parked();
        assert_eq!(heard.borrow().hide.as_ref().map(|(id, hidden)| (id.to_string(), *hidden)), Some(("b".to_string(), true)));
        let at = cx.debug_bounds("model-eye-claude-c").expect("an eye").center();
        cx.simulate_click(at, none);
        cx.run_until_parked();
        assert_eq!(heard.borrow().hide.as_ref().map(|(id, hidden)| (id.to_string(), *hidden)), Some(("c".to_string(), false)), "a hidden one is shown again");
        heard.borrow_mut().hide = None;
        let at = cx.debug_bounds("model-eye-claude-a").expect("an eye").center();
        cx.simulate_click(at, none);
        cx.run_until_parked();
        assert_eq!(heard.borrow().hide, None, "the default cannot be hidden");
    }
}

/// The star is a soft gold on a dark page and the full gold on a light one, never the dull brown of the warning text tone on white.
#[test]
fn the_star_is_gold_on_both_pages() {
    use crate::themes::named;
    for name in ["atelier Light", "atelier Dark"] {
        let theme = named(name).unwrap();
        let star = gpui_kit::Rgba::from(super::star_colour(&theme));
        assert!(star.r > 0.85 && star.g > 0.55 && star.b < 0.55, "{name}: a gold, not a brown: {star:?}");
    }
}
