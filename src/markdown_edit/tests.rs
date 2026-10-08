use super::*;

#[test]
fn a_mark_wraps_the_chosen_words_and_keeps_them_chosen() {
    assert_eq!(
        apply("say this now", 4..8, Format::Bold),
        ("say **this** now".into(), 6..10)
    );
    assert_eq!(
        apply("say this", 4..8, Format::Italic),
        ("say _this_".into(), 5..9)
    );
    assert_eq!(
        apply("call f", 5..6, Format::Code),
        ("call `f`".into(), 6..7)
    );
}

#[test]
fn a_mark_with_nothing_chosen_puts_the_caret_between_its_halves() {
    assert_eq!(apply("ab", 1..1, Format::Bold), ("a****b".into(), 3..3));
}

#[test]
fn a_link_wraps_the_words_and_chooses_the_address_to_type_over() {
    assert_eq!(
        apply("see docs", 4..8, Format::Link),
        ("see [docs](url)".into(), 11..14)
    );
}

#[test]
fn a_quote_and_a_list_mark_every_chosen_line() {
    assert_eq!(
        apply("one\ntwo\nthree", 0..7, Format::Quote).0,
        "> one\n> two\nthree"
    );
    assert_eq!(apply("one\ntwo", 2..5, Format::List).0, "- one\n- two");
    assert_eq!(apply("one", 1..1, Format::List), ("- one".into(), 3..3));
}
