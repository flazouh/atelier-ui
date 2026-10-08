use super::*;

fn item(name: &str) -> CommandItem {
    CommandItem {
        name: name.to_string().into(),
        source: CommandSource::Agent,
        summary: "".into(),
        args_hint: None,
    }
}

#[test]
fn a_slash_at_the_start_asks_for_a_command_until_a_space() {
    assert_eq!(trigger("/", 1), Some(Trigger::Command { query: "".into() }));
    assert_eq!(
        trigger("/co", 3),
        Some(Trigger::Command { query: "co".into() })
    );
    assert_eq!(
        trigger("/goal ship it", 13),
        None,
        "past the name, the words are its arguments"
    );
    assert_eq!(trigger("a /co", 5), None, "only at the start");
}

#[test]
fn an_at_sign_asks_for_a_file_up_to_the_caret() {
    assert_eq!(
        trigger("look at @src/li", 15),
        Some(Trigger::Mention {
            start: 8,
            query: "src/li".into()
        })
    );
    assert_eq!(
        trigger("@", 1),
        Some(Trigger::Mention {
            start: 0,
            query: "".into()
        })
    );
    assert_eq!(
        trigger("mail me@home", 12),
        None,
        "an @ inside a word is not a mention"
    );
    assert_eq!(
        trigger("see @a.rs now", 13),
        None,
        "past a space the mention is done"
    );
}

#[test]
fn the_list_filters_by_fuzzy_score_and_shows_all_for_no_query() {
    let items = vec![
        item("compact"),
        item("review"),
        item("clear-goal"),
        item("goal"),
    ];
    assert_eq!(ranked("", &items), vec![0, 1, 2, 3]);
    let found = ranked("gl", &items);
    assert_eq!(
        found.first(),
        Some(&3),
        "goal ranks first for gl: {found:?}"
    );
    assert!(!found.contains(&1), "review does not match gl");
}
