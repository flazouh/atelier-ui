use super::{TaskCommand, read};
use crate::keys::Press;

fn key(k: &str) -> Press {
    Press { key: k.into(), ..Press::default() }
}

fn with(k: &str, secondary: bool, shift: bool, alt: bool) -> Press {
    Press { key: k.into(), secondary, shift, alt }
}

#[test]
fn bare_letters_are_the_commands_of_linears_model() {
    use TaskCommand::*;
    let table = [
        ("j", Down),
        ("k", Up),
        ("g", First),
        ("x", Select),
        ("s", SetStatus),
        ("p", SetPriority),
        ("a", SetAssignee),
        ("i", AssignToMe),
        ("l", SetLabels),
        ("c", NewTask),
        ("f", Filter),
        ("]", MoveNext),
        ("[", MovePrev),
    ];
    for (letter, command) in table {
        assert_eq!(read(&key(letter), false), Some(command), "{letter}");
    }
    assert_eq!(read(&with("g", false, true, false), false), Some(Last), "shift g goes to the end");
    assert_eq!(read(&key("G"), false), Some(Last));
}

#[test]
fn the_arrows_enter_home_end_and_escape_are_commands_too() {
    use TaskCommand::*;
    for (name, command) in [("down", Down), ("up", Up), ("left", Fold), ("right", Unfold), ("enter", Open), ("home", First), ("end", Last), ("escape", Clear)] {
        assert_eq!(read(&key(name), false), Some(command), "{name}");
    }
}

#[test]
fn while_a_text_input_has_focus_no_bare_letter_is_read() {
    for letter in ["j", "k", "x", "s", "p", "a", "i", "l", "c", "f", "g"] {
        assert_eq!(read(&key(letter), true), None, "{letter} must type");
    }
    assert_eq!(read(&key("enter"), true), None, "Enter belongs to the input");
    assert_eq!(read(&key("escape"), true), Some(TaskCommand::Clear), "Escape still leaves the input");
    assert_eq!(read(&key("down"), true), Some(TaskCommand::Down), "an arrow still moves the list");
    assert_eq!(read(&with("a", true, false, false), true), Some(TaskCommand::SelectAll));
}

#[test]
fn a_modifier_changes_the_meaning_and_unknown_keys_are_nothing() {
    assert_eq!(read(&with("a", true, false, false), false), Some(TaskCommand::SelectAll));
    assert_eq!(read(&with("a", true, true, false), false), None);
    assert_eq!(read(&with("s", true, false, false), false), None, "⌘S is not set status");
    assert_eq!(read(&with("j", false, false, true), false), None, "an alt press is left for the platform");
    for other in ["z", "q", "1", "space", "tab", ""] {
        assert_eq!(read(&key(other), false), None, "{other:?}");
    }
    assert_eq!(read(&with("j", false, true, false), false), None, "shift j is not j");
}

#[test]
fn every_command_has_words_and_a_chord_and_no_two_bare_letters_collide() {
    let mut letters = Vec::new();
    for &command in TaskCommand::ALL {
        assert!(!command.words().is_empty() && !command.chord().is_empty());
        let chord = command.chord();
        if chord.chars().count() == 1 {
            assert!(!letters.contains(&chord), "{chord} is used twice");
            letters.push(chord);
        }
    }
    assert_eq!(TaskCommand::SetStatus.chord(), "s");
    assert_eq!(TaskCommand::SelectAll.chord(), "⌘a");
}

#[test]
fn the_chords_read_back_to_their_commands() {
    for &command in TaskCommand::ALL {
        let chord = command.chord();
        let press = match chord {
            "⌘a" => with("a", true, false, false),
            "Escape" => key("escape"),
            "Enter" => key("enter"),
            "←" => key("left"),
            "→" => key("right"),
            "G" => with("g", false, true, false),
            other => key(other),
        };
        let read_back = read(&press, false);
        assert!(
            read_back == Some(command) || matches!((command, read_back), (TaskCommand::Last, Some(TaskCommand::Last))),
            "{command:?} via {chord}: {read_back:?}"
        );
    }
}

#[test]
fn enter_is_read_as_the_key_table_spells_it() {
    // `Press::from_keystroke` spells the key "Enter" with a capital.
    assert_eq!(read(&key("Enter"), false), Some(TaskCommand::Open));
    assert_eq!(read(&key("Enter"), true), None, "in a text input, Enter is the input's");
}
