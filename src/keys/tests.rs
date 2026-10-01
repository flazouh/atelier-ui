use std::collections::HashSet;

use super::*;

const PROFILES: [Profile; 3] = [Profile::Standard, Profile::Vim, Profile::Off];

#[test]
fn the_standard_keys_are_gitquiets() {
    let p = Profile::Standard;
    let want = [
        (Command::GoToFile, "t"),
        (Command::GoToName, "T"),
        (Command::FileNames, "o"),
        (Command::Uses, "u"),
        (Command::NextFile, "s"),
        (Command::PreviousFile, "w"),
        (Command::MarkFile, "x"),
        (Command::ReviewMode, "r"),
        (Command::OpenAside, "A"),
        (Command::Search, "f"),
        (Command::Dismiss, "Escape"),
        (Command::ToggleDetails, "⌘b"),
        (Command::ToggleFiles, "⌘⇧b"),
        (Command::WorkingSet, "g d"),
        (Command::Repositories, "g r"),
        (Command::Activity, "g f"),
        (Command::Home, "g g"),
    ];
    for (command, chord) in want {
        assert_eq!(chord_for(p, command), Some(chord), "{command:?}");
    }
    assert_eq!(chords(p, Command::Search), ["f", "/"]);
}

#[test]
fn the_vim_keys_move_with_j_and_k_and_go_home_with_g_h() {
    let p = Profile::Vim;
    assert_eq!(chord_for(p, Command::NextFile), Some("j"));
    assert_eq!(chord_for(p, Command::PreviousFile), Some("k"));
    assert_eq!(chord_for(p, Command::OpenAside), Some("O"));
    assert_eq!(chords(p, Command::Search), ["/"]);
    assert_eq!(chord_for(p, Command::Home), Some("g h"));
}

#[test]
fn off_has_no_keys() {
    for command in Command::ALL {
        assert!(chords(Profile::Off, *command).is_empty(), "{command:?}");
    }
}

#[test]
fn no_chord_reaches_two_commands_in_any_profile() {
    for profile in PROFILES {
        let mut seen = HashSet::new();
        for command in Command::ALL {
            for chord in chords(profile, *command) {
                assert!(seen.insert(*chord), "{chord} twice in {profile:?}");
            }
        }
    }
}

#[test]
fn every_command_has_gitquiets_word_and_gist() {
    assert_eq!(KEYBOARD.len(), Command::ALL.len());
    assert_eq!(word(Command::NextFile), "Next file");
    assert_eq!(word(Command::MarkFile), "Mark file");
    assert_eq!(gist(Command::ReviewMode), "The files on the whole screen, and back");
    assert_eq!(word(Command::Dismiss), "Close");
}

#[test]
fn only_next_and_previous_repeat_while_held() {
    let held: Vec<_> = Command::ALL.iter().filter(|c| held_down(**c)).collect();
    assert_eq!(held, [&Command::NextFile, &Command::PreviousFile]);
}

fn press(key: &str) -> Press {
    Press { key: key.into(), ..Default::default() }
}

#[test]
fn a_bare_key_reads_as_its_command() {
    let p = Profile::Standard;
    let mut waiting = None;
    let at = Instant::now();
    assert_eq!(read(&press("s"), p, &mut waiting, at), Some(Command::NextFile));
    assert_eq!(read(&press("T"), p, &mut waiting, at), Some(Command::GoToName));
    assert_eq!(read(&press("/"), p, &mut waiting, at), Some(Command::Search));
    assert_eq!(read(&press("Escape"), p, &mut waiting, at), Some(Command::Dismiss));
    assert_eq!(read(&press("q"), p, &mut waiting, at), None);
}

#[test]
fn a_press_held_with_command_or_control_is_the_readers_own_unless_a_chord_names_it() {
    let p = Profile::Standard;
    let mut waiting = None;
    let at = Instant::now();
    let with = |key: &str, secondary, shift| Press { key: key.into(), secondary, shift, ..Default::default() };
    assert_eq!(read(&with("b", true, false), p, &mut waiting, at), Some(Command::ToggleDetails));
    assert_eq!(read(&with("b", true, true), p, &mut waiting, at), Some(Command::ToggleFiles));
    assert_eq!(read(&with("B", true, true), p, &mut waiting, at), Some(Command::ToggleFiles));
    // Cmd-S is the reader's save, not the next file.
    assert_eq!(read(&with("s", true, false), p, &mut waiting, at), None);
}

