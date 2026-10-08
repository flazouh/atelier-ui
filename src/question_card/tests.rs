use super::{helpers::{answer_text, is_answered, pressed}, types::QuestionView};

fn question(multiple: bool) -> QuestionView {
    QuestionView {
        header: "Crates".into(),
        question: "Which crates?".into(),
        options: vec![("Few".into(), "One or two".into()), ("None".into(), "Std only".into()), ("Many".into(), "Whatever fits".into())],
        multiple,
    }
}

#[test]
fn an_answer_is_the_labels_picked_then_the_readers_own_words() {
    assert_eq!(answer_text(&question(false), &[1], ""), "None");
    assert_eq!(answer_text(&question(true), &[0, 2], ""), "Few, Many");
    assert_eq!(answer_text(&question(true), &[0], "  and serde  "), "Few, and serde");
    assert_eq!(answer_text(&question(false), &[], "my own words"), "my own words");
    assert_eq!(answer_text(&question(false), &[1], "my own words"), "my own words", "words replace a single pick");
    assert_eq!(answer_text(&question(false), &[9], ""), "", "a pick the question does not have");
}

#[test]
fn a_question_is_answered_by_a_pick_or_by_words() {
    assert!(!is_answered(&[], ""));
    assert!(!is_answered(&[], "   "));
    assert!(is_answered(&[0], ""));
    assert!(is_answered(&[], "words"));
}

#[test]
fn a_single_choice_takes_one_pick_and_several_toggle() {
    assert_eq!(pressed(&[], 1, false), vec![1]);
    assert_eq!(pressed(&[1], 2, false), vec![2], "another replaces it");
    assert_eq!(pressed(&[2], 2, false), Vec::<usize>::new(), "the same lets go");
    assert_eq!(pressed(&[0], 2, true), vec![0, 2]);
    assert_eq!(pressed(&[2, 0], 2, true), vec![0], "a second press takes it off");
}
