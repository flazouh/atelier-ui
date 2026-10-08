use super::*;

#[test]
fn a_modifier_is_a_symbol_and_a_letter_is_text() {
    assert_eq!(
        parts("⌘K"),
        vec![
            KeyPart::Symbol(IconName::Command),
            KeyPart::Text("K".into())
        ]
    );
}

#[test]
fn two_symbols_in_a_row_are_two_icons() {
    assert_eq!(
        parts("⌘↵"),
        vec![
            KeyPart::Symbol(IconName::Command),
            KeyPart::Symbol(IconName::Return)
        ]
    );
}

#[test]
fn a_named_key_stays_one_run_of_text() {
    assert_eq!(parts("Esc"), vec![KeyPart::Text("Esc".into())]);
    assert_eq!(
        parts("⇧Tab"),
        vec![
            KeyPart::Symbol(IconName::Shift),
            KeyPart::Text("Tab".into())
        ]
    );
}

#[test]
fn spaces_only_separate_parts() {
    assert_eq!(
        parts("Ctrl ⌫"),
        vec![
            KeyPart::Text("Ctrl".into()),
            KeyPart::Symbol(IconName::Backspace)
        ]
    );
    assert_eq!(
        parts("Ctrl Shift K"),
        vec![
            KeyPart::Text("Ctrl".into()),
            KeyPart::Text("Shift".into()),
            KeyPart::Text("K".into()),
        ]
    );
}

/// K2: a Command chord given as it is written ("⌘⇧g") draws Control off the Mac, whoever passes it.
#[cfg(not(target_os = "macos"))]
#[test]
fn a_command_chord_draws_control_off_the_mac() {
    assert_eq!(
        parts(&Kbd::new("⌘⇧g").keys),
        vec![
            KeyPart::Symbol(IconName::Control),
            KeyPart::Symbol(IconName::Shift),
            KeyPart::Text("G".into())
        ]
    );
    assert_eq!(
        parts(&Kbd::new("Esc").keys),
        vec![KeyPart::Text("Esc".into())]
    );
}