#[test]
fn a_sequence_waits_for_its_second_key_but_not_forever() {
    let p = Profile::Standard;
    let mut waiting = None;
    let at = Instant::now();
    assert_eq!(read(&press("g"), p, &mut waiting, at), None);
    assert!(waiting.is_some(), "g opens a sequence");
    assert_eq!(read(&press("d"), p, &mut waiting, at), Some(Command::WorkingSet));
    assert!(waiting.is_none());
    read(&press("g"), p, &mut waiting, at);
    let late = at + PATIENCE + std::time::Duration::from_millis(1);
    // Too late: the second key reads on its own, and `d` alone is nothing.
    assert_eq!(read(&press("d"), p, &mut waiting, late), None);
    // A key that opens nothing leaves nothing waiting.
    assert_eq!(read(&press("z"), p, &mut waiting, at), None);
    assert!(waiting.is_none());
}

#[test]
fn a_cap_shows_the_chord_as_the_reader_presses_it() {
    assert_eq!(cap("s"), "s");
    assert_eq!(cap("Escape"), "Esc");
    assert_eq!(cap("g d"), "g d");
    let cmd = if cfg!(target_os = "macos") { "⌘" } else { "⌃" };
    assert_eq!(cap("⌘b"), format!("{cmd}B"));
    assert_eq!(cap("⌘⇧b"), format!("{cmd}⇧B"));
}

/// Shipping from a review holds Command (Control elsewhere), so it works from the editor as well.
#[test]
fn shipping_keys_hold_a_modifier_in_every_profile_with_keys() {
    for p in [Profile::Standard, Profile::Vim] {
        assert_eq!(chord_for(p, Command::Commit), Some("⌘⇧c"));
        assert_eq!(chord_for(p, Command::Push), Some("⌘⇧u"));
        assert_eq!(chord_for(p, Command::OpenPull), Some("⌘⇧r"));
    }
    assert_eq!(word(Command::Push), "Push");
}

#[test]
fn the_comment_key_is_c_in_the_standard_profile_and_g_c_in_vim_and_none_when_off() {
    assert_eq!(chord_for(Profile::Standard, Command::Comment), Some("c"));
    assert_eq!(chord_for(Profile::Vim, Command::Comment), Some("g c"));
    assert_eq!(chord_for(Profile::Off, Command::Comment), None);
    assert_eq!(word(Command::Comment), "Comment");
    assert!(!gist(Command::Comment).is_empty());
}
/// Undo of a file's last decision is U in every profile with keys: Command-Z stays the editor's own
/// undo of typing.
#[test]
fn undo_of_a_decision_has_its_own_key() {
    for p in [Profile::Standard, Profile::Vim] {
        assert_eq!(chord_for(p, Command::UndoDecision), Some("U"));
    }
    assert_eq!(word(Command::UndoDecision), "Undo decision");
}

#[test]
fn g_t_goes_to_the_tasks_in_both_profiles() {
    for profile in [Profile::Standard, Profile::Vim] {
        assert_eq!(chords(profile, Command::GoToTasks), ["g t"], "{profile:?}");
        assert!(KEYBOARD.iter().any(|(c, w, _)| *c == Command::GoToTasks && *w == "Tasks"), "on the keyboard sheet");
    }
}

/// K2: no cap holds the other platform's modifier. Off the Mac a Command chord draws Control, and on the Mac
/// no cap draws Control. This covers every chord of the table, the panels' chords and the shell's.
#[test]
fn no_cap_holds_the_other_platforms_modifier() {
    use crate::agent_panels::chord;
    let mut all: Vec<&str> =
        PROFILES.iter().flat_map(|p| Command::ALL.iter().flat_map(move |c| chords(*p, *c).iter().copied())).collect();
    all.extend([chord::NEXT_PANEL, chord::PREVIOUS_PANEL, chord::CLOSE_TAB, chord::TOGGLE_LAYOUT, chord::TOGGLE_GROUPING]);
    all.extend(["⌘o", "⌘⇧o", "⌘,", "⌘n", "⌘b", "⌘⇧p", "⌘⇧l", "⌘↵"]);
    for chord in all {
        assert!(!cap_on(chord, false).contains('⌘'), "{chord} off the Mac: {}", cap_on(chord, false));
        assert!(!cap_on(chord, true).contains('⌃'), "{chord} on the Mac: {}", cap_on(chord, true));
    }
    assert_eq!(cap_on("⌘⇧g", false), "⌃⇧G");
    assert_eq!(cap_on("⌘⇧g", true), "⌘⇧G");
    assert_eq!(cap_on("⇧Tab", false), "⇧Tab", "a named key keeps its case");
}
