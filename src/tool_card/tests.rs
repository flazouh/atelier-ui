use super::{
    helpers::{avatar_letter, hidden_label, more_label, rows_shown, state_badge},
    types::{ROWS_SHOWN, ToolCardState},
};
use crate::animated_badge::BadgeStatus;

#[test]
fn a_list_shows_ten_rows_until_it_is_opened() {
    assert_eq!(rows_shown(3, false), 3);
    assert_eq!(rows_shown(12, false), ROWS_SHOWN);
    assert_eq!(rows_shown(12, true), 12);
}

#[test]
fn a_list_of_ten_or_fewer_has_no_show_more() {
    assert_eq!(more_label(10, false), None);
    assert_eq!(more_label(12, false).as_deref(), Some("Show 2 more"));
    assert_eq!(more_label(12, true).as_deref(), Some("Show less"));
}

#[test]
fn rows_the_app_left_out_are_counted() {
    assert_eq!(hidden_label(0), None);
    assert_eq!(hidden_label(1).as_deref(), Some("1 more not shown"));
    assert_eq!(hidden_label(8).as_deref(), Some("8 more not shown"));
}

#[test]
fn each_state_has_its_own_mark_and_words() {
    assert_eq!(
        state_badge(ToolCardState::Running),
        (BadgeStatus::Loading, "Running")
    );
    assert_eq!(
        state_badge(ToolCardState::Waiting),
        (BadgeStatus::Warning, "Waiting for approval")
    );
    assert_eq!(
        state_badge(ToolCardState::Done),
        (BadgeStatus::Success, "Done")
    );
    assert_eq!(
        state_badge(ToolCardState::Failed),
        (BadgeStatus::Danger, "Failed")
    );
}

#[test]
fn an_avatar_shows_the_first_letter_in_capitals() {
    assert_eq!(avatar_letter("ana"), "A");
    assert_eq!(avatar_letter("  éva"), "É");
    assert_eq!(avatar_letter("--"), "·");
}

mod drawn {
    use std::{cell::RefCell, rc::Rc};

    use gpui_kit::{
        Context, IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext,
        VisualTestContext, Window, div, px, size,
    };

    use crate::{
        icon::IconName,
        theme::{Appearance, set_appearance},
        tool_card::{
            ToolAction, ToolCard, ToolCardData, ToolCardState, ToolIcon, ToolNode, ToolProvider,
            ToolRow, ToolText, ToolTone,
        },
    };

    fn row(text: &str, key: Option<&str>) -> ToolRow {
        ToolRow {
            node: ToolNode::Stack {
                row: true,
                gap: Default::default(),
                children: vec![
                    ToolNode::Icon {
                        icon: ToolIcon::Named(IconName::Bug),
                        tone: ToolTone::Danger,
                    },
                    ToolNode::Text {
                        value: text.to_string().into(),
                        style: ToolText::Title,
                        max_lines: Some(1),
                    },
                    ToolNode::Badge {
                        value: "high".into(),
                        tone: ToolTone::Warning,
                    },
                ],
            },
            action: key.map(|k| k.to_string().into()),
        }
    }

    fn data(rows: usize, hidden: usize, state: ToolCardState, footer: usize) -> ToolCardData {
        ToolCardData {
            provider: ToolProvider {
                name: "Linear".into(),
                account: Some("acme".into()),
                letter: "L".into(),
                color: Some(9),
            },
            title: "Searched Linear, 12 tasks".into(),
            state,
            origin: Some("Alex's agent".into()),
            body: Some(ToolNode::List {
                rows: (0..rows)
                    .map(|n| row(&format!("Task {n}"), Some(&format!("open:{n}"))))
                    .collect(),
                hidden,
                empty: Some("No tasks.".into()),
            }),
            footer: (0..footer)
                .map(|n| ToolAction {
                    label: format!("Action {n}").into(),
                    key: format!("act:{n}").into(),
                })
                .collect(),
        }
    }

