use super::*;

/// The numbers `pr_refs` finds, and that each range covers exactly `#N`.
fn found(text: &str) -> Vec<u64> {
    pr_refs(text)
        .into_iter()
        .map(|(range, n)| {
            assert_eq!(&text[range.clone()], format!("#{n}"), "{text:?} {range:?}");
            n
        })
        .collect()
}

#[test]
fn finds_refs_in_prose() {
    let cases: &[(&str, &[u64])] = &[
        ("Fixed in #3344.", &[3344]),
        ("See #12, #13 and #14", &[12, 13, 14]),
        ("(#7)", &[7]),
        ("Merged #3344!", &[3344]),
        ("**#42** is open", &[42]),
        ("a list:\n- #5 first\n- #6 second", &[5, 6]),
        ("é #9", &[9]),
        ("#1 Setup", &[1]),
        ("intro\n#2 next", &[2]),
        ("   #3 indented", &[3]),
        ("#3344 is merged.", &[3344]),
        ("- done\n#3344 fixes it", &[3344]),
        ("# Fix for #3344", &[3344]),
        ("- #5", &[5]),
    ];
    for (text, want) in cases {
        assert_eq!(found(text), *want, "{text:?}");
    }
}

#[test]
fn needs_a_word_boundary_before_the_hash_and_after_the_digits() {
    let cases: &[&str] = &[
        "abc#12", "x_#12", "#12abc", "#12_x", "C#9", "&#39;", "\\#12", "##12 ok",
    ];
    for text in cases {
        assert_eq!(found(text), Vec::<u64>::new(), "{text:?}");
    }
}

#[test]
fn takes_digits_only() {
    let cases: &[&str] = &["#", "# 12", "#-3", "#0", "#012", "#99999999999999999999999"];
    for text in cases {
        assert_eq!(found(text), Vec::<u64>::new(), "{text:?}");
    }
}

#[test]
fn skips_code() {
    let cases: &[&str] = &[
        "run `git show #12` first",
        "``a ` #12 ``",
        "```\nfix #12\n```",
        "~~~rust\n// #12\n~~~",
        "```\nunclosed #12",
    ];
    for text in cases {
        assert_eq!(found(text), Vec::<u64>::new(), "{text:?}");
    }
    assert_eq!(found("`code` then #12"), vec![12]);
    assert_eq!(found("```\n#1\n```\nafter #2"), vec![2]);
    // A lone backtick opens nothing.
    assert_eq!(found("a ` b #12"), vec![12]);
}

#[test]
fn skips_urls_and_links() {
    let cases: &[&str] = &[
        "https://example.com/#12",
        "see http://x.dev/a#12 now",
        "www.example.com/#34",
        "<https://example.com/#12>",
        "[the fix](https://example.com/#12)",
        "[see #12](https://example.com)",
        "[jump](#12)",
    ];
    for text in cases {
        assert_eq!(found(text), Vec::<u64>::new(), "{text:?}");
    }
    assert_eq!(found("https://example.com and #12"), vec![12]);
    assert_eq!(found("[a] #12"), vec![12]);
}
