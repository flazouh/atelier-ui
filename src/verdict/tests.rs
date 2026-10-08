use super::*;

#[test]
fn approve_needs_no_words_and_the_other_two_do() {
    assert!(enabled(Verb::Approve, "", false));
    assert!(!enabled(Verb::RequestChanges, "", false));
    assert!(!enabled(Verb::Comment, "   ", false));
    assert!(enabled(Verb::RequestChanges, "off by one", false));
    assert!(enabled(Verb::Comment, "fine", false));
}

#[test]
fn nothing_can_be_sent_while_a_verdict_is_on_its_way() {
    for verb in Verb::ALL {
        assert!(!enabled(verb, "words", true), "{verb:?}");
    }
}

#[test]
fn there_is_no_approve_on_your_own_pull_request() {
    assert_eq!(
        offered(false),
        [Verb::Approve, Verb::RequestChanges, Verb::Comment]
    );
    assert_eq!(offered(true), [Verb::RequestChanges, Verb::Comment]);
}

#[test]
fn the_summary_says_what_you_said_and_whether_it_is_still_current() {
    assert_eq!(summary(None), "not read yet by you");
    assert_eq!(
        summary(Some((Decision::Approved, true))),
        "You approved this"
    );
    assert_eq!(
        summary(Some((Decision::ChangesRequested, false))),
        "You asked for changes, at an older commit"
    );
    assert_eq!(
        summary(Some((Decision::Commented, true))),
        "You commented on this"
    );
}

#[test]
fn the_box_names_its_commit_by_seven_characters() {
    assert_eq!(about("f4a97b1c9e2d"), "f4a97b1");
    assert_eq!(hint(false), "An approval needs no words. The other two do.");
    assert_eq!(hint(true), "A comment needs words.");
    assert_eq!(placeholder(true), "Answer the review");
    assert_eq!(placeholder(false), "Say what you found");
}

/// The error for an empty box shows when a verb that needs words is pressed on an empty box, and not before.
#[test]
fn the_words_error_names_the_verb_and_shows_only_after_it_is_pressed() {
    assert_eq!(needs_words(Verb::Comment), Some("A comment needs words."));
    assert_eq!(
        needs_words(Verb::RequestChanges),
        Some("Requesting changes needs words.")
    );
    assert_eq!(needs_words(Verb::Approve), None, "an approval needs none");
}

#[gpui_kit::test]
fn nothing_is_said_of_words_until_a_verb_is_pressed_on_an_empty_box(
    cx: &mut gpui_kit::TestAppContext,
) {
    use gpui_kit::{
        AppContext as _, IntoElement, Modifiers, ParentElement, Render, Styled, Window, div, px,
    };
    struct Page(gpui_kit::Entity<VerdictBox>);
    impl Render for Page {
        fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
            div().w(px(320.)).child(self.0.clone())
        }
    }
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::theme::set_appearance(crate::theme::Appearance::Light, cx);
        cx.set_reduce_motion(true);
    });
    let (page, cx) = cx.add_window_view(|window, cx| {
        Page(cx.new(|cx| VerdictBox::new("894c659ab", true, window, cx)))
    });
    let verdict = page.read_with(cx, |p, _| p.0.clone());
    verdict.update(cx, |v, cx| {
        v.writing = true;
        cx.notify();
    });
    for _ in 0..3 {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    assert!(
        cx.debug_bounds("verdict-words-error").is_none(),
        "nothing before a press"
    );
    let comment = cx
        .debug_bounds("verdict-comment")
        .expect("Comment is drawn, and not greyed out")
        .center();
    cx.simulate_click(comment, Modifiers::default());
    for _ in 0..3 {
        cx.run_until_parked();
        page.update(cx, |_, cx| cx.notify());
    }
    assert!(
        cx.debug_bounds("verdict-words-error").is_some(),
        "the press on an empty box says why"
    );
}