    struct Host {
        data: ToolCardData,
        pressed: Rc<RefCell<Vec<String>>>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let pressed = self.pressed.clone();
            div().size_full().child(
                ToolCard::new("card", self.data.clone())
                    .on_action(move |key, _, _| pressed.borrow_mut().push(key.to_string())),
            )
        }
    }

    fn open(
        data: ToolCardData,
        cx: &mut TestAppContext,
    ) -> (Rc<RefCell<Vec<String>>>, &mut VisualTestContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set_appearance(Appearance::Dark, cx);
            cx.set_reduce_motion(true);
        });
        let pressed = Rc::new(RefCell::new(Vec::new()));
        let host = Host {
            data,
            pressed: pressed.clone(),
        };
        let (_host, cx) = cx.add_window_view(|_, _| host);
        cx.simulate_resize(size(px(520.), px(900.)));
        cx.run_until_parked();
        (pressed, cx)
    }

    const ROWS: [&str; 16] = [
        "tool-card-row-0",
        "tool-card-row-1",
        "tool-card-row-2",
        "tool-card-row-3",
        "tool-card-row-4",
        "tool-card-row-5",
        "tool-card-row-6",
        "tool-card-row-7",
        "tool-card-row-8",
        "tool-card-row-9",
        "tool-card-row-10",
        "tool-card-row-11",
        "tool-card-row-12",
        "tool-card-row-13",
        "tool-card-row-14",
        "tool-card-row-15",
    ];

    fn rows_drawn(cx: &mut VisualTestContext) -> usize {
        ROWS.iter()
            .filter(|name| cx.debug_bounds(name).is_some())
            .count()
    }

    #[gpui_kit::test]
    fn the_card_draws_its_header_parts(cx: &mut TestAppContext) {
        let (_, cx) = open(data(3, 0, ToolCardState::Done, 0), cx);
        for part in [
            "tool-card",
            "tool-card-tile",
            "tool-card-title",
            "tool-card-origin",
            "tool-card-state",
        ] {
            assert!(cx.debug_bounds(part).is_some(), "{part} is drawn");
        }
        assert_eq!(rows_drawn(cx), 3);
    }

    #[gpui_kit::test]
    fn a_long_list_shows_ten_rows_and_show_more_shows_the_rest(cx: &mut TestAppContext) {
        let (_, cx) = open(data(14, 0, ToolCardState::Done, 0), cx);
        assert_eq!(rows_drawn(cx), 10);
        // The button has no selector: press just under the last row, where it sits.
        let last = cx.debug_bounds("tool-card-row-9").expect("ten rows");
        let below = gpui_kit::point(last.left() + px(30.), last.bottom() + px(14.));
        cx.simulate_click(below, Modifiers::default());
        cx.run_until_parked();
        assert_eq!(rows_drawn(cx), 14, "a press on Show more opens the list");
    }

    #[gpui_kit::test]
    fn a_press_on_a_row_gives_back_its_action(cx: &mut TestAppContext) {
        let (pressed, cx) = open(data(3, 0, ToolCardState::Done, 0), cx);
        let second = cx.debug_bounds("tool-card-row-1").expect("a row").center();
        cx.simulate_click(second, Modifiers::default());
        assert_eq!(pressed.borrow().as_slice(), ["open:1"]);
    }

    #[gpui_kit::test]
    fn the_footer_has_three_buttons_at_most(cx: &mut TestAppContext) {
        let (pressed, cx) = open(data(1, 0, ToolCardState::Done, 5), cx);
        let card = cx.debug_bounds("tool-card").expect("the card");
        // The footer is the last strip: press along it, from the left.
        let y = card.bottom() - px(24.);
        for n in 0..8 {
            cx.simulate_click(
                gpui_kit::point(card.left() + px(30. + 60. * n as f32), y),
                Modifiers::default(),
            );
        }
        let pressed = pressed.borrow();
        assert!(!pressed.is_empty(), "the footer buttons answer");
        assert!(
            pressed.iter().all(|k| k != "act:3" && k != "act:4"),
            "buttons past the third are not drawn: {pressed:?}"
        );
    }

    #[gpui_kit::test]
    fn a_running_card_with_no_body_is_only_its_header(cx: &mut TestAppContext) {
        let mut running = data(0, 0, ToolCardState::Running, 0);
        running.body = None;
        running.title = "Searching Linear…".into();
        let (_, cx) = open(running, cx);
        assert!(cx.debug_bounds("tool-card-state").is_some());
        assert_eq!(rows_drawn(cx), 0);
    }

    #[gpui_kit::test]
    fn an_empty_list_says_so_and_draws_no_row(cx: &mut TestAppContext) {
        let (_, cx) = open(data(0, 0, ToolCardState::Done, 0), cx);
        assert_eq!(rows_drawn(cx), 0);
        assert!(cx.debug_bounds("tool-card").is_some());
    }
}
